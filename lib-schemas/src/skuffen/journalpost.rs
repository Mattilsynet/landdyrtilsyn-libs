use crate::error::{Result, SchemasError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::typer::{organisasjonsnummer::Organisasjonsnummer, personnummer::Personnummer};

/// Identifier for journalposter lagret i arkivet.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct JournalpostId(pub String);

/// Keys for å hente journalposter.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum JournalpostKey {
    ClientReference(Uuid),
    JournalpostId(JournalpostId),
}

/// Recipient definition brukt ved utsending.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct UtsendingMottaker {
    pub navn: String,
    pub adresse: String,
    pub postnummer: String,
    pub poststed: String,
    pub id: MottakerId,
}

/// Recipient identifier (person eller organization).
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum MottakerId {
    Person {
        personnummer: Personnummer,
    },
    Organisasjon {
        organisasjonsnummer: Organisasjonsnummer,
    },
}

/// Sender wrapper med flattened fields.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Avsender {
    #[serde(flatten)]
    avsender_mottaker: AvsenderMottaker,
}

/// Recipient wrapper med flattened fields.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mottaker {
    #[serde(flatten)]
    avsender_mottaker: AvsenderMottaker,
}

//TODO: Håndtere kopi

/// Common sender/recipient fields for journalposter.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct AvsenderMottaker {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adresse: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postnummer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poststed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub land: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_type: Option<String>,
}

/// Journalpost types mappet til archive codes.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum JournalpostType {
    Inngående,
    Utgående,
    InterntNotat,
}

/// Journalpost status values mappet til archive codes.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum Journalpoststatus {
    Registrert,
    Reservert,
    Midlertidig,
    Ferdig,
    Ekspedert,
    Journalført,
}

impl JournalpostId {
    /// Returner raw journalpost id string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl JournalpostType {
    /// Returner external code representation.
    pub fn code(self) -> char {
        match self {
            JournalpostType::Inngående => 'I',
            JournalpostType::Utgående => 'U',
            JournalpostType::InterntNotat => 'X',
        }
    }

    /// Parse fra external code representation.
    pub fn from_char(c: char) -> Result<Self> {
        let journalpost_type = match c {
            'I' => Self::Inngående,
            'U' => Self::Utgående,
            'X' => Self::InterntNotat,
            _ => {
                return Err(SchemasError::ParseError(
                    format!("Ukjent JournalpostType: {c}").into(),
                ));
            }
        };
        Ok(journalpost_type)
    }
}

impl Journalpoststatus {
    /// Returner external code representation.
    pub fn code(self) -> char {
        match self {
            Journalpoststatus::Registrert => 'S',
            Journalpoststatus::Reservert => 'R',
            Journalpoststatus::Midlertidig => 'M',
            Journalpoststatus::Ferdig => 'F',
            Journalpoststatus::Ekspedert => 'E',
            Journalpoststatus::Journalført => 'J',
        }
    }

    /// Parse fra external code representation.
    pub fn from_char(c: char) -> Result<Self> {
        let journalpoststatus = match c {
            'S' => Self::Registrert,
            'R' => Self::Reservert,
            'M' => Self::Midlertidig,
            'F' => Self::Ferdig,
            'E' => Self::Ekspedert,
            'J' => Self::Journalført,
            _ => {
                return Err(SchemasError::ParseError(
                    format!("Ukjent Journalpoststatus: {c}").into(),
                ));
            }
        };
        Ok(journalpoststatus)
    }
}
