use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::typer::{organisasjonsnummer::Organisasjonsnummer, personnummer::Personnummer};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournalpostId(String);

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JournalpostResponse {
    pub tittel: String,
    pub dokument_dato: DateTime<Utc>,
    pub journalposttype: JournalpostType,
    pub journalstatus: Journalstatus,
    pub unntatt_offentlighet: bool,

    pub saksbehandler: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dokumenter: Option<Vec<Dokument>>,
    pub journalpost_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kildesystem: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JournalpostCommon {
    pub tittel: String,
    pub dokument_dato: DateTime<Utc>,
    pub journalposttype: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilgangskode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilgangshjemmel: Option<String>,
    pub saksbehandler: String,
    pub saksbehandler_enhet: String,
    pub dokumenter: Vec<Dokument>,
    pub journalpost_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kildesystem: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettUgåendeJurnalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    avsender: Option<String>,
    mottaker: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettUgåendeJurnalpostMedUtsending {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    avsender: Option<String>,
    mottaker: Vec<UtsendingMottaker>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettInngåendeJurnalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
    avsender: String,
    mottaker: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettInterntNotatJurnalpost {
    #[serde(flatten)]
    pub felles: JournalpostCommon,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Dokument {
    pub tittel: String,
    pub filtype: String,
    pub dokument_referanse: Uuid,
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
pub enum Journalstatus {
    Registrert,
    Reservert,
    Midlertidig,
    Ferdig,
    Ekspedert,
    Journalført,
}

impl Journalstatus {
    pub fn code(self) -> char {
        match self {
            Journalstatus::Registrert => 'S',
            Journalstatus::Reservert => 'R',
            Journalstatus::Midlertidig => 'M',
            Journalstatus::Ferdig => 'F',
            Journalstatus::Ekspedert => 'E',
            Journalstatus::Journalført => 'J',
        }
    }

    pub fn from_code(c: char) -> Option<Self> {
        match c {
            'S' => Some(Self::Registrert),
            'R' => Some(Self::Reservert),
            'M' => Some(Self::Midlertidig),
            'F' => Some(Self::Ferdig),
            'E' => Some(Self::Ekspedert),
            'J' => Some(Self::Journalført),
            _ => None,
        }
    }
}
