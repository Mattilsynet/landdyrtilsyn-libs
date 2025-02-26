use std::env;

use google_cloud_storage::{
    client::{google_cloud_auth::credentials::CredentialsFile, Client, ClientConfig},
    http::objects::upload::{Media, UploadObjectRequest, UploadType},
    http::objects::{download::Range, get::GetObjectRequest},
};

#[derive(Clone)]
pub struct GcsClient {
    client: Client,
    bucket: String,
}

impl GcsClient {
    // TODO: Setup with retry and exponential backoff
    pub async fn new(bucket: &str) -> GcsClient {
        let config = ClientConfig::default().with_auth().await.unwrap();
        let client = Client::new(config);
        let bucket = bucket.to_string();
        Self { client, bucket }
    }

    pub async fn with_credentials(env_var: &str, bucket: &str) -> GcsClient {
        let google_credentials = env::var(env_var).unwrap().to_string();
        let credentials_file = CredentialsFile::new_from_str(&google_credentials)
            .await
            .unwrap();
        let config = ClientConfig::default()
            .with_credentials(credentials_file)
            .await
            .unwrap();

        let client = Client::new(config);
        let bucket = bucket.to_string();

        GcsClient { client, bucket }
    }

    pub async fn download_object(&self, object_name: String) -> Option<Vec<u8>> {
        let data = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.clone(),
                    object: object_name.clone(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await;
        match data {
            Ok(data) => Some(data),
            Err(e) => {
                tracing::error!("Error retrieving file {} from bucket: {}", object_name, e);
                None
            }
        }
    }

    pub async fn upload_file(&self, path: &str, content: Vec<u8>) {
        let upload_type = UploadType::Simple(Media::new(path.to_string()));
        let _uploaded = self
            .client
            .upload_object(
                &UploadObjectRequest {
                    bucket: self.bucket.clone(),
                    ..Default::default()
                },
                content,
                &upload_type,
            )
            .await;
    }
}
