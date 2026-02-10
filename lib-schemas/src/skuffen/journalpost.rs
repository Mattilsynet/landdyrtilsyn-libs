use crate::{
    error::{Result, SchemasError},
    skuffen::tilgang::Tilgang,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    skuffen::dokument::DokumentResponse,
    typer::{organisasjonsnummer::Organisasjonsnummer, personnummer::Personnummer},
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct JournalpostId(String);

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JournalpostResponse {
    pub client_reference: Option<Uuid>,
    pub tittel: String,
    pub dokument_dato: String, //TODO: Denne skal være datetime
    pub journalposttype: JournalpostType,
    pub journalstatus: Journalpoststatus,
    pub tilgang: Option<Tilgang>,
    pub saksbehandler: String,
    pub dokumenter: Vec<DokumentResponse>,
    pub journalpost_id: i32,
    pub kildesystem: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum JournalpostKey {
    ClientReference(Uuid),
    JournalpostId(JournalpostId),
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct UtsendingMottaker {
    pub navn: String,
    pub adresse: String,
    pub postnummer: String,
    pub poststed: String,
    pub id: MottakerId,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum MottakerId {
    Person {
        personnummer: Personnummer,
    },
    Organisasjon {
        organisasjonsnummer: Organisasjonsnummer,
    },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Avsender {
    #[serde(flatten)]
    avsender_mottaker: AvsenderMottaker,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mottaker {
    #[serde(flatten)]
    avsender_mottaker: AvsenderMottaker,
}

//TODO: Håndtere kopi

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

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum JournalpostType {
    Inngående,
    Utgående,
    InterntNotat,
}

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
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl JournalpostType {
    pub fn code(self) -> char {
        match self {
            JournalpostType::Inngående => 'I',
            JournalpostType::Utgående => 'U',
            JournalpostType::InterntNotat => 'X',
        }
    }

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
