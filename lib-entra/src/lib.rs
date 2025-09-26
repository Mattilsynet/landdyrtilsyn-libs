use reqwest::StatusCode;

use serde::{Deserialize, Serialize};
use serde_json;

use tracing::{self, instrument};

#[derive(Debug, thiserror::Error)]
pub enum EntraError {
    #[error("network error: {0}")]
    Network(String),
    #[error("unauthorized (token invalid or expired)")]
    Unauthorized,
    #[error("forbidden (insufficient permissions)")]
    Forbidden,
    #[error("unexpected status {status}: {body}")]
    UnexpectedResponse { status: StatusCode, body: String },
    #[error("deserialize error: {0}")]
    Deserialize(String),
    #[error("obo exchange failed: {0}")]
    Obo(String),
    #[error("missing env var: {0}")]
    MissingEnv(String),
}

pub type Result<T> = core::result::Result<T, EntraError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphUserMemberOf {
    pub id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GraphUser {
    pub id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub mail: Option<String>,
    #[serde(rename = "userPrincipalName")]
    pub user_principal_name: Option<String>,
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    pub surname: Option<String>,
    #[serde(rename = "jobTitle")]
    pub job_title: Option<String>,
    #[serde(rename = "employeeId")]
    pub employeeid: Option<String>,
    #[serde(rename = "memberOf")]
    pub groups: Option<Vec<GraphUserMemberOf>>,
}

#[derive(Deserialize)]
struct MemberOfPage {
    value: Vec<GraphUserMemberOf>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
}

async fn fetch_member_of(token: &str) -> Result<Vec<GraphUserMemberOf>> {
    let client = reqwest::Client::new();
    let mut url = "https://graph.microsoft.com/v1.0/me/memberOf?$select=id".to_string();
    let mut all: Vec<GraphUserMemberOf> = Vec::new();

    loop {
        let resp = client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| EntraError::Network(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| EntraError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(EntraError::UnexpectedResponse { status, body });
        }

        let page: MemberOfPage =
            serde_json::from_str(&body).map_err(|e| EntraError::Deserialize(e.to_string()))?;

        all.extend(page.value.into_iter());

        if let Some(next) = page.next_link {
            url = next;
        } else {
            break;
        }
    }

    Ok(all)
}

#[instrument(name = "Henter fra graphAPIet", skip(token), level = "info")]
async fn get_user_profile(token: &str, include_groups: bool) -> Result<GraphUser> {
    tracing::info!("Henter brukerinformasjon fra graphAPIet");

    let client = reqwest::Client::new();

    // Only user profile now
    let url = "https://graph.microsoft.com/v1.0/me?$select=id,displayName,mail,userPrincipalName,givenName,surname,jobTitle,employeeId";

    let resp = client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Klarte ikke hente data fra graph API: {}", e.to_string());
            EntraError::Network(e.to_string())
        })?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| {
        tracing::error!("Klarte ikke lese body: {}", e.to_string());
        EntraError::Network(e.to_string())
    })?;

    let mut user = match status {
        StatusCode::OK => serde_json::from_str::<GraphUser>(&body)
            .map_err(|e| EntraError::Deserialize(e.to_string()))?,
        StatusCode::UNAUTHORIZED => return Err(EntraError::Unauthorized),
        StatusCode::FORBIDDEN => return Err(EntraError::Forbidden),
        other => {
            return Err(EntraError::UnexpectedResponse {
                status: other,
                body,
            });
        }
    };

    if include_groups {
        tracing::info!("Inkluderer grupper (separate kall + paging)");
        match fetch_member_of(token).await {
            Ok(groups) => {
                user.groups = Some(groups);
            }
            Err(e) => {
                tracing::warn!("Klarte ikke hente grupper: {e}");
            }
        }
    }

    Ok(user)
}

/// Configuration needed for On-Behalf-Of token exchange.
#[derive(Debug, Clone)]
pub struct OboConfig {
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
}

impl OboConfig {
    pub fn from_env() -> Result<Self> {
        let tenant_id = std::env::var("AZURE_TENANT_ID")
            .map_err(|_| EntraError::MissingEnv("AZURE_TENANT_ID".into()))?;
        let client_id = std::env::var("AZURE_CLIENT_ID")
            .map_err(|_| EntraError::MissingEnv("AZURE_CLIENT_ID".into()))?;
        let client_secret = std::env::var("AZURE_CLIENT_SECRET")
            .map_err(|_| EntraError::MissingEnv("AZURE_CLIENT_SECRET".into()))?;
        Ok(Self {
            tenant_id,
            client_id,
            client_secret,
        })
    }
}

#[derive(serde::Deserialize)]
struct OboTokenResponse {
    access_token: String,
}

// EntraID on-behalf-of exchange: Bytter frontend user token (hvor aud er API scope) til graphAPI access token.
#[instrument(
    name = "OBO exchange av brukertoken mot graphAPI token",
    skip(user_token),
    level = "info"
)]
pub async fn exchange_for_graph_token_obo(user_token: &str) -> Result<String> {
    let cfg = OboConfig::from_env()?;

    tracing::info!(
        "Starter flyt for on-behalf-of exchange av frontend access token mot graphAPI access token"
    );
    let scope_param = "https://graph.microsoft.com/User.Read";
    let url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        cfg.tenant_id
    );
    let params = [
        ("client_id", cfg.client_id.as_str()),
        ("client_secret", cfg.client_secret.as_str()),
        ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
        ("requested_token_use", "on_behalf_of"),
        ("assertion", user_token),
        ("scope", scope_param),
    ];
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .form(&params)
        .send()
        .await
        .map_err(|e| EntraError::Network(e.to_string()))?;
    let status = resp.status();
    let body = resp.text().await.map_err(|e| {
        tracing::error!("Klarte ikke lese response body: {}", e);
        EntraError::Network(e.to_string())
    })?;
    if !status.is_success() {
        tracing::error!("On-behalf-off exchange feilet: status {}: {}", status, body);
        return Err(EntraError::Obo(format!("status {status}: {body}")));
    }
    let parsed: OboTokenResponse = serde_json::from_str(&body).map_err(|e| {
        tracing::error!("Klarte ikke parse on-behalf-of token response");
        EntraError::Deserialize(e.to_string())
    })?;
    Ok(parsed.access_token)
}

// Fra frontend access token, utfør o-b-o og hent data fra graphAPI
pub async fn get_user(user_token: &str, include_groups: bool) -> Result<GraphUser> {
    let graph_token = exchange_for_graph_token_obo(user_token).await?;
    get_user_profile(&graph_token, include_groups).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn invalid_token_results_in_error() {
        let token = "invalid.token";
        let result = get_user_profile(token, false).await;
        assert!(result.is_err());
    }
}
