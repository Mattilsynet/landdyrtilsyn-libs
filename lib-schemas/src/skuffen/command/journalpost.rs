use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::{
    dokument::Dokument, journalpost::UtsendingMottaker, query::queries::SakKey, tilgang::Tilgang,
};

/// Lag en outgoing journalpost.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettUgåendeJournalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    pub avsender: Option<String>,
    pub mottaker: String,
}

/// Lag en outgoing journalpost med multiple recipients.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettUgåendeJournalpostMedUtsending {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    pub avsender: Option<String>,
    pub mottaker: Vec<UtsendingMottaker>,
}

/// Lag en incoming journalpost.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettInngåendeJournalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    pub avsender: String,
    pub mottaker: Option<String>,
}

/// Lag en internal note journalpost.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettInterntNotatJournalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
}

/// Fields shared av alle journalpost creation commands.
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
    /// Første dokument i lista er hoveddokument.
    pub dokumenter: Vec<Dokument>,
    pub sak_key: SakKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kildesystem: Option<String>,
}
