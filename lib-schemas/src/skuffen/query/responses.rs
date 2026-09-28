use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::dokument::DokumentId;
use crate::skuffen::journalpost::{JournalpostId, JournalpostType, Journalpoststatus};
use crate::skuffen::sak::{Ordningsverdi, Saksnummer, Saksstatus, Sakstittel};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum TilgjengelighetResponse {
    Offentlig,
    Skjermet {
        tilgangskode: String,
        tilgangshjemmel: String,
    },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum ParttypeResponse {
    Person,
    Virksomhet,
    Annet(String),
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct KorrespondansepartResponse {
    pub navn: String,
    pub parttype: ParttypeResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SakResponse {
    pub sakstittel: Sakstittel,
    pub saksbehandler: Option<String>,
    pub saksbehandler_enhet: Option<String>,
    pub saksstatus: Saksstatus,
    pub tilgjengelighet: TilgjengelighetResponse,
    pub ordningsverdi: Ordningsverdi,
    pub saksnummer: Saksnummer,
    pub kildesystem: String,
    pub lukket: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journalposter: Option<Vec<JournalpostResponse>>,
}

/// Metadata for et dokument i arkivet. Første dokument i en journalposts
/// dokumentliste er hoveddokumentet; resten er vedlegg.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct DokumentResponse {
    pub dokument_id: DokumentId,
    pub tittel: String,
    pub filtype: String,
    pub dokument_referanse: Option<Uuid>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JournalpostResponse {
    pub tittel: String,
    /// Arkivets dokumentdato uten tidssone, f.eks. `2025-10-14T00:00:00`.
    pub dokument_dato: NaiveDateTime,
    pub journalposttype: JournalpostType,
    pub journalstatus: Journalpoststatus,
    pub tilgjengelighet: TilgjengelighetResponse,
    pub saksbehandler: Option<String>,
    pub saksbehandler_enhet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub korrespondanseparter: Option<Vec<KorrespondansepartResponse>>,
    pub dokumenter: Vec<DokumentResponse>,
    pub journalpost_id: JournalpostId,
    pub kildesystem: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn journalpost_response_har_sonefri_dokumentdato_og_dokument_id() {
        let value = json!({
            "tittel": "Vedtak",
            "dokument_dato": "2025-10-14T00:00:00",
            "journalposttype": "InterntNotat",
            "journalstatus": "Midlertidig",
            "tilgjengelighet": "Offentlig",
            "saksbehandler": null,
            "saksbehandler_enhet": null,
            "dokumenter": [
                {
                    "dokument_id": "77357",
                    "tittel": "Hoveddokument",
                    "filtype": "PDF",
                    "dokument_referanse": null
                }
            ],
            "journalpost_id": "51410",
            "kildesystem": "SKUFFEN"
        });

        let parsed: JournalpostResponse = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(
            parsed.dokument_dato,
            chrono::NaiveDate::from_ymd_opt(2025, 10, 14)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        );
        assert_eq!(parsed.dokumenter[0].dokument_id.as_str(), "77357");
        assert_eq!(parsed.saksbehandler, None);
        assert_eq!(serde_json::to_value(&parsed).unwrap(), value);
    }

    #[test]
    fn journalpost_response_avviser_dato_uten_klokkeslett() {
        let value = json!({
            "tittel": "Vedtak",
            "dokument_dato": "2025-10-14",
            "journalposttype": "InterntNotat",
            "journalstatus": "Midlertidig",
            "tilgjengelighet": "Offentlig",
            "saksbehandler": null,
            "saksbehandler_enhet": null,
            "dokumenter": [],
            "journalpost_id": "51410",
            "kildesystem": "SKUFFEN"
        });

        assert!(serde_json::from_value::<JournalpostResponse>(value).is_err());
    }

    #[test]
    fn tilgjengelighet_response_speiler_command_side_shape() {
        let offentlig = TilgjengelighetResponse::Offentlig;
        assert_eq!(
            serde_json::to_value(&offentlig).unwrap(),
            json!("Offentlig")
        );

        let skjermet = TilgjengelighetResponse::Skjermet {
            tilgangskode: "UO".to_string(),
            tilgangshjemmel: "Offl. § 13".to_string(),
        };
        assert_eq!(
            serde_json::to_value(&skjermet).unwrap(),
            json!({ "Skjermet": { "tilgangskode": "UO", "tilgangshjemmel": "Offl. § 13" } })
        );
    }

    #[test]
    fn tilgjengelighet_response_er_permissiv_for_historiske_koder() {
        let value = json!({
            "Skjermet": { "tilgangskode": "", "tilgangshjemmel": "utgått hjemmel" }
        });
        let parsed: TilgjengelighetResponse = serde_json::from_value(value).unwrap();
        assert_eq!(
            parsed,
            TilgjengelighetResponse::Skjermet {
                tilgangskode: "".to_string(),
                tilgangshjemmel: "utgått hjemmel".to_string(),
            }
        );
    }

    #[test]
    fn korrespondansepart_response_tolererer_ukjent_parttype() {
        let value = json!({
            "navn": "Ukjent Part",
            "parttype": { "Annet": "Foretak" },
            "id": "123"
        });
        let parsed: KorrespondansepartResponse = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.navn, "Ukjent Part");
        assert_eq!(
            parsed.parttype,
            ParttypeResponse::Annet("Foretak".to_string())
        );
        assert_eq!(parsed.id.as_deref(), Some("123"));
    }
}
