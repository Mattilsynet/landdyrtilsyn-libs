use crate::error::Result;
use secrecy::SecretString;
#[derive(Debug, Clone)]
pub struct ClientConfiguration {
    pub client_id: SecretString,
    pub auth_url: String,
    pub client_secret: SecretString,
}
impl ClientConfiguration {
    pub async fn new(client_id: &str, auth_url: &str, client_secret: &str) -> ClientConfiguration {
        ClientConfiguration {
            client_id: SecretString::new(client_id.into()),
            auth_url: auth_url.to_string(),
            client_secret: SecretString::new(client_secret.into()),
        }
    }

    pub async fn build(api_key: &str) -> Result<ClientConfiguration> {
        let client_id = std::env::var(format!("{}_CLIENT_ID", api_key.to_uppercase()))
            .unwrap_or_else(|_| panic!("Expected env {}_CLIENT_ID", api_key.to_uppercase()));
        let auth_url = std::env::var(format!("{}_AUTH_URL", api_key.to_uppercase()))
            .unwrap_or_else(|_| panic!("Expected env {}_AUTH_URL", api_key.to_uppercase()));
        let client_secret = std::env::var(format!("{}_CLIENT_SECRET", api_key.to_uppercase()))
            .unwrap_or_else(|_| panic!("Expected env {}_CLIENT_SECRET", api_key.to_uppercase()));
        Ok(ClientConfiguration {
            client_id: SecretString::new(client_id.into_boxed_str()),
            auth_url,
            client_secret: SecretString::new(client_secret.into_boxed_str()),
        })
    }
}
