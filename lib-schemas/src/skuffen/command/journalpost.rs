use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::{
    dokument::Dokument, journalpost::Postnummer, query::queries::SakKey, tilgang::Tilgjengelighet,
};
use crate::typer::{organisasjonsnummer::Organisasjonsnummer, personnummer::Personnummer};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum Parttype {
    Person,
    Virksomhet,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Korrespondansepart {
    pub navn: String,
    pub parttype: Parttype,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum MottakerId {
    Person {
        fødselsnummer: Personnummer,
    },
    Virksomhet {
        organisasjonsnummer: Organisasjonsnummer,
    },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Postadresse {
    pub adresse: String,
    pub postnummer: Postnummer,
    pub poststed: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Utsendingsmottaker {
    pub navn: String,
    pub id: MottakerId,
    pub adresse: Postadresse,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettInngåendeJournalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    pub avsender: Korrespondansepart,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettUtgåendeJournalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    pub mottakere: Vec<Korrespondansepart>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettUtgåendeJournalpostMedUtsending {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    pub mottakere: Vec<Utsendingsmottaker>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettInterntNotatJournalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct JournalpostCommon {
    pub client_reference: Uuid,
    pub tittel: String,
    pub dokument_dato: String,
    pub saksbehandler: String,
    pub saksbehandler_enhet: String,
    pub tilgjengelighet: Tilgjengelighet,
    /// Første dokument i lista er hoveddokument.
    pub dokumenter: Vec<Dokument>,
    pub sak_key: SakKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kildesystem: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skuffen::dokument::Dokument;
    use crate::skuffen::tilgang::{Tilgangshjemmel, Tilgangskode};
    use serde_json::json;

    fn felles_skjermet() -> JournalpostCommon {
        JournalpostCommon {
            client_reference: Uuid::nil(),
            tittel: "Tittel".to_string(),
            dokument_dato: "2026-01-01".to_string(),
            saksbehandler: "sb".to_string(),
            saksbehandler_enhet: "enhet".to_string(),
            tilgjengelighet: Tilgjengelighet::Skjermet {
                tilgangskode: Tilgangskode::new("UO").unwrap(),
                tilgangshjemmel: Tilgangshjemmel::new("Offl. § 13").unwrap(),
            },
            dokumenter: Vec::<Dokument>::new(),
            sak_key: SakKey::ClientReference(Uuid::nil()),
            kildesystem: None,
        }
    }

    #[test]
    fn inngaaende_med_skjermet_og_korrespondansepart_roundtrip() {
        let cmd = OpprettInngåendeJournalpost {
            felles: felles_skjermet(),
            avsender: Korrespondansepart {
                navn: "Ola Nordmann".to_string(),
                parttype: Parttype::Person,
            },
        };
        let value = serde_json::to_value(&cmd).unwrap();
        // tilgjengelighet er nested (eksternt tagget), avsender har navn+parttype.
        assert_eq!(
            value["tilgjengelighet"],
            json!({ "Skjermet": { "tilgangskode": "UO", "tilgangshjemmel": "Offl. § 13" } })
        );
        assert_eq!(value["avsender"]["parttype"], json!("Person"));
        let back: OpprettInngåendeJournalpost = serde_json::from_value(value).unwrap();
        assert_eq!(back, cmd);
    }

    #[test]
    fn utgaaende_med_utsendingsmottaker_roundtrip() {
        let cmd = OpprettUtgåendeJournalpostMedUtsending {
            felles: felles_skjermet(),
            mottakere: vec![Utsendingsmottaker {
                navn: "Bedrift AS".to_string(),
                id: MottakerId::Virksomhet {
                    organisasjonsnummer: Organisasjonsnummer::new("995298775").unwrap(),
                },
                adresse: Postadresse {
                    adresse: "Storgata 1".to_string(),
                    postnummer: Postnummer::new("0350").unwrap(),
                    poststed: "Oslo".to_string(),
                },
            }],
        };
        let value = serde_json::to_value(&cmd).unwrap();
        assert_eq!(
            value["mottakere"][0]["adresse"]["postnummer"],
            json!("0350")
        );
        let back: OpprettUtgåendeJournalpostMedUtsending = serde_json::from_value(value).unwrap();
        assert_eq!(back, cmd);
    }
}
