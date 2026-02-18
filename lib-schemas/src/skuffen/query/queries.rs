use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::journalpost::JournalpostKey;
use crate::skuffen::sak::Saksnummer;

/// Query payloads for Skuffen.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Query {
    HentSak(HentSakQuery),
    HentJournalpost(HentJournalpostQuery),
}

/// Query for å hente journalpost.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HentJournalpostQuery {
    pub key: JournalpostKey,
}

/// Query for å hente sak.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HentSakQuery {
    pub key: SakKey,
}

/// Key for å hente sak basert på client reference eller arkiv id.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum SakKey {
    ClientReference(Uuid),
    ArkivId(Saksnummer),
}
