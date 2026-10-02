use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use async_nats::jetstream::{Context, stream};
use lib_mt_ansatte::{Error, MtAnsatte, Result};
use uuid::Uuid;

const STREAM: &str = "ansatte";

struct NatsServer {
    child: Child,
    config_path: PathBuf,
    store_dir: PathBuf,
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

        let name = format!("lib-mt-ansatte-{}", Uuid::new_v4());
        let store_dir = std::env::temp_dir().join(format!("{name}-js"));
        std::fs::create_dir_all(&store_dir).expect("create jetstream store dir");
        let config_path = std::env::temp_dir().join(format!("{name}.conf"));
        let config = format!(
            "jetstream {{ store_dir: '{}' }}\nmax_payload: 8388608\n",
            store_dir.display()
        );
        std::fs::write(&config_path, config).expect("write temporary nats-server config");

        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("reserve ephemeral port");
        let port = listener.local_addr().expect("read ephemeral port").port();
        drop(listener);

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
                let _ = std::fs::remove_dir_all(&store_dir);
                panic!("start nats-server: {error}");
            });

        Some(Self {
            child,
            config_path,
            store_dir,
            url: format!("nats://127.0.0.1:{port}"),
        })
    }
}

impl Drop for NatsServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.config_path);
        let _ = std::fs::remove_dir_all(&self.store_dir);
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

async fn jetstream(server: &NatsServer) -> (MtAnsatte, Context) {
    let client = connect(&server.url).await;
    let context = async_nats::jetstream::new(client.clone());
    context
        .get_or_create_stream(stream::Config {
            name: STREAM.to_string(),
            subjects: vec!["ansatte.>".to_string()],
            storage: stream::StorageType::Memory,
            ..Default::default()
        })
        .await
        .expect("create stream");
    (MtAnsatte::new(client), context)
}

async fn publiser(context: &Context, subject: &str, payload: serde_json::Value) {
    context
        .publish(
            subject.to_string(),
            serde_json::to_vec(&payload).expect("serialize").into(),
        )
        .await
        .expect("publish")
        .await
        .expect("publish ack");
}

fn payload(navn: Option<&str>, enhet: Option<&str>, enabled: bool) -> serde_json::Value {
    serde_json::json!({
        "accountEnabled": enabled,
        "displayName": navn,
        "department": enhet,
        "employeeId": "99990000",
        "ignoredField": "verdi",
    })
}

#[tokio::test]
async fn henter_ansattnavn() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), true),
    )
    .await;

    assert_eq!(
        ansatte.ansatt_navn("99990001").await.unwrap(),
        Some("Test Person En".to_string())
    );
}

#[tokio::test]
async fn henter_enhetsnavn() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), true),
    )
    .await;

    assert_eq!(
        ansatte.enhet_navn("M10000").await.unwrap(),
        Some("Seksjon En".to_string())
    );
}

#[tokio::test]
async fn hoyeste_sekvens_vinner_pa_samme_subject() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    let subject = "ansatte.M10000.99990001";
    publiser(
        &context,
        subject,
        payload(Some("Gammelt Navn"), Some("Seksjon En"), true),
    )
    .await;
    publiser(
        &context,
        subject,
        payload(Some("Nytt Navn"), Some("Seksjon En"), true),
    )
    .await;

    assert_eq!(
        ansatte.ansatt_navn("99990001").await.unwrap(),
        Some("Nytt Navn".to_string())
    );
}

#[tokio::test]
async fn hoyeste_sekvens_vinner_pa_tvers_av_enheter() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Gammelt Navn"), Some("Seksjon En"), true),
    )
    .await;
    publiser(
        &context,
        "ansatte.M20000.99990001",
        payload(Some("Nytt Navn"), Some("Seksjon To"), true),
    )
    .await;

    assert_eq!(
        ansatte.ansatt_navn("99990001").await.unwrap(),
        Some("Nytt Navn".to_string())
    );
}

#[tokio::test]
async fn nyere_ukjent_subject_vinner() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Gammelt Navn"), Some("Seksjon En"), true),
    )
    .await;
    publiser(
        &context,
        "ansatte.ukjent.99990001",
        payload(Some("Uten Enhet"), None, true),
    )
    .await;

    assert_eq!(
        ansatte.ansatt_navn("99990001").await.unwrap(),
        Some("Uten Enhet".to_string())
    );
}

#[tokio::test]
async fn enhet_velger_nyeste_melding_pa_tvers_av_ansatte() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), true),
    )
    .await;
    publiser(
        &context,
        "ansatte.M10000.99990002",
        payload(Some("Test Person To"), Some("Seksjon To"), true),
    )
    .await;

    assert_eq!(
        ansatte.enhet_navn("M10000").await.unwrap(),
        Some("Seksjon To".to_string())
    );
}

#[tokio::test]
async fn deaktivert_ansatt_kan_slaas_opp() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), false),
    )
    .await;

    assert_eq!(
        ansatte.ansatt_navn("99990001").await.unwrap(),
        Some("Test Person En".to_string())
    );
}

#[tokio::test]
async fn nyeste_melding_uten_navn_gir_none() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    let subject = "ansatte.M10000.99990001";
    publiser(
        &context,
        subject,
        payload(Some("Gammelt Navn"), Some("Seksjon En"), true),
    )
    .await;
    publiser(&context, subject, payload(None, Some("Seksjon En"), true)).await;

    assert_eq!(ansatte.ansatt_navn("99990001").await.unwrap(), None);
}

#[tokio::test]
async fn ingen_treff_gir_none() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), true),
    )
    .await;

    assert_eq!(ansatte.ansatt_navn("99990099").await.unwrap(), None);
    assert_eq!(ansatte.enhet_navn("M99999").await.unwrap(), None);
}

#[tokio::test]
async fn manglende_stream_gir_feil() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let client = connect(&server.url).await;
    let ansatte = MtAnsatte::new(client);

    let result: Result<Option<String>> = ansatte.ansatt_navn("99990001").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn ugyldig_ansatt_id_avvises_for_nats() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let client = connect(&server.url).await;
    let ansatte = MtAnsatte::new(client);

    let result: Result<Option<String>> = ansatte.ansatt_navn("17912.*").await;
    assert!(matches!(result, Err(Error::InvalidId(_))));
}

#[tokio::test]
async fn ugyldig_payload_gir_feil() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    context
        .publish("ansatte.M10000.99990001", "ikke-json".into())
        .await
        .expect("publish")
        .await
        .expect("publish ack");

    let result: Result<Option<String>> = ansatte.ansatt_navn("99990001").await;
    assert!(matches!(result, Err(Error::Payload(_))));
}
