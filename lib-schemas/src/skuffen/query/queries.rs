use serde::{Deserialize, Serialize};

use crate::skuffen::{journalpost::JournalpostKey, sak::SakKey};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Query {
    HentSak(HentSakQuery),
    HentJournalpost(HentJournalpostQuery),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HentJournalpostQuery {
    pub key: JournalpostKey,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HentSakQuery {
    pub key: SakKey,
    pub inkluder_journalposter: bool,
}
