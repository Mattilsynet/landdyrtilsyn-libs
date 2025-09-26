use crate::auth::get_keycloak_token;
use crate::config::ClientConfiguration;
use crate::error::ApiError;
use reqwest::Response;
use reqwest_middleware::reqwest::Client;
use reqwest_middleware::{ClientBuilder as MiddlewareClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use secrecy::ExposeSecret;
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

pub struct Token {
    value: String,
    expires_at: Instant,
}
pub struct TokenProvider {
    current_token: RwLock<Token>,
    refresh_lock: Mutex<()>,
    client_id: String,
    client_secret: String,
    auth_url: String,
}
impl TokenProvider {
    async fn new(client_id: String, client_secret: String, auth_url: String) -> Self {
        let (value, expires_at) =
            Self::fetch_new_token(&client_id, &client_secret, &auth_url).await;
        Self {
            current_token: RwLock::new(Token { value, expires_at }),
            refresh_lock: Mutex::new(()),
            client_id,
            client_secret,
            auth_url,
        }
    }

    async fn get(&self) -> String {
        {
            let token = self.current_token.read().await;
            if token.expires_at > Instant::now() {
                return token.value.clone();
            }
        }

        let _guard = self.refresh_lock.lock().await;

        {
            let token = self.current_token.read().await;
            if token.expires_at > Instant::now() {
                return token.value.clone();
            }
        }

        let (value, expires_at) =
            Self::fetch_new_token(&self.client_id, &self.client_secret, &self.auth_url).await;
        {
            let mut token = self.current_token.write().await;
            token.value = value.clone();
            token.expires_at = expires_at;
        }
        value
    }

    async fn fetch_new_token(
        client_id: &str,
        client_secret: &str,
        auth_url: &str,
    ) -> (String, Instant) {
        let value = get_keycloak_token(client_id, client_secret, auth_url)
            .await
            .unwrap_or_else(|e| ApiError::TokenError(e.to_string()).to_string());

        let expires_at = Instant::now() + Duration::from_secs(300);
        (value, expires_at)
    }
}

#[derive(Clone)]
pub struct ApiClient {
    client: ClientWithMiddleware,
    token_provider: Arc<TokenProvider>,
    base_url: String,
}

impl ApiClient {
    pub async fn new(base_url_prefix: &str, auth_config_prefix: &str) -> Self {
        let base_url = env::var(format!("{}_BASE_URL", base_url_prefix.to_uppercase()))
            .unwrap_or_else(|_| panic!("Expected env {}_BASE_URL", base_url_prefix.to_uppercase()));

        let client_config = ClientConfiguration::build(auth_config_prefix)
            .await
            .expect("Failed to build client configuration");

        let token_provider = Arc::new(
            TokenProvider::new(
                client_config.client_id.expose_secret().to_string(),
                client_config.client_secret.expose_secret().to_string(),
                client_config.auth_url.clone(),
            )
            .await,
        );

        let client = Client::new();

        ApiClient {
            client: ClientWithMiddleware::new(client, vec![]),
            token_provider,
            base_url,
        }
    }

    pub fn get_client(&self) -> &ClientWithMiddleware {
        &self.client
    }

    pub async fn get_token(&self) -> String {
        self.token_provider.get().await
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
        let token = self.token_provider.get().await;
        let response = self
            .get_client()
            .get(url)
            .header(
                reqwest::header::CONTENT_TYPE.to_string(),
                "application/json",
            )
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;
        Ok(response)
    }
}
