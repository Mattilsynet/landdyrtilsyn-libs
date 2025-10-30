use crate::koordinat::response::{AddressResult, GeonorgeResponse};
use crate::koordinat::{GeonorgeError, Result};
use reqwest_middleware::reqwest::Client;
use reqwest_middleware::{ClientBuilder as MiddlewareClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use tracing::{debug, error, info};

const GEONORGE_API_URL: &str = "https://ws.geonorge.no/adresser/v1";

fn normalize_house_letter(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let mut in_digits = false;

    while i < chars.len() {
        let c = chars[i];

        if c.is_ascii_digit() {
            in_digits = true;
            out.push(c);
            i += 1;
            continue;
        }

        if c.is_whitespace() && in_digits {
            // Peek ahead over whitespace; if the next non-space is alphabetic,
            // drop the spaces (compact like "12 b" -> "12b"). Otherwise, keep them.
            let mut k = i;
            while k < chars.len() && chars[k].is_whitespace() {
                k += 1;
            }
            if k < chars.len() && chars[k].is_alphabetic() {
                // Skip all whitespace; next loop iteration will push the letter.
                i = k;
                continue;
            } else {
                // Not followed by a letter; keep current whitespace.
                out.push(c);
                in_digits = false;
                i += 1;
                continue;
            }
        }

        out.push(c);
        in_digits = false;
        i += 1;
    }

    out
}

#[derive(Clone)]
pub struct KoordinatClient {
    client: ClientWithMiddleware,
    base_url: String,
}

impl KoordinatClient {
    pub fn new() -> Self {
        Self::with_base_url(GEONORGE_API_URL.to_string())
    }

    pub fn with_base_url(base_url: String) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = MiddlewareClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Self { client, base_url }
    }

    pub async fn search_address(&self, address: &str) -> Result<Vec<AddressResult>> {
        if address.trim().is_empty() {
            return Err(GeonorgeError::InvalidAddress(
                "Address cannot be empty".to_string(),
            ));
        }

        let normalized_address = normalize_house_letter(address);

        info!("Searching for address: {}", address);
        let url = format!("{}/sok", self.base_url);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("sok", normalized_address.as_str()),
                ("treffPerSide", "10"),
                ("side", "0"),
            ])
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Geonorge API: {}", e);
                GeonorgeError::RequestError(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Geonorge API returned error {}: {}", status, error_text);
            return Err(GeonorgeError::ApiError(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }

        let response_text = response.text().await.map_err(|e| {
            error!("Failed to read response body: {}", e);
            GeonorgeError::RequestError(e.to_string())
        })?;

        debug!("Geonorge API response: {}", response_text);

        let geonorge_response: GeonorgeResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                error!("Failed to parse Geonorge response: {}", e);
                GeonorgeError::ParseError(e.to_string())
            })?;

        if geonorge_response.addresses.is_empty() {
            return Err(GeonorgeError::NoResults(address.to_string()));
        }

        info!(
            "Found {} result(s) for address: {}",
            geonorge_response.addresses.len(),
            address
        );

        Ok(geonorge_response.addresses)
    }

    pub async fn get_coordinates(&self, address: &str) -> Result<Option<(f64, f64)>> {
        let results = self.search_address(address).await?;

        Ok(results.first().and_then(|r| r.get_coordinates()))
    }
}

impl Default for KoordinatClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_house_letter;

    #[test]
    fn normalizes_space_between_number_and_letter() {
        assert_eq!(
            normalize_house_letter("Banevegen 12 b, Ridabu"),
            "Banevegen 12b, Ridabu"
        );
        assert_eq!(normalize_house_letter("Storgata 7 a"), "Storgata 7a");
    }

    #[test]
    fn leaves_already_compact_intact() {
        assert_eq!(
            normalize_house_letter("Banevegen 12b, Ridabu"),
            "Banevegen 12b, Ridabu"
        );
    }

    #[test]
    fn does_not_remove_spaces_after_letters_or_words() {
        assert_eq!(normalize_house_letter("Gate 12b c"), "Gate 12b c");
        assert_eq!(normalize_house_letter("Gate 123, Oslo"), "Gate 123, Oslo");
    }
}
