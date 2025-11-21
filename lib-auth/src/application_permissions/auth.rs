use crate::error::{Error, Result};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::time::{Duration, SystemTime};

#[derive(Clone, Debug)]
pub struct ApplicationAccessToken {
    pub token: SecretString,
    pub expires_at: SystemTime,
}

#[derive(Debug, Deserialize)]
struct ClientCredentialsTokenResponse {
    expires_in: u64,
    access_token: String,
}

#[derive(Clone, Debug)]
pub struct AuthConfig {
    tenant_id: String,
    client_id: SecretString,
    client_secret: SecretString,
}

impl AuthConfig {
    pub fn new(tenant_id: &str, client_id: SecretString, client_secret: SecretString) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            client_id,
            client_secret,
        }
    }

    pub fn from_env() -> Result<Self> {
        let tenant_id = std::env::var("AZURE_TENANT_ID")
            .map_err(|_| Error::MissingEnvVariable("AZURE_TENANT_ID is missing".to_string()))?;
        let client_id = std::env::var("AZURE_CLIENT_ID")
            .map_err(|_| Error::MissingEnvVariable("AZURE_CLIENT_ID is missing".to_string()))?;
        let client_secret = std::env::var("AZURE_CLIENT_SECRET")
            .map_err(|_| Error::MissingEnvVariable("AZURE_CLIENT_SECRET is missing".to_string()))?;
        Ok(Self {
            tenant_id,
            client_id: SecretString::from(client_id),
            client_secret: SecretString::from(client_secret),
        })
    }

    /// Lag OAuth2 endepunktet for vår tenant
    fn token_endpoint(&self) -> String {
        format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.tenant_id
        )
    }

    /// Fetch et application token for Microsoft Graph
    /// Bruker scope: https://graph.microsoft.com/.default
    pub async fn fetch_graph_token(&self) -> Result<ApplicationAccessToken> {
        self.fetch_token(&["https://graph.microsoft.com/.default"])
            .await
    }

    async fn fetch_token(&self, scopes: &[&str]) -> Result<ApplicationAccessToken> {
        let scope = scopes.join(" ");
        let form = [
            ("client_id", self.client_id.expose_secret()),
            ("client_secret", self.client_secret.expose_secret()),
            ("grant_type", "client_credentials"),
            ("scope", &scope),
        ];

        let client = reqwest::Client::new();
        let resp = client
            .post(self.token_endpoint())
            .form(&form)
            .send()
            .await
            .map_err(Error::ReqwestError)?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::InternalError(
                format!("Token request failed ({status}): {body}").into(),
            ));
        }

        let result: ClientCredentialsTokenResponse =
            resp.json().await.map_err(Error::ReqwestError)?;

        let token = SecretString::from(result.access_token);

        let expires_at =
            SystemTime::now() + Duration::from_secs(result.expires_in.saturating_sub(30)); // Trekker fra 30 sekunder som en buffer

        Ok(ApplicationAccessToken { token, expires_at })
    }
}
