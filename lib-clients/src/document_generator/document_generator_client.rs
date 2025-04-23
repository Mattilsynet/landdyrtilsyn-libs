use crate::client::ApiClient;
use crate::document_generator::response::{InterntDokument, VedleggDokument};
use crate::error::ApiError;
use crate::error::Result;
use reqwest_middleware::reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap};
use tracing::{info, instrument};
use uuid::Uuid;

pub struct DokumentGeneratorClient {
    api_client: ApiClient,
}

impl DokumentGeneratorClient {
    #[instrument(name = "Creating DokumentGeneratorClient")]
    pub async fn new(base_url_prefix: &str, auth_config_prefix: &str) -> Self {
        DokumentGeneratorClient {
            api_client: ApiClient::new(base_url_prefix, auth_config_prefix).await,
        }
    }

    #[instrument(
        name = "Creating interndokument",
        skip(self, interndokument),
        fields(request_id = %Uuid::new_v4())
    )]
    pub async fn create_interndokument(&self, interndokument: InterntDokument) -> Result<Vec<u8>> {
        let json_body =
            serde_json::to_string(&interndokument).expect("Failed to serialize dokument");

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/pdf".parse().unwrap());
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        let response = self
            .api_client
            .get_client()
            .post(format!("{}/v1/interntdokument", &self.api_client.get_base_url()).as_str())
            .headers(headers)
            .bearer_auth(self.api_client.get_token())
            .body(json_body)
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;

        let status = response.status();

        if response.status().is_success() {
            let bytes = response.bytes().await?;
            let byte_array: Vec<u8> = bytes.to_vec();

            info!("Byte array length: {}", byte_array.len());
            Ok(byte_array)
        } else {
            Err(ApiError::ClientError {
                resource: "DokumentGenerator".to_string(),
                error_message: format!(
                    "Failed to generate interndokument, HTTP Status: {}, response {:?}",
                    status,
                    response.text().await
                ),
            })
        }
    }

    #[instrument(
        name = "Creating vedlegg dokument",
        skip(self, vedlegg),
        fields(request_id = %Uuid::new_v4())
    )]
    pub async fn create_vedlegg(&self, vedlegg: VedleggDokument) -> Result<Vec<u8>> {
        let json_body = serde_json::to_string(&vedlegg).expect("Failed to serialize dokument");

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/pdf".parse().unwrap());
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        let response = self
            .api_client
            .get_client()
            .post(format!("{}/v1/vedlegg", &self.api_client.get_base_url()).as_str())
            .headers(headers)
            .bearer_auth(self.api_client.get_token())
            .body(json_body)
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;

        let status = response.status();

        if response.status().is_success() {
            let bytes = response.bytes().await?;
            let byte_array: Vec<u8> = bytes.to_vec();

            info!("Byte array length: {}", byte_array.len());
            Ok(byte_array)
        } else {
            Err(ApiError::ClientError {
                resource: "DokumentGenerator".to_string(),
                error_message: format!(
                    "Failed to generate interndokument, HTTP Status: {}, response {:?}",
                    status,
                    response.text().await
                ),
            })
        }
    }
}
