use super::response::{Ansatt, Avdeling, Kontor, Region, Seksjon};
use crate::client::ApiClient;
use crate::error::ApiError;
use crate::{error::Result, orgenhet::response::Orgenhet};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde::Deserialize;
use tracing::{error, info, instrument};
use uuid::Uuid;

pub struct OrgEnhetClient {
    api_client: ApiClient,
}

#[derive(Deserialize)]
struct AnsatteResponse {
    #[serde(rename = "_embedded")]
    embedded: AnsatteEmbedded,
}

#[derive(Deserialize)]
struct AnsatteEmbedded {
    #[serde(rename = "ansattList")]
    ansatt_list: Vec<Ansatt>,
}

#[derive(Deserialize)]
struct RegionResponse {
    #[serde(rename = "_embedded")]
    embedded: RegionEmbedded,
}

#[derive(Deserialize)]
struct RegionEmbedded {
    #[serde(rename = "regionList")]
    region_list: Vec<Region>,
}

#[derive(Deserialize)]
struct AvdelingResponse {
    #[serde(rename = "_embedded")]
    embedded: AvdelingEmbedded,
}

#[derive(Deserialize)]
struct AvdelingEmbedded {
    #[serde(rename = "avdelingList")]
    avdeling_list: Vec<Avdeling>,
}

#[derive(Deserialize)]
struct SeksjonResponse {
    #[serde(rename = "_embedded")]
    embedded: SeksjonEmbedded,
}

#[derive(Deserialize)]
struct SeksjonEmbedded {
    #[serde(rename = "seksjonList")]
    seksjonlist: Vec<Seksjon>,
}

impl OrgEnhetClient {
    #[instrument(name = "Creating OrgenhetClient")]
    pub async fn new(base_url_prefix: Option<&str>, auth_config_prefix: Option<&str>) -> Self {
        let base = base_url_prefix.unwrap_or("ORG_ENHET");
        let auth = auth_config_prefix.unwrap_or("KEYCLOAK_ORGENHET");

        OrgEnhetClient {
            api_client: ApiClient::new(base, auth).await,
        }
    }

