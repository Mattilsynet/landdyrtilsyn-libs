use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::{
    dokument::Dokument, journalpost::Postnummer, query::queries::SakKey, tilgang::Tilgjengelighet,
};
use crate::typer::{organisasjonsnummer::Organisasjonsnummer, personnummer::Personnummer};

/// Type korrespondansepart: privatperson eller virksomhet.
///
/// Uttrykker intensjon, ikke vendor-mekanikk. Kontrakten skiller kun på det
/// domenet trenger å vite — om motparten er en person eller en virksomhet.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum Parttype {
    Person,
    Virksomhet,
}

/// En korrespondansepart uten utsending.
///
/// Brukes for avsender på inngående journalposter og for mottaker på
/// utgående journalposter der det ikke skal skje noen faktisk utsending.
/// Bærer kun navn og parttype — ingen adresse eller identifikator, fordi
/// ingen forsendelse skal adresseres.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Korrespondansepart {
    pub navn: String,
    pub parttype: Parttype,
}

/// Identifikator for en mottaker ved utsending.
///
/// Erstatter den tidligere `MottakerId` (Person/Organisasjon) fra
/// `journalpost.rs`. Person identifiseres med validert fødselsnummer,
/// virksomhet med validert organisasjonsnummer.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum MottakerId {
    Person {
        fødselsnummer: Personnummer,
    },
    Virksomhet {
        organisasjonsnummer: Organisasjonsnummer,
    },
}

/// Postadresse for en utsendingsmottaker.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Postadresse {
    pub adresse: String,
    pub postnummer: Postnummer,
    pub poststed: String,
}

/// En mottaker som skal motta en faktisk utsending.
///
/// Krever både identifikator og postadresse, fordi en forsendelse må kunne
/// adresseres. Dette skiller seg bevisst fra [`Korrespondansepart`], som ikke
/// medfører utsending.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Utsendingsmottaker {
    pub navn: String,
    pub id: MottakerId,
    pub adresse: Postadresse,
}

/// Lag en inngående journalpost.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettInngåendeJournalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    pub avsender: Korrespondansepart,
}

/// Lag en utgående journalpost uten faktisk utsending.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettUtgåendeJournalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    pub mottaker: Korrespondansepart,
}

/// Lag en utgående journalpost med faktisk utsending til én eller flere mottakere.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettUtgåendeJournalpostMedUtsending {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    pub mottakere: Vec<Utsendingsmottaker>,
}

/// Lag et internt notat.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettInterntNotatJournalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
}

/// Felles felter delt av alle journalpost-opprettingskommandoer.
///
/// NB: Denne structen brukes med `#[serde(flatten)]` i kommandoene over.
/// `deny_unknown_fields` er derfor bevisst *utelatt* her — kombinasjonen
/// `flatten` + `deny_unknown_fields` er offisielt ustøttet i serde. Streng
/// felt-validering hører eventuelt hjemme på de ytre, ikke-flattenede
/// kommando-structene.
///
/// `tilgjengelighet` er en nested nøkkel (ALDRI flatten), slik at den
/// eksternt taggede [`Tilgjengelighet`]-shapen bevares på wire.
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
