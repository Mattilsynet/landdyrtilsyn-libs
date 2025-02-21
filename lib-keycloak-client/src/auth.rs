use reqwest::Client;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
struct KeycloakTokenResponse {
    access_token: String,
}

pub async fn get_keycloak_token(
    client_id: &str,
    client_secret: &str,
    auth_url: &str,
) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("grant_type", "client_credentials"),
        ("scope", "openid"),
    ];

    let res = client
        .post(auth_url)
        .form(&params)
        .send()
        .await?
        .json::<KeycloakTokenResponse>()
        .await?;

    Ok(res.access_token)
}
