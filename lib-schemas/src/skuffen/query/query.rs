use serde::{Deserialize, Serialize};

use crate::skuffen::{journalpost::JournalpostKey, sak::SakKey};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Query {
    HentSak(HentSakQuery),
    HentJournalpost(HentJournalpostQuery),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct HentJournalpostQuery {
    pub key: JournalpostKey,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct HentSakQuery {
    pub key: SakKey,
    pub inkluder_journalposter: bool,
}
