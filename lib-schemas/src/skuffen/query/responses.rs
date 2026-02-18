use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::journalpost::{JournalpostId, JournalpostType, Journalpoststatus};
use crate::skuffen::sak::{Ordningsverdi, Saksnummer, Saksstatus, Sakstittel};
use crate::skuffen::tilgang::Tilgang;

/// Response payload for en sak.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SakResponse {
    pub sakstittel: Sakstittel,
    pub saksbehandler: Option<String>,
    pub saksbehandler_enhet: Option<String>,
    pub saksstatus: Saksstatus,
    pub tilgang: Option<Tilgang>,
    pub ordningsverdi: Ordningsverdi,
    pub saksnummer: Saksnummer,
    pub kildesystem: String,
    pub lukket: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journalposter: Option<Vec<JournalpostResponse>>,
}

/// Response metadata for dokument.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct DokumentResponse {
    pub tittel: String,
    pub filtype: String,
    pub dokument_referanse: Option<Uuid>,
}

/// Response payload for en journalpost.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JournalpostResponse {
    pub tittel: String,
    pub dokument_dato: String, // TODO: Denne skal være datetime
    pub journalposttype: JournalpostType,
    pub journalstatus: Journalpoststatus,
    pub tilgang: Option<Tilgang>,
    pub saksbehandler: Option<String>,
    pub saksbehandler_enhet: Option<String>,
    pub dokumenter: Vec<DokumentResponse>,
    pub journalpost_id: JournalpostId,
    pub kildesystem: String,
}
