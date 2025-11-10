use reqwest::{StatusCode, header};

use serde::Deserialize;
use serde_json;

use crate::{
    error::EntraError,
    types::{GraphUser, GraphUserMemberOf, OboConfig, Result},
};
use base64::{Engine as _, engine::general_purpose};
use tracing::{self, instrument};

/// (Krever *delegated permisson*)
///
/// get_user(): Tar brukerens accessToken som hen har fått ved pålogging i frontend og gjør en on-behalf-of-exchange for å hente data om brukeren.
/// ### Eksempel
/// ``` let user: EntraUser = get_user(user_token:"2dfsalfga3hafks", include_groups:true); ```
///
pub async fn get_user(user_token: &str, include_groups: bool) -> Result<GraphUser> {
    let graph_token = exchange_for_graph_token_obo(user_token).await?;
    get_user_profile(&graph_token, include_groups).await
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
    let url: &'static str = "https://graph.microsoft.com/v1.0/me?$select=id,displayName,mail,userPrincipalName,givenName,surname,jobTitle,employeeId";
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
        match fetch_member_of(token).await {
            Ok(groups) => {
                user.groups = Some(groups);
            }
            Err(e) => {
                tracing::warn!("Klarte ikke hente grupper: {e}");
            }
        }
    }
    let photo_base64 = match client
        .get("https://graph.microsoft.com/v1.0/me/photo/$value")
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(resp) => {
            // Own the content type string before consuming resp with bytes()
            let content_type = resp
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "image/jpeg".to_string()); // Fallback

            match resp.bytes().await {
                Ok(bytes) => {
                    let b64 = general_purpose::STANDARD.encode(bytes);
                    Some(format!("data:{};base64,{}", content_type, b64))
                }
                Err(e) => {
                    tracing::warn!("Klarte ikke lese bilde bytes: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!("Klarte ikke hente bilde fra graph API: {}", e);
            None
        }
    };
    user.photo = photo_base64;

    Ok(user)
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
async fn exchange_for_graph_token_obo(user_token: &str) -> Result<String> {
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
