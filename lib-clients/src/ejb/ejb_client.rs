use crate::ejb::response_tilfeller::ALLOWED_FIELDS_TILFELLER;
use crate::error::ApiError;
use crate::error::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use urlencoding::encode;

use crate::client::ApiClient;
use crate::ejb::response_begrensninger::{
    ALLOWED_FIELDS_BEGRENSNINGER, ApiResponseBegrensninger, Begrensning,
};
use crate::ejb::response_tilfeller::{ApiResponseTilfelle, Sykdomstilfelle};
use tracing::{debug, info};
use uuid::Uuid;

pub struct EjbClient {
    api_client: ApiClient,
}

impl EjbClient {
    #[tracing::instrument(name = "Creating RestEjbClient")]
    pub async fn new() -> Self {
        EjbClient {
            api_client: ApiClient::new("EJB", "KEYCLOAK_EJB").await,
        }
    }

    #[tracing::instrument(
        name = "Henter tilfeller for filter",
        skip(self),
        fields(
            request_id = %Uuid::new_v4(),
            limit = %limit,
        )
    )]
    pub async fn hent_tilfelle(
        &self,
        filter: HashMap<String, FilterCondition>,
        limit: u16,
    ) -> Result<Vec<Sykdomstilfelle>> {
        let filter = match validate_filter_map(filter, ALLOWED_FIELDS_TILFELLER) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };

        let url = format!("{}/v1/tilfeller", &self.api_client.get_base_url());
        let url = append_filter_to_url(&url, &filter, Some(limit)).map_err(|e| {
            ApiError::ValidationError(format!("Failed to append filter to URL: {e}"))
        })?;

        info!("Henter tilfelle fra url : {:?}", url);

        let response = self
            .api_client
            .get_client()
            .get(&url)
            .bearer_auth(self.api_client.get_token())
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;

        let r = response.bytes().await?.clone();
        let raw_json: serde_json::Value =
            serde_json::from_slice(&r).map_err(|e| ApiError::ClientError {
                resource: "serde_json".to_string(),
                error_message: e.to_string(),
            })?;
        println!(
            "Rå respons:\n{}",
            serde_json::to_string_pretty(&raw_json).unwrap_or_default()
        );

        let de = &mut serde_json::Deserializer::from_slice(&r);
        let result: std::result::Result<ApiResponseTilfelle, _> =
            serde_path_to_error::deserialize(de);
        match result {
            Ok(obj) => {
                println!("Deserialized: {:?}", obj.results.first());
                Ok(obj.results)
            }
            Err(e) => {
                // e.path() tells you exactly where the error occurred!
                println!("Failed at path: {}", e.path());
                println!("Serde error: {}", e.inner());
                Err(ApiError::ClientError {
                    resource: "hent_tilfelle".to_string(),
                    error_message: "Fant ikke tilfelle i response".to_string(),
                })
            }
        }
    }

    #[tracing::instrument(
        name = "Henter begrensinger for filter",
        skip(self),
        fields(
            request_id = %Uuid::new_v4(),
            limit = %limit,
            dato = ?dato,
        )
    )]
    pub async fn hent_begrensninger(
        &self,
        filter: HashMap<String, FilterCondition>,
        dato: Option<String>,
        limit: u16,
    ) -> Result<Vec<Begrensning>> {
        let filter = match validate_filter_map(filter, ALLOWED_FIELDS_BEGRENSNINGER) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };

        let url = format!("{}/v1/begrensninger", &self.api_client.get_base_url());
        let mut url = append_filter_to_url(&url, &filter, Some(limit)).map_err(|e| {
            ApiError::ValidationError(format!("Failed to append filter to URL: {e}"))
        })?;

        if let Some(dato_verdi) = dato {
            url.push_str(&format!("&dato={dato_verdi}"));
        }

        info!("Henter begrensinger fra url: {:?}", url);

        let response = self
            .api_client
            .get_client()
            .get(&url)
            .bearer_auth(self.api_client.get_token())
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;

        let r = response.bytes().await?.clone();
        let raw_json: serde_json::Value =
            serde_json::from_slice(&r).map_err(|e| ApiError::ClientError {
                resource: "serde_json".to_string(),
                error_message: e.to_string(),
            })?;
        debug!(
            "Rå respons:\n{}",
            serde_json::to_string_pretty(&raw_json).unwrap_or_default()
        );

        let de = &mut serde_json::Deserializer::from_slice(&r);
        let result: std::result::Result<ApiResponseBegrensninger, _> =
            serde_path_to_error::deserialize(de);
        match result {
            Ok(obj) => {
                debug!("Deserialized: {:?}", obj.results.first());
                Ok(obj.results)
            }
            Err(e) => {
                println!("Failed at path: {}", e.path());
                println!("Serde error: {}", e.inner());
                Err(ApiError::ClientError {
                    resource: "hent_tilfelle".to_string(),
                    error_message: "Fant ikke tilfelle i response".to_string(),
                })
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum FilterCondition {
    SingleOp { eq: Value },
    MultiOp(HashMap<String, Value>),
}

pub fn validate_filter_map(
    input: HashMap<String, FilterCondition>,
    allowed_fields: &[&str],
) -> Result<HashMap<String, FilterCondition>> {
    for key in input.keys() {
        if !allowed_fields.contains(&key.as_str()) {
            return Err(ApiError::ValidationError(format!(
                "Ugyldig filterfelt: {key}"
            )));
        }
    }
    Ok(input)
}

pub fn append_filter_to_url(
    base_url: &str,
    filters: &HashMap<String, FilterCondition>,
    limit: Option<u16>,
) -> Result<String> {
    let json_value = serde_json::to_value(filters)
        .map_err(|e| ApiError::ValidationError(format!("Failed to serialize filters: {e}")))?;
    let json_string = json_value.to_string();
    let encoded_filter = encode(&json_string);
    let mut url = format!("{base_url}?filter={encoded_filter}");
    if let Some(lim) = limit {
        url.push_str(&format!("&limit={lim}"));
    }
    Ok(url)
}
