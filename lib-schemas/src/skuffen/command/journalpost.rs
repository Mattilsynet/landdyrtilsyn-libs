use serde::{Deserialize, Serialize};

use crate::skuffen::journalpost::{JournalpostCommon, UtsendingMottaker};

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
