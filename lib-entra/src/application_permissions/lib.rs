use reqwest::{StatusCode, Url};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

use crate::{
    error::EntraError,
    types::{GRAPH_USER_SELECT_FIELDS, GraphUser, GraphUserSearchResponse, Result},
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
        .header("ConsistencyLevel", "eventual")
        .send()
        .await
        .map_err(|e| EntraError::Network(e.to_string()))?;

    let status = response.status();

    let body = response
        .text()
        .await
        .map_err(|e| EntraError::Network(e.to_string()))?;

    let graph_response = match status {
        StatusCode::OK => serde_json::from_str::<GraphUserSearchResponse>(&body)
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
    let user = graph_response.value.first();

    match user {
        Some(user) => Ok(user.to_owned()),
        None => Err(EntraError::NoSuchEmployeeId(employee_id.to_string())),
    }
}

pub async fn get_users_by_cost_center(
    access_token: SecretString,
    cost_center: &str,
) -> Result<Vec<GraphUser>> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| EntraError::Network(e.to_string()))?;
    let mut url = cost_center_url(cost_center)?;
    let mut users = Vec::new();

    loop {
        let response = client
            .get(url)
            .bearer_auth(access_token.expose_secret())
            .header("ConsistencyLevel", "eventual")
            .send()
            .await
            .map_err(|e| EntraError::Network(e.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| EntraError::Network(e.to_string()))?;
        let page = users_page(status, &body)?;
        users.extend(page.value);

        match page.next_link {
            Some(next) => url = graph_users_url(&next)?,
            None => return Ok(users),
        }
    }
}

fn cost_center_url(cost_center: &str) -> Result<Url> {
    let mut url = graph_users_url("https://graph.microsoft.com/v1.0/users")?;
    let filter = format!(
        "onPremisesExtensionAttributes/extensionAttribute6 eq '{}'",
        cost_center.replace('\'', "''")
    );
    url.query_pairs_mut().extend_pairs([
        ("$filter", filter.as_str()),
        (
            "$select",
            "id,displayName,employeeId,onPremisesExtensionAttributes",
        ),
        ("$count", "true"),
    ]);
    Ok(url)
}

fn graph_users_url(value: &str) -> Result<Url> {
    let url =
        Url::parse(value).map_err(|_| EntraError::Deserialize("invalid Graph users URL".into()))?;
    if url.scheme() != "https"
        || url.host_str() != Some("graph.microsoft.com")
        || url.port_or_known_default() != Some(443)
        || url.path() != "/v1.0/users"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(EntraError::Deserialize("invalid Graph users URL".into()));
    }
    Ok(url)
}

#[derive(Deserialize)]
struct GraphUsersPage {
    value: Vec<GraphUser>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
}

fn users_page(status: StatusCode, body: &str) -> Result<GraphUsersPage> {
    match status {
        StatusCode::OK => {
            serde_json::from_str(body).map_err(|e| EntraError::Deserialize(e.to_string()))
        }
        StatusCode::UNAUTHORIZED => Err(EntraError::Unauthorized),
        StatusCode::FORBIDDEN => Err(EntraError::Forbidden),
        other => Err(EntraError::UnexpectedResponse {
            status: other,
            body: body.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_center_query_escapes_odata_and_url_characters() {
        let url = cost_center_url("123' or id ne 'x&$select=mail+#?").unwrap();
        let query: Vec<_> = url.query_pairs().collect();
        assert_eq!(
            query,
            vec![
                (
                    "$filter".into(),
                    "onPremisesExtensionAttributes/extensionAttribute6 eq '123'' or id ne ''x&$select=mail+#?'".into()
                ),
                (
                    "$select".into(),
                    "id,displayName,employeeId,onPremisesExtensionAttributes".into()
                ),
                ("$count".into(), "true".into())
            ]
        );
        assert!(url.fragment().is_none());
    }

    #[test]
    fn pagination_preserves_next_link_query_and_accepts_terminal_page() {
        let next = "https://graph.microsoft.com/v1.0/users?$skiptoken=a%2Bb%2F%3D&$select=id%2CdisplayName&$count=true";
        let page = users_page(
            StatusCode::OK,
            &serde_json::json!({
                "value": [{"id": "first"}],
                "@odata.nextLink": next
            })
            .to_string(),
        )
        .unwrap();
        assert_eq!(page.value[0].id.as_deref(), Some("first"));
        let url = graph_users_url(page.next_link.as_deref().unwrap()).unwrap();
        assert_eq!(url.as_str(), next);
        assert_eq!(url.query_pairs().next().unwrap().1, "a+b/=");

        for body in [r#"{"value":[]}"#, r#"{"value":[],"@odata.nextLink":null}"#] {
            let page = users_page(StatusCode::OK, body).unwrap();
            assert!(page.value.is_empty());
            assert!(page.next_link.is_none());
        }
    }

    #[test]
    fn rejects_next_links_outside_graph_users_endpoint() {
        for url in [
            "http://graph.microsoft.com/v1.0/users",
            "https://example.com/v1.0/users",
            "https://graph.microsoft.com.example.com/v1.0/users",
            "https://graph.microsoft.com:444/v1.0/users",
            "https://graph.microsoft.com/beta/users",
            "https://graph.microsoft.com/v1.0/groups",
            "https://graph.microsoft.com/v1.0/users/other",
            "https://graph.microsoft.com/v1.0/users%2Fother",
            "https://user@graph.microsoft.com/v1.0/users",
            "https://user:password@graph.microsoft.com/v1.0/users",
            "https://graph.microsoft.com/v1.0/users#fragment",
            "/v1.0/users?$skiptoken=abc",
            "not a URL",
        ] {
            assert!(matches!(
                graph_users_url(url),
                Err(EntraError::Deserialize(_))
            ));
        }
        assert!(graph_users_url("https://graph.microsoft.com:443/v1.0/users").is_ok());
    }

    #[test]
    fn page_errors_are_not_treated_as_empty_results() {
        assert!(matches!(
            users_page(StatusCode::UNAUTHORIZED, ""),
            Err(EntraError::Unauthorized)
        ));
        assert!(matches!(
            users_page(StatusCode::FORBIDDEN, ""),
            Err(EntraError::Forbidden)
        ));
        for status in [
            StatusCode::FOUND,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::INTERNAL_SERVER_ERROR,
        ] {
            assert!(matches!(
                users_page(status, "error body"),
                Err(EntraError::UnexpectedResponse { status: actual, body })
                    if actual == status && body == "error body"
            ));
        }
        for body in [
            "invalid JSON",
            "{}",
            r#"{"value":null}"#,
            r#"{"value":[],"@odata.nextLink":42}"#,
        ] {
            assert!(matches!(
                users_page(StatusCode::OK, body),
                Err(EntraError::Deserialize(_))
            ));
        }
    }
}
