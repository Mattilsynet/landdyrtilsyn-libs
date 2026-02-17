use std::env;

use bytes::Bytes;
use google_cloud_auth::credentials::{Credentials, service_account};
use google_cloud_storage::client::Storage;
use serde_json::Value;

#[derive(Clone)]
pub struct GcsClient {
    client: Storage,
    bucket: String,
}

impl GcsClient {
    // TODO: Setup with retry and exponential backoff
    pub async fn new(bucket: &str) -> GcsClient {
        let client = Storage::builder()
            // Use default ADC; builder will find credentials automatically
            .build()
            .await
            .unwrap();
        let bucket = bucket.to_string();
        Self { client, bucket }
    }

    // Initialize with explicit service account credentials JSON from env var
    pub async fn with_credentials(env_var: &str, bucket: &str) -> GcsClient {
        let google_credentials = env::var(env_var).unwrap();
        let key: Value = serde_json::from_str(&google_credentials).unwrap();
        let cred: Credentials = service_account::Builder::new(key).build().unwrap();

        let client = Storage::builder()
            .with_credentials(cred)
            .build()
            .await
            .unwrap();

        let bucket = bucket.to_string();
        GcsClient { client, bucket }
    }

    pub async fn download_object(&self, object_name: String) -> Option<Vec<u8>> {
        let bucket_path = format!("projects/_/buckets/{}", self.bucket);
        let result = self
            .client
            .read_object(bucket_path, object_name.clone())
            .send()
            .await;

        match result {
            Ok(mut resp) => {
                let mut out: Vec<u8> = Vec::new();
                while let Some(next) = resp.next().await {
                    match next {
                        Ok(chunk) => out.extend_from_slice(&chunk),
                        Err(e) => {
                            tracing::error!(
                                "Error streaming file {} from bucket: {}",
                                object_name,
                                e
                            );
                            return None;
                        }
                    }
                }
                Some(out)
            }
            Err(e) => {
                tracing::error!("Error retrieving file {} from bucket: {}", object_name, e);
                None
            }
        }
    }

    pub async fn upload_file(&self, path: &str, content: Vec<u8>) {
        let bucket_path = format!("projects/_/buckets/{}", self.bucket);
        let payload: Bytes = Bytes::from(content);
        let _uploaded = self
            .client
            .write_object(bucket_path, path.to_string(), payload)
            .send_buffered()
            .await;
    }
}
