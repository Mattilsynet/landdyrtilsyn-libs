use crate::bilde::response::ImageMetaData;
use crate::client::ApiClient;
use crate::error::ApiError;
use crate::error::Result;
use tracing::{error, info};

pub struct BildeClient {
    api_client: ApiClient,
}

impl BildeClient {
    pub async fn new() -> Self {
        BildeClient {
            api_client: ApiClient::new("BILDE_API", "KEYCLOAK").await,
        }
    }

    pub async fn hent_bilde(
        &self,
        bilde_id: String,
        storrelse: String,
    ) -> Result<(Vec<u8>, String)> {
        let url = format!(
            "{}kategorier/bilder/{}/{}?filter.app=MAKKS",
            &self.api_client.get_base_url(),
            bilde_id,
            storrelse,
        );
        info!("Henter bilde fra: {:?}", url);
        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();

            let bilde_data = response.bytes().await.map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;

            Ok((bilde_data.to_vec(), content_type))
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente bilde. bilde_id {}, error code {}, error message {}",
                bilde_id, status, error_message
            );
            Err(ApiError::ClientError {
                resource: "bilde-api".to_string(),
                error_message: format!(
                    "Failed to fetch bilde. HTTP Status: {}, response: {}",
                    status, error_message
                ),
            })
        }
    }

    pub async fn hent_bilde_metadata(&self, bilde_id: String) -> Result<ImageMetaData> {
        let url = format!(
            "{}/kategorier/bilder/{}?filter.app=MAKKS",
            &self.api_client.get_base_url(),
            bilde_id,
        );
        info!("Henter bilde metadata fra: {:?}", url);
        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let metadata: ImageMetaData = response
                .json()
                .await
                .map_err(|e| ApiError::ParseError(e.to_string()))?;
            info!("Hentet bilde metadata {:?} fra bilde api.", metadata);
            Ok(metadata)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente bilde medtadata. bilde_id {}, error code {}, error message {}",
                bilde_id, status, error_message
            );
            Err(ApiError::ClientError {
                resource: "bilde-api".to_string(),
                error_message: format!(
                    "Failed to fetch bilde metadata. HTTP Status: {}, response: {}",
                    status, error_message
                ),
            })
        }
    }
}
