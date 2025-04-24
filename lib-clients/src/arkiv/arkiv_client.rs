use crate::arkiv::response::{
    ArkivClientJournalpost, ArkivClientSak, ArkivPdfKvittering, ArkivSakArkivering,
    ArkiverDokument, Dokument,
};
use crate::client::ApiClient;
use crate::error::ApiError;
use crate::error::Result;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap};
use serde::Deserialize;
use tracing::{error, info};
use uuid::Uuid;

pub struct ArkivClient {
    api_client: ApiClient,
}

#[derive(Deserialize)]
struct JournalposterEmbedded {
    #[serde(rename = "journalpostList")]
    journalpost_list: Vec<ArkivClientJournalpost>,
}

#[derive(Deserialize)]
struct JournalposterResponse {
    #[serde(rename = "_embedded")]
    embedded: JournalposterEmbedded,
}

impl ArkivClient {
    pub async fn new(base_url_prefix: Option<&str>, auth_config_prefix: Option<&str>) -> Self {
        let base = base_url_prefix.unwrap_or("ARKIV");
        let auth = auth_config_prefix.unwrap_or("KEYCLOAK_ARKIV");

        ArkivClient {
            api_client: ApiClient::new(base, auth).await,
        }
    }

    #[tracing::instrument(
    name = "Henter arkiv sak",
    skip(self),
    fields(
        request_id = %Uuid::new_v4(),
        noarkaar = %noarkaar,
        noarksaksnummer = %noarksaksnummer
    )
    )]
    pub async fn get_arkiv_sak(
        &self,
        noarkaar: &str,
        noarksaksnummer: &str,
    ) -> Result<ArkivClientSak> {
        let url = format!(
            "{}/arkiv/saker/{}/{}",
            self.api_client.get_base_url(),
            noarkaar,
            noarksaksnummer
        );

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let text = response.text().await.unwrap();
            let sak: ArkivClientSak =
                serde_json::from_str(&text).map_err(|e| ApiError::ParseError(e.to_string()))?;
            info!("Hentet sak {:?} fra arkiv api.", sak);
            Ok(sak)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente sak {}/{}, error message {}",
                noarkaar, noarksaksnummer, error_message
            );

            Err(ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!(
                    "Failed to get arkiv sak, HTTP Status: {}, response {}",
                    status, error_message
                ),
            })
        }
    }

    #[tracing::instrument(
        name = "Henter arkiv sak sine journalposter",
        skip(self),
        fields(
        request_id = %Uuid::new_v4(),
        noarkaar = %noarkaar,
        noarksaksnummer = %noarksaksnummer
        )
    )]
    pub async fn get_arkiv_sak_journalposter(
        &self,
        noarkaar: &str,
        noarksaksnummer: &str,
    ) -> Result<Vec<ArkivClientJournalpost>> {
        let url = format!(
            "{}/arkiv/saker/{}/{}/journalposter",
            self.api_client.get_base_url(),
            noarkaar,
            noarksaksnummer
        );

        let response = self.api_client.api_get(&url).await?;

        if response.status().is_success() {
            let text = response.text().await?;
            let journalposter_response: JournalposterResponse =
                serde_json::from_str(&text).map_err(|e| ApiError::ParseError(e.to_string()))?;
            let journalposter = journalposter_response.embedded.journalpost_list;
            info!(
                "Hentet journalposter på sak {:?} fra arkiv api.",
                journalposter
            );
            Ok(journalposter)
        } else {
            let status = response.status();
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Klarte ikke hente journalposter paa sak {}/{}, error message {}",
                noarkaar, noarksaksnummer, error_message
            );

            Err(ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!(
                    "Failed to get arkiv sak, HTTP Status: {}, response {}",
                    status, error_message
                ),
            })
        }
    }

    #[tracing::instrument(
        name = "Oppretter arkiv sak",
        skip(self),
        fields(request_id = %Uuid::new_v4(), arkiv_sak = %arkiv_post_sak)
    )]
    pub async fn opprett_arkiv_sak_med_mt_enhet(
        &self,
        arkiv_post_sak: &ArkivSakArkivering,
    ) -> Result<ArkivSakArkivering> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        let response = self
            .api_client
            .get_client()
            .post(format!("{}/arkiv/sakMtEnhet", self.api_client.get_base_url()).as_str())
            .bearer_auth(self.api_client.get_token())
            .headers(headers)
            .json(&arkiv_post_sak)
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;
        info!("Response: {:?}", response);

        let status = response.status();
        let response_text = response.text().await.map_err(|e| ApiError::ClientError {
            resource: "reqwest".to_string(),
            error_message: e.to_string(),
        })?;

        if !status.is_success() {
            return Err(ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!(
                    "Failed to create arkiv sak, HTTP Status: {}, response {}",
                    status, response_text
                ),
            });
        }
        let archive_response: ArkivSakArkivering = serde_json::from_str(&response_text)
            .map_err(|e| ApiError::ParseError(e.to_string()))?;
        Ok(archive_response)
    }

    #[tracing::instrument(
        name = "Legger til journalpost på sak",
        skip(self),
        fields(request_id = %Uuid::new_v4(), journalpost = %journalpost)
    )]
    pub async fn legg_til_journalpost_paa_sak(
        &self,
        journalpost: &ArkiverDokument,
    ) -> Result<ArkivPdfKvittering> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());

        let response = self
            .api_client
            .get_client()
            .post(format!("{}/arkiv/fil", self.api_client.get_base_url()).as_str())
            .bearer_auth(self.api_client.get_token())
            .headers(headers)
            .json(&journalpost)
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;
        let status = response.status();
        let response_text = response.text().await.map_err(|e| ApiError::ClientError {
            resource: "reqwest".to_string(),
            error_message: e.to_string(),
        })?;
        if !status.is_success() {
            return Err(ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!(
                    "Failed to legg_til_journalpost_paa_sak arkiv sak, HTTP Status: {}, response {}",
                    status, response_text
                ),
            });
        }
        let archive_response: ArkivPdfKvittering =
            serde_json::from_str(&response_text).map_err(|e| ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!("RESPONSE : {} : error : {}", response_text, e),
            })?;
        Ok(archive_response)
    }

    #[tracing::instrument(
        name = "Legger til journalpost på sak med arkiv bruker",
        skip(self),
        fields(request_id = %Uuid::new_v4(), journalpost = %journalpost)
    )]
    pub async fn legg_til_journalpost_paa_sak_med_arkiv_bruker(
        &self,
        journalpost: &ArkiverDokument,
    ) -> Result<ArkivPdfKvittering> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());

        let response = self
            .api_client
            .get_client()
            .post(format!("{}/arkiv/filArkivBruker", self.api_client.get_base_url()).as_str())
            .bearer_auth(self.api_client.get_token())
            .headers(headers)
            .json(&journalpost)
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;
        let status = response.status();
        let response_text = response.text().await.map_err(|e| ApiError::ClientError {
            resource: "reqwest".to_string(),
            error_message: e.to_string(),
        })?;
        if !status.is_success() {
            return Err(ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!(
                    "Failed to legg_til_journalpost_paa_sak_med_arkiv_bruker paa arkiv sak, HTTP Status: {}, response {}",
                    status, response_text
                ),
            });
        }
        let archive_response: ArkivPdfKvittering =
            serde_json::from_str(&response_text).map_err(|e| ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!("RESPONSE : {} : error : {}", response_text, e),
            })?;
        Ok(archive_response)
    }

    #[tracing::instrument(
        name = "Legger til vedlegg på journalpost",
        skip(self),
        fields(
        request_id = %Uuid::new_v4(),
        journalpost_id = %journalpost_id,
        vedlegg = %vedlegg,
        )
    )]
    pub async fn legg_til_vedlegg_paa_journalpost(
        &self,
        journalpost_id: &str,
        hoveddokument: Option<bool>,
        vedlegg: &Dokument,
    ) -> Result<String> {
        let response = self
            .api_client
            .get_client()
            .post(
                format!(
                    "{}/arkiv/journalposter/{}/dokumenter?erHoveddokument={}",
                    self.api_client.get_base_url(),
                    journalpost_id,
                    hoveddokument.unwrap_or(false)
                )
                .as_str(),
            )
            .bearer_auth(self.api_client.get_token())
            .json(vedlegg)
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| ApiError::ClientError {
            resource: "reqwest".to_string(),
            error_message: e.to_string(),
        })?;

        if !status.is_success() {
            return Err(ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!(
                    "Failed to legg_til_vedlegg_paa_journalpost arkiv sak, HTTP Status: {}, response {}",
                    status, response_text
                ),
            });
        }

        Ok(response_text)
    }

    #[tracing::instrument(
        name = "Setter journalpost status på sak",
        skip(self),
        fields(
        request_id = %Uuid::new_v4(),
        noarkaar = %noarkaar,
        noarksaksnummer = %noarksaksnummer,
        journalpost_id = %journalpost_id,
        status = %status
        )
    )]
    pub async fn set_journalpost_status(
        &self,
        noarkaar: &str,
        noarksaksnummer: &str,
        journalpost_id: &str,
        status: &str,
    ) -> Result<String> {
        let response = self
            .api_client
            .get_client()
            .put(
                format!(
                    "{}/arkiv/saker/{}/{}/journalposter/{}/status/{}",
                    self.api_client.get_base_url(),
                    noarkaar,
                    noarksaksnummer,
                    journalpost_id,
                    status,
                )
                .as_str(),
            )
            .bearer_auth(self.api_client.get_token())
            .json(status)
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;
        let status = response.status();
        let response_text = response.text().await.map_err(|e| ApiError::ClientError {
            resource: "reqwest".to_string(),
            error_message: e.to_string(),
        })?;

        if !status.is_success() {
            return Err(ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!(
                    "Failed to set_journalpost_status arkiv sak, HTTP Status: {}, response {}",
                    status, response_text
                ),
            });
        }

        Ok(response_text)
    }

    #[tracing::instrument(
        name = "Setter saksansvarlig på sak",
        skip(self),
        fields(
        request_id = %Uuid::new_v4(),
        noarkaar = %noarkaar,
        noarksaksnummer = %noarksaksnummer,
        ansvarlig = %ansvarlig
        )
    )]
    pub async fn set_saksansvarlig(
        &self,
        noarkaar: &str,
        noarksaksnummer: &str,
        ansvarlig: &str,
    ) -> Result<String> {
        let response = self
            .api_client
            .get_client()
            .put(
                format!(
                    "{}/arkiv/saker/{}/{}/saksansvarlig/{}",
                    self.api_client.get_base_url(),
                    noarkaar,
                    noarksaksnummer,
                    ansvarlig,
                )
                .as_str(),
            )
            .bearer_auth(self.api_client.get_token())
            .json(ansvarlig)
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;
        let status = response.status();
        let response_text = response.text().await.map_err(|e| ApiError::ClientError {
            resource: "reqwest".to_string(),
            error_message: e.to_string(),
        })?;

        if !status.is_success() {
            return Err(ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!(
                    "Failed put saksansvarlig, HTTP Status: {}, response {}",
                    status, response_text
                ),
            });
        }

        Ok(response_text)
    }

    #[tracing::instrument(
        name = "Setter status på sak",
        skip(self),
        fields(
        request_id = %Uuid::new_v4(),
        noarkaar = %noarkaar,
        noarksaksnummer = %noarksaksnummer,
        status = %status
        )
    )]
    pub async fn set_sak_status(
        &self,
        noarkaar: &str,
        noarksaksnummer: &str,
        status: &str,
    ) -> Result<String> {
        // FERDIG("SAKSTATUS\$F"),
        // AVSLUTTET("SAKSTATUS\$A"),
        // UNDER_BEHANDLING("SAKSTATUS\$B")
        let response = self
            .api_client
            .get_client()
            .put(
                format!(
                    "{}/arkiv/saker/{}/{}/status/{}",
                    self.api_client.get_base_url(),
                    noarkaar,
                    noarksaksnummer,
                    status,
                )
                .as_str(),
            )
            .bearer_auth(self.api_client.get_token())
            .json(status)
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;
        let status = response.status();
        let response_text = response.text().await.map_err(|e| ApiError::ClientError {
            resource: "reqwest".to_string(),
            error_message: e.to_string(),
        })?;

        if !status.is_success() {
            return Err(ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!(
                    "Failed to set_sak_status arkiv sak, HTTP Status: {}, response {}",
                    status, response_text
                ),
            });
        }

        Ok(response_text)
    }

    #[tracing::instrument(
        name = "Avskriver restanse på journalpost",
        skip(self),
        fields(
        request_id = %Uuid::new_v4(),
        journalpost_id = %journalpost_id,
        avskrivingsmaate = %avskrivingsmaate,
        merknad = %merknad
        )
    )]
    pub async fn avskriv_restanse_journalpost(
        &self,
        journalpost_id: &str,
        avskrivingsmaate: &str,
        merknad: &str,
    ) -> Result<String> {
        let response = self
            .api_client
            .get_client()
            .post(
                format!(
                    "{}/arkiv/journalposter/{}/avskriv?avskrivingsmaate={}&merknad={}",
                    self.api_client.get_base_url(),
                    journalpost_id,
                    avskrivingsmaate,
                    merknad,
                )
                .as_str(),
            )
            .header("Content-Length", "0") //Trengs for og unngå 411 Length Required
            .bearer_auth(self.api_client.get_token())
            .send()
            .await
            .map_err(|e| ApiError::ClientError {
                resource: "reqwest".to_string(),
                error_message: e.to_string(),
            })?;
        let status = response.status();
        let response_text = response.text().await.map_err(|e| ApiError::ClientError {
            resource: "reqwest".to_string(),
            error_message: e.to_string(),
        })?;

        if !status.is_success() {
            return Err(ApiError::ClientError {
                resource: "Arkiv".to_string(),
                error_message: format!(
                    "Failed to avskriv_restanse_journalpost {}, HTTP Status: {}, response {}",
                    journalpost_id, status, response_text
                ),
            });
        }
        Ok(response_text)
    }
}
