use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::journalpost::JournalpostKey;
use crate::skuffen::sak::Saksnummer;

/// Query payloads for Skuffen.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Query {
    HentSak(HentSakQuery),
    HentSakMedJournalposter(HentSakMedJournalposterQuery),
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

/// Query for å hente sak med journalposter og dokumentmetadata, uten
/// dokumentinnhold.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HentSakMedJournalposterQuery {
    pub key: SakKey,
}

/// Key for å hente sak basert på client reference eller arkiv id.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum SakKey {
    ClientReference(Uuid),
    ArkivId(Saksnummer),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hent_sak_med_journalposter_query_har_samme_key_shape_som_hent_sak() {
        let value = json!({ "key": { "type": "arkivId", "value": "2026/12345" } });

        let query: HentSakMedJournalposterQuery = serde_json::from_value(value.clone()).unwrap();

        assert_eq!(
            query.key,
            SakKey::ArkivId(Saksnummer::new("2026/12345").unwrap())
        );
        assert_eq!(serde_json::to_value(&query).unwrap(), value);
    }
}
