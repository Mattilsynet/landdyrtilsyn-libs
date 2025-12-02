use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct DokumentResponse {
    pub tittel: String,
    pub filtype: String,
    pub dokument_referanse: Option<Uuid>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Dokument {
    pub tittel: String,
    pub filtype: String,
    pub dokument_referanse: Uuid,
}
