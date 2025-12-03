use serde::{Deserialize, Serialize};

use crate::skuffen::journalpost::{JournalpostCommon, UtsendingMottaker};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettUgåendeJurnalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    avsender: Option<String>,
    mottaker: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettUgåendeJurnalpostMedUtsending {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    avsender: Option<String>,
    mottaker: Vec<UtsendingMottaker>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettInngåendeJurnalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    avsender: String,
    mottaker: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettInterntNotatJurnalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
}
