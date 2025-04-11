use super::response::{Ansatt, Avdeling, Region};
use crate::client::ApiClient;
use crate::error::ApiError;
use crate::{error::Result, orgenhet::response::Orgenhet};
use reqwest::Response;
use serde::Deserialize;
use tracing::{error, info};

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

impl OrgEnhetClient {
    pub async fn new() -> Self {
        OrgEnhetClient {
            api_client: ApiClient::new("ORG_ENHET", "KEYCLOAK_ORGENHET").await,
        }
    }

    //https://tilsynskvittering.inspektor-utv.mattilsynet.io/api/orgenhet-api/ansatte?page.size=10000&page.number=0
    pub async fn hent_alle_ansatte(&self) -> Result<Vec<Ansatt>> {
        let url = format!("{}ansatte", &self.api_client.get_base_url(),);

        info!("Henter alle ansatte fra: {:?}", url);

        let response = self.api_get(&url).await?;

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

    async fn api_get(&self, url: &String) -> Result<Response> {
        let response = self
            .api_client
            .get_client()
            .get(url)
            .header(
                reqwest::header::CONTENT_TYPE.to_string(),
                "application/json",
            )
            .bearer_auth(self.api_client.get_token())
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;
        Ok(response)
    }

    pub async fn hent_ansatt_med_brukernavn(&self, brukernavn: String) -> Result<Ansatt> {
        let url = format!("{}ansatte/{}", &self.api_client.get_base_url(), brukernavn);

        info!("Henter ansatt med brukernavn fra: {:?}", url);

        let response = self.api_get(&url).await?;

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
    pub async fn hent_ansatte_i_avdeling(&self, avdeling_id: String) -> Result<Vec<Ansatt>> {
        let url = format!(
            "{}kontorer/{}/ansatte",
            &self.api_client.get_base_url(),
            avdeling_id
        );

        info!("Henter ansatte fra: {:?}", url);

        let response = self
            .api_client
            .get_client()
            .get(&url)
            .header(
                reqwest::header::CONTENT_TYPE.to_string(),
                "application/json",
            )
            .bearer_auth(self.api_client.get_token())
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;

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
            "{}orgenheter/parenttype/{}/id/{}",
            &self.api_client.get_base_url(),
            orgenhet_type,
            id
        );

        info!("Henter org_enhet fra: {:?}", url);

        let response = self.api_get(&url).await?;

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

        let response = self.api_get(&url).await?;

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

        let response = self.api_get(&url).await?;

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
}