    //https://tilsynskvittering.inspektor-utv.mattilsynet.io/api/orgenhet-api/ansatte?page.size=10000&page.number=0
    pub async fn hent_alle_ansatte(&self) -> Result<Vec<Ansatt>> {
        let url = format!(
            "{}/ansatte?page.size=2000&page.number=0",
            &self.api_client.get_base_url(),
        );

        info!("Henter alle ansatte fra: {:?}", url);

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            // let json: serde_json::Value = response.json().await.unwrap();
            // println!("{:?}", json);
            let ansatt_response: AnsatteResponse = response
                .json()
                .await
                .map_err(|e| ApiError::ParseError(e.to_string()))?;

            let ansatte = ansatt_response.embedded.ansatt_list;
            info!("Hentet {} ansatte fra org_enhet api.", ansatte.len());
            Ok(ansatte)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente ansatte. error code {}, error message {}",
                status, error_message
            );
            Err(ApiError::ClientError {
                resource: "org_enhet".to_string(),
                error_message: format!(
                    "Failed to fetch ansatte i avdeling. HTTP Status: {}, response: {}",
                    status, error_message
                ),
            })
            .into()
        }
    }

    pub async fn hent_ansatt_med_brukernavn(&self, brukernavn: String) -> Result<Ansatt> {
        let url = format!("{}/ansatte/{}", &self.api_client.get_base_url(), brukernavn);

        info!("Henter ansatt med brukernavn fra: {:?}", url);

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            // let json: serde_json::Value = response.json().await.unwrap();
            // println!("{:?}", json);
            let ansatt: Ansatt = response
                .json()
                .await
                .map_err(|e| ApiError::ParseError(e.to_string()))?;
            info!("Hentet ansatt {:?} fra org_enhet api.", ansatt);
            Ok(ansatt)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Klarte ikke hente ansatt. brukernavn {}", brukernavn);
            Err(ApiError::ClientError {
                resource: "org_enhet".to_string(),
                error_message: format!(
                    "Failed to fetch ansatte i avdeling. HTTP Status: {}, response: {}",
                    status, error_message
                ),
            })?
        }
    }

    pub async fn hent_ansatte_i_seksjon(&self, seksjon_id: String) -> Result<Vec<Ansatt>> {
        let url = format!(
            "{}/seksjoner/{}/ansatte?page.size=2000&page.number=0",
            &self.api_client.get_base_url(),
            seksjon_id
        );

        info!("Henter ansatte fra: {:?}", url);

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let ansatt_response: AnsatteResponse = response
                .json()
                .await
                .map_err(|e| ApiError::ParseError(e.to_string()))?;
            let ansatte = ansatt_response.embedded.ansatt_list;
            info!("Hentet {} ansatte fra org_enhet api.", ansatte.len());
            Ok(ansatte)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente ansatte. seksjon_id {}, error code {}, error message {}",
                seksjon_id, status, error_message
            );
            Err(ApiError::ClientError {
                resource: "org_enhet".to_string(),
                error_message: format!(
                    "Failed to fetch ansatte i seksjon. HTTP Status: {}, response: {}",
                    status, error_message
                ),
            })
            .into()
        }
    }

    pub async fn hent_ansatte_i_avdeling(&self, avdeling_id: String) -> Result<Vec<Ansatt>> {
        let url = format!(
            "{}/kontorer/{}/ansatte?page.size=2000&page.number=0",
            &self.api_client.get_base_url(),
            avdeling_id
        );

        info!("Henter ansatte fra: {:?}", url);

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let ansatt_response: AnsatteResponse = response
                .json()
                .await
                .map_err(|e| ApiError::ParseError(e.to_string()))?;
            let ansatte = ansatt_response.embedded.ansatt_list;
            info!("Hentet {} ansatte fra org_enhet api.", ansatte.len());
            Ok(ansatte)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente ansatte. avdeling_id {}, error code {}, error message {}",
                avdeling_id, status, error_message
            );
            Err(ApiError::ClientError {
                resource: "org_enhet".to_string(),
                error_message: format!(
                    "Failed to fetch ansatte i avdeling. HTTP Status: {}, response: {}",
                    status, error_message
                ),
            })
            .into()
        }
    }

    pub async fn hent_overordnet_orgenhet(
        &self,
        orgenhet_type: String,
        id: &str,
    ) -> Result<Orgenhet> {
        let url = format!(
            "{}/orgenheter/parenttype/{}/id/{}",
            &self.api_client.get_base_url(),
            orgenhet_type,
            id
        );

        info!("Henter org_enhet fra: {:?}", url);

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let orgenhet: Orgenhet = response
                .json()
                .await
                .map_err(|e| ApiError::ParseError(e.to_string()))?;
            info!("Hentet {:?} orgenhet fra org_enhet api.", orgenhet);
            Ok(orgenhet)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente orgeneht. type {} id {}, error code {}, error message {}",
                orgenhet_type, id, status, error_message
            );
            Err(ApiError::ClientError {
                resource: "org_enhet".to_string(),
                error_message: format!(
                    "Failed to fetch ansatte i avdeling. HTTP Status: {}, response: {}",
                    status, error_message
                ),
            })
        }
    }

    pub async fn hent_regioner(&self) -> Result<Vec<Region>> {
        let url = format!("{}/regioner", &self.api_client.get_base_url(),);

        info!("Henter regioner fra: {:?}", url);

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let region_response: RegionResponse = response
                .json()
                .await
                .map_err(|e| ApiError::ParseError(e.to_string()))?;
            let regioner = region_response.embedded.region_list;
            info!("Hentet {} regioner fra org_enhet api.", regioner.len());
            Ok(regioner)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente regioner. error code {}, error message {}",
                status, error_message
            );
            Err(ApiError::ClientError {
                resource: "org_enhet".to_string(),
                error_message: format!(
                    "Failed to fetch regioner. HTTP Status: {}, response: {}",
                    status, error_message
                ),
            })
        }
    }

    pub async fn hent_avdelinger(&self) -> Result<Vec<Avdeling>> {
        let url = format!("{}/avdelinger", &self.api_client.get_base_url(),);

        info!("Henter avdelinger fra: {:?}", url);

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let avdeling_response: AvdelingResponse = response
                .json()
                .await
                .map_err(|e| ApiError::ParseError(e.to_string()))?;
            let avdelinger = avdeling_response.embedded.avdeling_list;
            info!("Hentet {} avdelinger fra org_enhet api.", avdelinger.len());
            Ok(avdelinger)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente avdelinger. error code {}, error message {}",
                status, error_message
            );
            Err(ApiError::ClientError {
                resource: "org_enhet".to_string(),
                error_message: format!(
                    "Failed to fetch avdelinger. HTTP Status: {}, response: {}",
                    status, error_message
                ),
            })
        }
    }

    pub async fn hent_seksjoner(&self) -> Result<Vec<Seksjon>> {
        let url = format!("{}/seksjoner", &self.api_client.get_base_url(),);

        info!("Henter seksjoner fra: {:?}", url);

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let seksjon_response: SeksjonResponse = response
                .json()
                .await
                .map_err(|e| ApiError::ParseError(e.to_string()))?;
            let seksjoner = seksjon_response.embedded.seksjonlist;
            info!("Hentet {} seksjoner fra org_enhet api.", seksjoner.len());
            Ok(seksjoner)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente regioner. error code {}, error message {}",
                status, error_message
            );
            Err(ApiError::ClientError {
                resource: "org_enhet".to_string(),
                error_message: format!(
                    "Failed to fetch regioner. HTTP Status: {}, response: {}",
                    status, error_message
                ),
            })
        }
    }

    #[instrument(
        name = "Fetching kontor by orgenhet_id",
        skip(self),
        fields(
            request_id = %Uuid::new_v4(),
            orgenhet_id = %orgenhet_id
        )
    )]
    pub async fn hent_kontor_med_id(&self, orgenhet_id: &str) -> Result<Kontor> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let url = format!(
            "{}/kontorer/{}",
            self.api_client.get_base_url(),
            orgenhet_id,
        );

        let response = self.api_client.api_get(&url).await?;
        let status = response.status();
        let response_text = response.text().await.map_err(|e| ApiError::ClientError {
            resource: "reqwest".to_string(),
            error_message: e.to_string(),
        })?;

        if !status.is_success() {
            return Err(ApiError::ClientError {
                resource: "org_enhet".to_string(),
                error_message: format!(
                    "Failed to get kontor, HTTP Status: {}, response {}",
                    status, response_text
                ),
            });
        }

        let orgenhet_response: Kontor = serde_json::from_str(&response_text).unwrap();
        Ok(orgenhet_response)
    }
}
