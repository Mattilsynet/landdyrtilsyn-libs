use crate::arkiv::response::{ArkivClientJournalpost, ArkivClientSak};
use crate::error::{Result};
use crate::error::ApiError;
use crate::client::ApiClient;
use serde::Deserialize;
use tracing::{error, info};
use uuid::{Uuid};

pub struct ArkivClient {
    api_client: ApiClient,
}

#[derive(Deserialize)]
struct JournalposterEmbedded {
    #[serde(rename = "journalpostList")]
    journalpost_list: Vec<ArkivClientJournalpost>,
}

#[derive(Deserialize)]
struct JournalposterResponse {
    #[serde(rename = "_embedded")]
    embedded: JournalposterEmbedded,
}

impl ArkivClient {
    pub async fn new(base_url_prefix: &str, auth_config_prefix: &str) -> Self {
        ArkivClient {
            api_client: ApiClient::new(base_url_prefix, auth_config_prefix).await,
        }
    }

    #[tracing::instrument(
    name = "Henter arkiv sak",
    skip(self),
    fields(
        request_id = %Uuid::new_v4(),
        noarkaar = %noarkaar,
        noarksaksnummer = %noarksaksnummer
    )
    )]
    pub async fn get_arkiv_sak(
        &self,
        noarkaar: &str,
        noarksaksnummer: &str,
    ) -> Result<ArkivClientSak> {
        let url = format!(
            "{}arkiv/saker/{}/{}",
            self.api_client.get_base_url(),
            noarkaar,
            noarksaksnummer
        );

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let text = response.text().await.unwrap();
            let sak: ArkivClientSak =
                serde_json::from_str(&text).map_err(|e|ApiError::ParseError(e.to_string()))?;
            info!("Hentet sak {:?} fra arkiv api.", sak);
            Ok(sak)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente sak {}/{}, error message {}",
                noarkaar, noarksaksnummer, error_message
            );

            Err(ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!(
                    "Failed to get arkiv sak, HTTP Status: {}, response {}",
                    status, error_message
                ),
            })
        }
    }

    #[tracing::instrument(
        name = "Henter arkiv sak sine journalposter",
        skip(self),
        fields(
        request_id = %Uuid::new_v4(),
        noarkaar = %noarkaar,
        noarksaksnummer = %noarksaksnummer
        )
    )]
    pub async fn get_arkiv_sak_journalposter(
        &self,
        noarkaar: &str,
        noarksaksnummer: &str,
    ) -> Result<Vec<ArkivClientJournalpost>> {
        let url = format!(
            "{}arkiv/saker/{}/{}/journalposter",
            self.api_client.get_base_url(),
            noarkaar,
            noarksaksnummer
        );

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let text = response.text().await?;
            let journalposter_response: JournalposterResponse = serde_json::from_str(&text)
                .map_err(|e| ApiError::ParseError(e.to_string()))?;
            let journalposter = journalposter_response.embedded.journalpost_list;
            info!(
                "Hentet journalposter på sak {:?} fra arkiv api.",
                journalposter
            );
            Ok(journalposter)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente journalposter paa sak {}/{}, error message {}",
                noarkaar, noarksaksnummer, error_message
            );

            Err(ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!(
                    "Failed to get arkiv sak, HTTP Status: {}, response {}",
                    status, error_message
                ),
            })
        }
    }
}
