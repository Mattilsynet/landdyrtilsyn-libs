use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::journalpost::{JournalpostId, JournalpostType, Journalpoststatus};
use crate::skuffen::sak::{Ordningsverdi, Saksnummer, Saksstatus, Sakstittel};

/// Permissiv respons-representasjon av tilgjengelighet.
///
/// Respons-typene skal *rapportere*, aldri re-validere. Historiske eller
/// ugyldige koder som allerede finnes i arkivet må kunne deserialiseres uten
/// å feile. Derfor gjenbruker vi bevisst *ikke* de validerende kommando-side
/// newtypene [`crate::skuffen::tilgang::Tilgangskode`] /
/// [`crate::skuffen::tilgang::Tilgangshjemmel`] her — `Skjermet` bærer rå
/// `String`-felter som aldri avviser input.
///
/// Formen speiler [`crate::skuffen::tilgang::Tilgjengelighet`] (eksternt
/// tagget, jf. SKU-0004) slik at command- og query-siden er konsistente på
/// wire, men uten kommando-sidens strenghet.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum TilgjengelighetResponse {
    Offentlig,
    Skjermet {
        tilgangskode: String,
        tilgangshjemmel: String,
    },
}

/// Permissiv respons-representasjon av parttype.
///
/// Tolerant for ukjente/historiske verdier via `Annet`, slik at deserialisering
/// aldri feiler på data som allerede finnes i arkivet.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum ParttypeResponse {
    Person,
    Virksomhet,
    /// Ukjent eller historisk parttype som ikke tolkes videre.
    Annet(String),
}

/// Permissiv respons-representasjon av en korrespondansepart (avsender/mottaker).
///
/// Bærer kun det responsen trygt kan rapportere: navn, parttype og en valgfri
/// rå identifikator. Ingen validering skjer ved deserialisering.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct KorrespondansepartResponse {
    pub navn: String,
    pub parttype: ParttypeResponse,
    /// Rå identifikator (f.eks. fødsels-/organisasjonsnummer) dersom kjent.
    /// Ikke validert — kun rapportert.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Response payload for en sak.
///
/// `deny_unknown_fields` er satt for konsistens med [`JournalpostResponse`].
/// Ingen `#[serde(flatten)]` brukes her, så kombinasjonen er trygg.
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

/// Response metadata for dokument.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct DokumentResponse {
    pub tittel: String,
    pub filtype: String,
    pub dokument_referanse: Option<Uuid>,
}

/// Response payload for en journalpost.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JournalpostResponse {
    pub tittel: String,
    pub dokument_dato: String, // TODO: Denne skal være datetime
    pub journalposttype: JournalpostType,
    pub journalstatus: Journalpoststatus,
    pub tilgjengelighet: TilgjengelighetResponse,
    pub saksbehandler: Option<String>,
    pub saksbehandler_enhet: Option<String>,
    /// Korrespondanseparter (avsendere/mottakere) slik de er rapportert av arkivet.
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
        // Tom kode ville blitt avvist av command-side newtypene, men responsen
        // skal tolerere den.
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
