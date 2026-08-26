use std::collections::HashMap;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use lib_nats::chunked_upload::{
    ChunkedUploadClient, ChunkedUploadClientConfig, ChunkedUploadServer, ChunkedUploadServerConfig,
    CompletedUpload, StoredUpload, UploadRequest, UploadStore, UploadStoreError,
};
use tokio::sync::{Mutex, oneshot};
use uuid::Uuid;

struct MemoryStore {
    uploads: Arc<Mutex<HashMap<String, CompletedUpload>>>,
    inspect_count: AtomicUsize,
    failover_claimed: Arc<AtomicBool>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
}

impl MemoryStore {
    fn new(
        uploads: Arc<Mutex<HashMap<String, CompletedUpload>>>,
        failover_claimed: Arc<AtomicBool>,
        shutdown: oneshot::Sender<()>,
    ) -> Self {
        Self {
            uploads,
            inspect_count: AtomicUsize::new(0),
            failover_claimed,
            shutdown: Mutex::new(Some(shutdown)),
        }
    }

    async fn shutdown(&self) {
        if let Some(shutdown) = self.shutdown.lock().await.take() {
            let _ = shutdown.send(());
        }
    }
}

#[async_trait]
impl UploadStore for MemoryStore {
    async fn inspect(&self, upload_id: &str) -> Result<Option<StoredUpload>, UploadStoreError> {
        self.inspect_count.fetch_add(1, Ordering::Relaxed);
        if !self.failover_claimed.swap(true, Ordering::SeqCst) {
            self.shutdown().await;
        }
        Ok(self
            .uploads
            .lock()
            .await
            .get(upload_id)
            .map(|upload| StoredUpload {
                size: upload.descriptor.size,
                sha256: upload.descriptor.sha256.clone(),
            }))
    }

    async fn store(&self, upload: CompletedUpload) -> Result<(), UploadStoreError> {
        let mut uploads = self.uploads.lock().await;
        if let Some(existing) = uploads.get(&upload.descriptor.upload_id) {
            if existing.descriptor.size == upload.descriptor.size
                && existing.descriptor.sha256 == upload.descriptor.sha256
            {
                return Ok(());
            }
            return Err(UploadStoreError::Conflict);
        }
        uploads.insert(upload.descriptor.upload_id.clone(), upload);
        Ok(())
    }
}

struct NatsServer {
    child: Child,
    config_path: PathBuf,
    url: String,
}

impl NatsServer {
    fn start() -> Option<Self> {
        match Command::new("nats-server")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
            Err(error) => panic!("failed to execute nats-server: {error}"),
        }

        let listener = TcpListener::bind("127.0.0.1:0").expect("reserve an ephemeral port");
        let port = listener.local_addr().expect("read ephemeral port").port();
        drop(listener);
        let config_path =
            std::env::temp_dir().join(format!("lib-nats-chunked-upload-{}.conf", Uuid::new_v4()));
        std::fs::write(&config_path, "max_payload: 8388608\n")
            .expect("write temporary nats-server config");
        let child = Command::new("nats-server")
            .arg("--addr")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(port.to_string())
            .arg("--config")
            .arg(&config_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|error| {
                let _ = std::fs::remove_file(&config_path);
                panic!("start nats-server: {error}");
            });
        Some(Self {
            child,
            config_path,
            url: format!("nats://127.0.0.1:{port}"),
        })
    }
}

impl Drop for NatsServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.config_path);
    }
}

async fn connect(url: &str) -> async_nats::Client {
    for _ in 0..50 {
        if let Ok(client) = async_nats::connect(url).await {
            return client;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("nats-server did not become ready");
}

#[tokio::test]
async fn restarts_large_upload_after_selected_receiver_shuts_down() {
    let Some(server_process) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let nats = connect(&server_process.url).await;
    let base_subject = format!("test.chunked.{}", Uuid::new_v4());
    let uploads = Arc::new(Mutex::new(HashMap::new()));
    let failover_claimed = Arc::new(AtomicBool::new(false));
    let (first_shutdown_tx, first_shutdown_rx) = oneshot::channel();
    let (second_shutdown_tx, second_shutdown_rx) = oneshot::channel();
    let first_store = Arc::new(MemoryStore::new(
        uploads.clone(),
        failover_claimed.clone(),
        first_shutdown_tx,
    ));
    let second_store = Arc::new(MemoryStore::new(
        uploads.clone(),
        failover_claimed,
        second_shutdown_tx,
    ));
    let server_config = ChunkedUploadServerConfig {
        base_subject: base_subject.clone(),
        begin_queue: "test-upload-receivers".to_string(),
        shutdown_grace: Duration::ZERO,
        cleanup_interval: Duration::from_millis(50),
        ..ChunkedUploadServerConfig::default()
    };
    let first_server =
        ChunkedUploadServer::new(nats.clone(), server_config.clone(), first_store.clone())
            .expect("create first upload server");
    let second_server = ChunkedUploadServer::new(nats.clone(), server_config, second_store.clone())
        .expect("create second upload server");
    assert_ne!(first_server.receiver_id(), second_server.receiver_id());

    let first_task = tokio::spawn(first_server.run(async move {
        let _ = first_shutdown_rx.await;
    }));
    let second_task = tokio::spawn(second_server.run(async move {
        let _ = second_shutdown_rx.await;
    }));
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = ChunkedUploadClient::new(
        nats,
        ChunkedUploadClientConfig {
            base_subject,
            request_timeout: Duration::from_secs(2),
            ..ChunkedUploadClientConfig::default()
        },
    );
    let bytes = Bytes::from(vec![0x5a; 2_500_123]);
    let receipt = client
        .upload(UploadRequest {
            upload_id: "media-first".to_string(),
            bytes: bytes.clone(),
            filename: Some("first.bin".to_string()),
            content_type: Some("application/octet-stream".to_string()),
        })
        .await
        .expect("upload restarts on the surviving receiver");
    assert_eq!(receipt.size, bytes.len() as u64);
    assert_eq!(uploads.lock().await["media-first"].bytes, bytes);
    assert!(first_store.inspect_count.load(Ordering::Relaxed) > 0);
    assert!(second_store.inspect_count.load(Ordering::Relaxed) > 0);

    client
        .upload(UploadRequest {
            upload_id: "media-first".to_string(),
            bytes: bytes.clone(),
            filename: Some("first.bin".to_string()),
            content_type: Some("application/octet-stream".to_string()),
        })
        .await
        .expect("already stored upload is idempotent");

    first_store.shutdown().await;
    second_store.shutdown().await;
    first_task
        .await
        .expect("join first receiver")
        .expect("first receiver shuts down cleanly");
    second_task
        .await
        .expect("join second receiver")
        .expect("second receiver shuts down cleanly");
}
