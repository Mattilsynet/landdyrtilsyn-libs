use crate::client::ApiClient;
use crate::error::ApiError;
use crate::virksomhet::response::{Underenhet, Virksomhet};

pub struct VirksomhetClient {
    api_client: ApiClient,
}

impl VirksomhetClient {
    pub async fn new() -> Self {
        VirksomhetClient {
            api_client: ApiClient::new("VIRKSOMHET_API", "KEYCLOAK").await,
        }
    }

    pub async fn get_virksomhet(&self, orgnr: String) -> Result<Virksomhet, ApiError> {
        let url = format!(
            "{}/virksomheter/orgnummer/{}",
            &self.api_client.get_base_url(),
            orgnr,
        );

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let virksomhet: Virksomhet =
                response.json().await.map_err(|e| ApiError::ClientError {
                    resource: "reqwest".to_string(),
                    error_message: e.to_string(),
                })?;
            Ok(virksomhet)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ApiError::ClientError {
                resource: "virksomhet-api".to_string(),
                error_message: format!(
                    "Failed to fetch virksomhet. HTTP Status: {}, response: {}",
                    status, error_message
                ),
            })
        }
    }

    pub async fn hent_underenheter_paa_virksomhet(
        &self,
        orgnr: String,
    ) -> Result<Vec<Underenhet>, ApiError> {
        let url = format!(
            "{}/virksomheter/{}/underenheter",
            &self.api_client.get_base_url(),
            orgnr,
        );

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let underenheter: Vec<Underenhet> =
                response.json().await.map_err(|e| ApiError::ClientError {
                    resource: "reqwest".to_string(),
                    error_message: e.to_string(),
                })?;
            Ok(underenheter)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ApiError::ClientError {
                resource: "virksomhet-api".to_string(),
                error_message: format!(
                    "Failed to fetch underenheter. HTTP Status: {}, response: {}",
                    status, error_message
                ),
            })
        }
    }
}
