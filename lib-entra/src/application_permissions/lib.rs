use reqwest::StatusCode;
use secrecy::{ExposeSecret, SecretString};

use crate::{
    error::EntraError,
    types::{GRAPH_USER_SELECT_FIELDS, GraphUser, Result},
};

pub async fn get_user_from_employee_id(
    access_token: SecretString,
    employee_id: &str,
) -> Result<GraphUser> {
    let client = reqwest::Client::new();

    let request_url = format!(
        "https://graph.microsoft.com/v1.0/users?$count=true&$search=\"employeeid:{employee_id}\"&$select={GRAPH_USER_SELECT_FIELDS}"
    );

    let response = client
        .get(request_url)
        .bearer_auth(access_token.expose_secret())
        .send()
        .await
        .map_err(|e| EntraError::Network(e.to_string()))?;

    let status = response.status();

    let body = response
        .text()
        .await
        .map_err(|e| EntraError::Network(e.to_string()))?;

    let user = match status {
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

    Ok(user)
}
