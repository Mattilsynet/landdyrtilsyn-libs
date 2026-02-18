use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
