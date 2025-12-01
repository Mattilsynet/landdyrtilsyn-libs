use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::typer::{organisasjonsnummer::Organisasjonsnummer, personnummer::Personnummer};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct JournalpostId(String);

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JournalpostResponse {
    pub tittel: String,
    pub dokument_dato: DateTime<Utc>,
    pub journalposttype: JournalpostType,
    pub journalstatus: Journalpoststatus,
    pub unntatt_offentlighet: bool,
    pub saksbehandler: String,
    pub dokumenter: Vec<Dokument>,
    pub journalpost_id: i32,
    pub kildesystem: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JournalpostCommon {
    pub tittel: String,
    pub dokument_dato: DateTime<Utc>,
    pub journalposttype: JournalpostType,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct HentJournalpostRequest {
    pub key: JournalpostKey,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum JournalpostKey {
    SkuffenId(Uuid),
    ArkivId(JournalpostId),
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

    pub fn from_string(s: String) -> Option<Self> {
        let mut chars = s.chars();
        let c = chars.next()?;
        if chars.next().is_some() {
            return None;
        }

        match c {
            'I' => Some(Self::Inngående),
            'U' => Some(Self::Utgående),
            'X' => Some(Self::InterntNotat),
            _ => None,
        }
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

    pub fn from_string(s: String) -> Option<Self> {
        let mut chars = s.chars();
        let c = chars.next()?;
        if chars.next().is_some() {
            return None;
        }

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
