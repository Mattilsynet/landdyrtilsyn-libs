use crate::client::ApiClient;
use crate::error::ApiError;
use crate::hrng::error::{HrngError, HrngResult};
use reqwest::header::ACCEPT;
use serde::de::DeserializeOwned;
use tracing::instrument;
use uuid::Uuid;

/// Client for HRNG's lookup API. Follows `links.lookup` from HRNG events,
/// authenticated with an Entra ID application token.
#[derive(Clone)]
pub struct HrngClient {
    api_client: ApiClient,
}

impl HrngClient {
    /// Reads `{PREFIX}_CLIENT_ID` (default prefix `HRNG`), the client id of the
    /// lookup API in Entra, and `{AUTH}_TENANT_ID`, `{AUTH}_CLIENT_ID` and
    /// `{AUTH}_CLIENT_SECRET` (default prefix `AZURE`). Panics if any of them is
    /// missing.
    #[instrument(name = "Creating HrngClient")]
    pub async fn new(prefix: Option<&str>, auth_config_prefix: Option<&str>) -> Self {
        let prefix = prefix.unwrap_or("HRNG");
        let auth = auth_config_prefix.unwrap_or("AZURE");

        HrngClient {
            api_client: ApiClient::new_entra(prefix, auth).await,
        }
    }

    /// Fetches and deserializes the resource behind `links.lookup` from an
    /// HRNG event. The token is sent to the url as given.
    #[instrument(
        name = "Henter fra HRNG lookup",
        skip(self, lookup_url),
        fields(request_id = %Uuid::new_v4())
    )]
    pub async fn hent<T: DeserializeOwned>(&self, lookup_url: &str) -> HrngResult<T> {
        let request = self
            .api_client
            .get_client()
            .get(lookup_url)
            .header(ACCEPT, "application/json")
            .bearer_auth(self.api_client.get_token().await);

        let response = self
            .api_client
            .send_request_with_refresh(request)
            .await
            .map_err(|e| match e {
                ApiError::AuthError { .. } => HrngError::Auth,
                other => HrngError::Request(other),
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(HrngError::Http { status });
        }

        let body = response
            .bytes()
            .await
            .map_err(|e| HrngError::Request(ApiError::RequestError(e)))?;

        parse_body(&body)
    }
}

fn parse_body<T: DeserializeOwned>(body: &[u8]) -> HrngResult<T> {
    let mut deserializer = serde_json::Deserializer::from_slice(body);
    serde_path_to_error::deserialize(&mut deserializer).map_err(|e| HrngError::Parse {
        path: e.path().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn client_and_hent_future_are_send() {
        fn assert_send_sync<T: Send + Sync>() {}
        fn assert_send<T: Send>(_: T) {}
        fn check(client: &HrngClient) {
            assert_send(client.hent::<serde_json::Value>(""));
        }

        assert_send_sync::<HrngClient>();
        let _ = check;
    }

    #[derive(Debug, Deserialize)]
    struct Aktivitet {
        id: u64,
        detaljer: Detaljer,
    }

    #[derive(Debug, Deserialize)]
    struct Detaljer {
        antall: u32,
    }

    #[test]
    fn parses_body_into_requested_type() {
        let aktivitet: Aktivitet = parse_body(br#"{"id":13338,"detaljer":{"antall":12}}"#).unwrap();

        assert_eq!(aktivitet.id, 13338);
        assert_eq!(aktivitet.detaljer.antall, 12);
    }

    #[test]
    fn parses_body_as_json_value() {
        let value: serde_json::Value = parse_body(br#"{"id":13338}"#).unwrap();

        assert_eq!(value["id"], 13338);
    }

    #[test]
    fn parse_error_reports_path_without_values() {
        let error =
            parse_body::<Aktivitet>(br#"{"id":13338,"detaljer":{"antall":"Ola Nordmann"}}"#)
                .unwrap_err();

        match &error {
            HrngError::Parse { path } => assert_eq!(path, "detaljer.antall"),
            other => panic!("unexpected error: {other:?}"),
        }
        assert!(!error.to_string().contains("Ola Nordmann"));
        assert!(!format!("{error:?}").contains("Ola Nordmann"));
    }
}
