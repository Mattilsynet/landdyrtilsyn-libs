use crate::client::ApiClient;
use crate::error::ApiError;
use crate::tilsynskvittering::response::TilsynsobjektKvittering;
use reqwest::header::{CONTENT_TYPE, HeaderMap};
use serde::Deserialize;
use tracing::info;

pub struct TilsynskvitteringClient {
    api_client: ApiClient,
}

#[derive(Deserialize)]
struct TidligereTilsynEmbedded {
    #[serde(rename = "tilsynsobjektKvitteringDTOList")]
    tidligere_tilsyn_list: Vec<TilsynsobjektKvittering>,
}

#[derive(Deserialize)]
struct TidligereTilsynResponse {
    #[serde(rename = "_embedded")]
    embedded: TidligereTilsynEmbedded,
}

impl TilsynskvitteringClient {
    pub async fn new() -> Self {
        TilsynskvitteringClient {
            api_client: ApiClient::new("TILSYNSKVITTERING_API", "KEYCLOAK").await,
        }
    }

    pub async fn hent_info_tildligere_tilsyn(
        &self,
        tilsynsobjekt_ids: Vec<String>,
    ) -> Result<Vec<TilsynsobjektKvittering>, ApiError> {
        let url = format!(
            "{}/v1/tilsynskvitteringer/tilsynsobjekter/info-tidligere-tilsyn",
            &self.api_client.get_base_url(),
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        let response = self
            .api_client
            .get_client()
            .post(url)
            .bearer_auth(self.api_client.get_token())
            .headers(headers)
            .json(&tilsynsobjekt_ids)
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;
        info!("Response: {:?}", response);

        if response.status().is_success() {
            let text = response.text().await?;
            let tidligere_tilsyn_response: TidligereTilsynResponse =
                serde_json::from_str(&text).map_err(|e| ApiError::ParseError(e.to_string()))?;
            let tidligere_tilsyn = tidligere_tilsyn_response.embedded.tidligere_tilsyn_list;
            info!(
                "Hentet tidligere tilsyn på tilsynobjekt(r) {tilsynsobjekt_ids:?} fra tilsynskvittering-api."
            );
            Ok(tidligere_tilsyn)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ApiError::ClientError {
                resource: "tilsynskvittering-api".to_string(),
                error_message: format!(
                    "Failed to fetch info. HTTP Status: {status}, response: {error_message}"
                ),
            })
        }
    }
}
