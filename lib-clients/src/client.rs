use crate::auth::get_keycloak_token;
use crate::config::ClientConfiguration;
use crate::error::ApiError;
use lib_auth::application_permissions::auth::AuthConfig;
use reqwest::Response;
use reqwest_middleware::reqwest::Client;
use reqwest_middleware::{
    ClientBuilder as MiddlewareClientBuilder, ClientWithMiddleware, RequestBuilder,
};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use secrecy::{ExposeSecret, SecretString};
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tracing::error;

const KEYCLOAK_TOKEN_LIFETIME: Duration = Duration::from_secs(300);

pub struct Token {
    value: String,
    expires_at: Instant,
}

enum Credentials {
    Keycloak {
        client_id: String,
        client_secret: String,
        auth_url: String,
    },
    Entra {
        auth_config: AuthConfig,
        scope: String,
    },
}

pub struct TokenProvider {
    current_token: RwLock<Token>,
    refresh_lock: Mutex<()>,
    credentials: Credentials,
}
impl TokenProvider {
    async fn new(credentials: Credentials) -> Self {
        let (value, expires_at) = Self::fetch_new_token(&credentials).await;
        Self {
            current_token: RwLock::new(Token { value, expires_at }),
            refresh_lock: Mutex::new(()),
            credentials,
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

        let (value, expires_at) = Self::fetch_new_token(&self.credentials).await;
        {
            let mut token = self.current_token.write().await;
            token.value = value.clone();
            token.expires_at = expires_at;
        }
        value
    }

    async fn fetch_new_token(credentials: &Credentials) -> (String, Instant) {
        match credentials {
            Credentials::Keycloak {
                client_id,
                client_secret,
                auth_url,
            } => {
                let value = get_keycloak_token(client_id, client_secret, auth_url)
                    .await
                    .unwrap_or_else(|e| ApiError::TokenError(e.to_string()).to_string());
                (value, Instant::now() + KEYCLOAK_TOKEN_LIFETIME)
            }
            Credentials::Entra { auth_config, scope } => {
                match auth_config.fetch_token(&[scope.as_str()]).await {
                    Ok(token) => (
                        token.token.expose_secret().to_string(),
                        instant_from(token.expires_at),
                    ),
                    // Expires immediately, so the next call fetches a new token.
                    Err(e) => {
                        error!(error = %e, "Failed to fetch Entra token");
                        (String::new(), Instant::now())
                    }
                }
            }
        }
    }
}

/// lib-auth reports expiry as wall-clock time; the cache compares monotonic time.
fn instant_from(expires_at: SystemTime) -> Instant {
    let remaining = expires_at
        .duration_since(SystemTime::now())
        .unwrap_or(Duration::ZERO);
    Instant::now() + remaining
}

#[derive(Clone)]
pub struct ApiClient {
    client: ClientWithMiddleware,
    token_provider: Arc<TokenProvider>,
    base_url: String,
}

impl ApiClient {
    // Keycloak
    pub async fn new(base_url_prefix: &str, auth_config_prefix: &str) -> Self {
        let base_url = env::var(format!("{}_BASE_URL", base_url_prefix.to_uppercase()))
            .unwrap_or_else(|_| panic!("Expected env {}_BASE_URL", base_url_prefix.to_uppercase()));

        let client_config = ClientConfiguration::build(auth_config_prefix)
            .await
            .expect("Failed to build client configuration");

        let token_provider = Arc::new(
            TokenProvider::new(Credentials::Keycloak {
                client_id: client_config.client_id.expose_secret().to_string(),
                client_secret: client_config.client_secret.expose_secret().to_string(),
                auth_url: client_config.auth_url.clone(),
            })
            .await,
        );

        let client = Client::new();

        ApiClient {
            client: ClientWithMiddleware::new(client, vec![]),
            token_provider,
            base_url,
        }
    }

    // Entra
    pub async fn new_entra(prefix: &str, auth_config_prefix: &str) -> Self {
        let prefix = prefix.to_uppercase();
        let auth = auth_config_prefix.to_uppercase();

        let scope = format!("{}/.default", required_env(&format!("{prefix}_CLIENT_ID")));
        let auth_config = AuthConfig::new(
            &required_env(&format!("{auth}_TENANT_ID")),
            SecretString::from(required_env(&format!("{auth}_CLIENT_ID"))),
            SecretString::from(required_env(&format!("{auth}_CLIENT_SECRET"))),
        );

        let token_provider =
            Arc::new(TokenProvider::new(Credentials::Entra { auth_config, scope }).await);

        ApiClient {
            client: ClientWithMiddleware::new(Client::new(), vec![]),
            token_provider,
            base_url: String::new(),
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

    pub async fn refresh_token(&self) -> String {
        let token_provider = &self.token_provider;
        let (value, expires_at) = TokenProvider::fetch_new_token(&token_provider.credentials).await;
        {
            let mut token = token_provider.current_token.write().await;
            token.value = value.clone();
            token.expires_at = expires_at;
        }
        value
    }

    pub async fn send_request_with_refresh(
        &self,
        mut request: RequestBuilder,
    ) -> std::result::Result<reqwest::Response, ApiError> {
        let mut response =
            request
                .try_clone()
                .unwrap()
                .send()
                .await
                .map_err(|e| ApiError::ClientError {
                    resource: "HTTP Request".to_string(),
                    error_message: format!("Failed to send request: {}", e),
                })?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED
            || response.status() == reqwest::StatusCode::FORBIDDEN
        {
            let new_token = self.refresh_token().await;
            request = request.bearer_auth(new_token);
            response = request.send().await.map_err(|e| ApiError::ClientError {
                resource: "HTTP Request".to_string(),
                error_message: format!("Failed to send request after token refresh: {}", e),
            })?;
            if response.status() == reqwest::StatusCode::UNAUTHORIZED
                || response.status() == reqwest::StatusCode::FORBIDDEN
            {
                return Err(ApiError::AuthError {
                    error_message: "Authentication failed or token expired after refresh. Please check credentials.".to_string(),
                });
            }
        }
        Ok(response)
    }
}

fn required_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("Expected env {name}"))
}
