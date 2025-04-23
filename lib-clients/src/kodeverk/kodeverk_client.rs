use crate::client::ApiClient;
use crate::error::ApiError;
use crate::error::Result;
use crate::kodeverk::response::KodeverkResponse;
use reqwest_middleware::reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use tracing::instrument;
use uuid::Uuid;

pub struct KodeverkClient {
    api_client: ApiClient,
}

impl KodeverkClient {
    #[instrument(name = "Creating KodeverkClient")]
    pub async fn new(base_url_prefix: &str, auth_config_prefix: &str) -> Self {
        let client = ApiClient::new(base_url_prefix, auth_config_prefix)
            .await
            .clone();

        KodeverkClient { api_client: client }
    }

    #[instrument(
        name = "Fetching related kodeverk",
        skip(self),
        fields(
            request_id = %Uuid::new_v4(),
            realtion_name = %realtion_name,
            kodetype = %kodetype,
            kodenavn = %kodenavn
        )
    )]
    pub async fn get_relatert_kodeverk(
        &self,
        realtion_name: &str,
        kodetype: &str,
        kodenavn: &str,
    ) -> Result<String> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let url = format!(
            "{}/kodeverk/code/related/{}/{}/{}",
            self.api_client.get_base_url(),
            realtion_name,
            kodetype,
            kodenavn
        );
        println!("url : {}", url);

        let client = self.api_client.get_client();
        let token = self.api_client.get_token();

        let response = client
            .get(&url)
            .bearer_auth(token)
            .headers(headers)
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| ApiError::ClientError {
            resource: "reqwest".to_string(),
            error_message: e.to_string(),
        })?;
        if !status.is_success() {
            return Err(ApiError::ClientError {
                resource: "org_enhet".to_string(),
                error_message: format!(
                    "Failed to get kodeverk relation, HTTP Status: {}, response {}",
                    status, response_text
                ),
            });
        }
        println!("response_text : {}", response_text);

        let kodeverk_response: KodeverkResponse = serde_json::from_str(&response_text)
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        println!(
            "kodeverk_response : {:?}",
            kodeverk_response._embedded.related_code_list
        );

        let related_code_list_string =
            serde_json::to_string_pretty(&kodeverk_response).map_err(|e| {
                ApiError::ClientError {
                    resource: "serde".to_string(),
                    error_message: e.to_string(),
                }
            })?;

        Ok(related_code_list_string)
    }
}
