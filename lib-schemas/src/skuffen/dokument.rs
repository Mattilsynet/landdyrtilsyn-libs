use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct DokumentResponse {
    pub client_reference: Option<Uuid>,
    pub tittel: String,
    pub filtype: String,
    pub dokument_referanse: Option<Uuid>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Dokument {
    pub client_reference: Uuid,
    pub tittel: String,
    pub filtype: String,
    pub dokument_referanse: Uuid,
}
