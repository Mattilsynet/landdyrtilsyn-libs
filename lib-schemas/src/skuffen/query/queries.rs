use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::journalpost::JournalpostKey;
use crate::skuffen::sak::Saksnummer;

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
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum SakKey {
    ClientReference(Uuid),
    ArkivId(Saksnummer),
}
