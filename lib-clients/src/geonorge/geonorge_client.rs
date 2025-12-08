use crate::geonorge::response::{AddressResult, GeonorgeResponse, KommuneOgFylke};
use crate::geonorge::{GeonorgeError, Koordinater, Result};
use reqwest_middleware::reqwest::Client;
use reqwest_middleware::{ClientBuilder as MiddlewareClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use tracing;

#[derive(Clone)]
pub struct GeoNorgeClient {
    client: ClientWithMiddleware,
}

const ADRESSER_URL: &str = "https://ws.geonorge.no/adresser/v1";
const PUNKTSOK_URL: &str = "https://api.kartverket.no/kommuneinfo/v1";

impl GeoNorgeClient {
    pub fn new() -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = MiddlewareClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Self { client }
    }

    #[tracing::instrument(
        name = "Henter kommune og fylke basert på koordinater fra GeoNorge.",
        skip(self, koordinater)
    )]
    async fn search_punkt(&self, koordinater: &Koordinater) -> Result<KommuneOgFylke> {
        let url = format!(
            "{}/punkt?nord={}&ost={}&koordsys=4258",
            PUNKTSOK_URL, koordinater.latitude, koordinater.longitude
        );

        let response = self.client.get(&url).send().await.map_err(|e| {
            tracing::error!("Klarte ikke sende request til GeoNorge");
            GeonorgeError::RequestError(e.to_string())
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Geonorge API returned error {}: {}", status, error_text);
            return Err(GeonorgeError::ApiError(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }

        let response_text = response.text().await.map_err(|e| {
            tracing::error!("Failed to read response body: {}", e);
            GeonorgeError::RequestError(e.to_string())
        })?;

        let geonorge_response: KommuneOgFylke =
            serde_json::from_str(&response_text).map_err(|e| {
                tracing::error!("Failed to parse Geonorge response: {}", e);
                GeonorgeError::ParseError(e.to_string())
            })?;

        Ok(geonorge_response)
    }

    async fn get_address_object(
        &self,
        adresse: &str,
        postnummer: &str,
        poststed: &str,
    ) -> Result<AddressResult> {
        if adresse.trim().is_empty() {
            return Err(GeonorgeError::InvalidAddress(
                "Address cannot be empty".to_string(),
            ));
        }

        let url = format!("{}/sok", ADRESSER_URL);
        let normalized_address = normalize_house_letter(adresse);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("fuzzy", "false"),
                ("adressetekst", &normalized_address),
                ("postnummer", postnummer),
                ("poststed", poststed),
                ("treffPerSide", "1"),
                ("side", "0"),
            ])
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Klarte ikke sende request til GeoNorge");
                GeonorgeError::RequestError(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Geonorge API returned error {}: {}", status, error_text);
            return Err(GeonorgeError::ApiError(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }

        let response_text = response.text().await.map_err(|e| {
            tracing::error!("Failed to read response body: {}", e);
            GeonorgeError::RequestError(e.to_string())
        })?;

        let geonorge_response: GeonorgeResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                tracing::error!("Failed to parse Geonorge response: {}", e);
                GeonorgeError::ParseError(e.to_string())
            })?;

        match geonorge_response.addresses.into_iter().next() {
            Some(address) => Ok(address),
            None => Err(GeonorgeError::NoResults(adresse.to_string())),
        }
    }

    async fn search_address(&self, address: &str) -> Result<Vec<AddressResult>> {
        if address.trim().is_empty() {
            return Err(GeonorgeError::InvalidAddress(
                "Address cannot be empty".to_string(),
            ));
        }

        let normalized_address = normalize_house_letter(address);

        let url = format!("{}/sok", ADRESSER_URL);

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
                tracing::error!("Klarte ikke sende request til GeoNorge");
                GeonorgeError::RequestError(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Geonorge API returned error {}: {}", status, error_text);
            return Err(GeonorgeError::ApiError(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }

        let response_text = response.text().await.map_err(|e| {
            tracing::error!("Failed to read response body: {}", e);
            GeonorgeError::RequestError(e.to_string())
        })?;

        let geonorge_response: GeonorgeResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                tracing::error!("Failed to parse Geonorge response: {}", e);
                GeonorgeError::ParseError(e.to_string())
            })?;

        if geonorge_response.addresses.is_empty() {
            return Err(GeonorgeError::NoResults(address.to_string()));
        }

        Ok(geonorge_response.addresses)
    }

    #[tracing::instrument(
        name = "Henter adresse basert på koordinater fra GeoNorge.",
        skip(self, koordinater)
    )]
    async fn search_address_from_coordinate(
        &self,
        koordinater: &Koordinater,
    ) -> Result<Option<AddressResult>> {
        let url = format!("{}/punktsok", ADRESSER_URL);
        let response = self
            .client
            .get(&url)
            .query(&[
                ("lat", koordinater.latitude.to_string()),
                ("long", koordinater.longitude.to_string()),
                ("radius", "100".to_string()), // meter
                ("treffPerSide", "1".to_string()),
            ])
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Klarte ikke sende request til GeoNorge");
                GeonorgeError::RequestError(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Geonorge API returned error {}: {}", status, error_text);
            return Err(GeonorgeError::ApiError(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }

        let response_text = response.text().await.map_err(|e| {
            tracing::error!("Failed to read response body: {}", e);
            GeonorgeError::RequestError(e.to_string())
        })?;

        let geonorge_response: GeonorgeResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                tracing::error!("Failed to parse Geonorge response: {}", e);
                GeonorgeError::ParseError(e.to_string())
            })?;

        let address = geonorge_response.addresses.into_iter().next();

        match address {
            Some(address) => Ok(Some(address)),
            _ => Err(GeonorgeError::NoResults(
                "Ingen adresse på koordinate".to_string(),
            )),
        }
    }

    pub async fn get_koordinater(&self, address: &str) -> Result<Option<(f64, f64)>> {
        let results = self.search_address(address).await?;
        Ok(results.first().and_then(|r| r.get_koordinater()))
    }

    pub async fn get_koordinater_fra_adresse(
        &self,
        adresse: &str,
        postnummer: &str,
        poststed: &str,
    ) -> Result<Option<(f64, f64)>> {
        let results = self
            .get_address_object(adresse, postnummer, poststed)
            .await?;
        Ok(results.get_koordinater())
    }

    pub async fn get_addresse_fra_koordinater(
        &self,
        koordinater: &Koordinater,
    ) -> Result<Option<AddressResult>> {
        self.search_address_from_coordinate(koordinater).await
    }

    pub async fn get_kommune_og_fylke_from_koordinat(
        &self,
        koordinater: &Koordinater,
    ) -> Result<KommuneOgFylke> {
        self.search_punkt(koordinater).await
    }
}

impl Default for GeoNorgeClient {
    fn default() -> Self {
        Self::new()
    }
}

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

#[cfg(test)]
mod tests {
    use crate::geonorge::GeoNorgeClient;

    #[tokio::test]
    async fn test_get_address_object() {
        let adresse = "Tårnvegen 41";
        let postnummer = "2380";
        let poststed = "Brumunddal";

        let geonorge_client = GeoNorgeClient::new();
        let adresse_obj = geonorge_client
            .get_address_object(adresse, postnummer, poststed)
            .await
            .unwrap();
        let koordinater = adresse_obj.get_koordinater();

        assert_eq!(koordinater, Some((60.87750554180073, 10.92918075347748)))
    }
}
