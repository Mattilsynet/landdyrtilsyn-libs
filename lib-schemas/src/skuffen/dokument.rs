use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifier for dokumenter lagret i arkivet.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DokumentId(pub String);

impl DokumentId {
    /// Returner raw dokument id string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Document metadata brukt i journalposter.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Dokument {
    /// Client side correlation for idempotency.
    pub client_reference: Uuid,
    /// Tittel for dokumentet.
    pub tittel: String,
    /// File type/extension (e.g. "pdf").
    pub filtype: String,
    /// Archive reference til det lagrede dokumentet.
    pub dokument_referanse: Uuid,
}
