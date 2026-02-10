use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::{
    dokument::Dokument, journalpost::UtsendingMottaker, query::queries::SakKey, tilgang::Tilgang,
};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettUgåendeJournalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    avsender: Option<String>,
    mottaker: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettUgåendeJournalpostMedUtsending {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    avsender: Option<String>,
    mottaker: Vec<UtsendingMottaker>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettInngåendeJournalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    avsender: String,
    mottaker: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettInterntNotatJournalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JournalpostCommon {
    pub client_reference: Uuid,
    pub tittel: String,
    pub dokument_dato: String,
    pub saksbehandler: String,
    pub saksbehandler_enhet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilgang: Option<Tilgang>,
    /// Første dokument i lista er hoveddokument
    pub dokumenter: Vec<Dokument>,
    pub sak_key: SakKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kildesystem: Option<String>,
}
