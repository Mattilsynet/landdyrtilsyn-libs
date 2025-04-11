use crate::auth::get_keycloak_token;
use crate::config::ClientConfiguration;
use crate::error::ApiError;
use reqwest::Response;
use reqwest_middleware::reqwest::Client;
use reqwest_middleware::{ClientBuilder as MiddlewareClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use secrecy::ExposeSecret;
use std::env;

#[derive(Clone)]
pub struct ApiClient {
    client: ClientWithMiddleware,
    token: String,
    base_url: String,
}

impl ApiClient {
    pub async fn new(base_url_prefix: &str, auth_config_prefix: &str) -> Self {
        let base_url = env::var(format!("{}_BASE_URL", base_url_prefix.to_uppercase()))
            .unwrap_or_else(|_| panic!("Expected env {}_BASE_URL", base_url_prefix.to_uppercase()));

        let client_config = ClientConfiguration::build(auth_config_prefix)
            .await
            .expect("Failed to build client configuration");
        let token = get_keycloak_token(
            client_config.client_id.expose_secret(),
            client_config.client_secret.expose_secret(),
            client_config.auth_url.as_str(),
        )
        .await
        .unwrap_or_else(|e| ApiError::TokenError(e.to_string()).to_string());

        let client = Client::new();

        ApiClient {
            client: ClientWithMiddleware::new(client, vec![]),
            token,
            base_url,
        }
    }

    pub fn get_client(&self) -> &ClientWithMiddleware {
        &self.client
    }

    pub fn get_token(&self) -> &str {
        &self.token
    }

    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }

    pub fn add_retry_policy(&mut self) -> &mut ApiClient {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        self.client = MiddlewareClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        self
    }

    pub async fn api_get(&self, url: &String) -> crate::error::Result<Response> {
        let response = self
            .get_client()
            .get(url)
            .header(
                reqwest::header::CONTENT_TYPE.to_string(),
                "application/json",
            )
            .bearer_auth(self.get_token())
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;
        Ok(response)
    }
}
