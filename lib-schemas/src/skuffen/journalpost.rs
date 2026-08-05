use crate::error::{ParseError, Result, SchemasError};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct JournalpostId(pub String);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum JournalpostKey {
    ClientReference(Uuid),
    JournalpostId(JournalpostId),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct Postnummer(String);

impl Postnummer {
    pub fn new(postnummer: impl Into<String>) -> Result<Self> {
        let postnummer = postnummer.into();
        if postnummer.is_empty() {
            return Err(SchemasError::ParseError(ParseError::Message(
                "postnummer er tomt".to_string(),
            )));
        }
        if postnummer.len() != 4 || !postnummer.chars().all(|c| c.is_ascii_digit()) {
            return Err(SchemasError::ParseError(ParseError::Message(format!(
                "ugyldig postnummer '{postnummer}'; forventet 4 siffer"
            ))));
        }
        Ok(Self(postnummer))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Postnummer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Postnummer {
    type Error = SchemasError;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl From<Postnummer> for String {
    fn from(value: Postnummer) -> Self {
        value.0
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn postnummer_validering_og_bare_string_serde() {
        assert!(Postnummer::new("").is_err());
        assert!(Postnummer::new("123").is_err());
        assert!(Postnummer::new("12345").is_err());
        assert!(Postnummer::new("12a4").is_err());
        let p = Postnummer::new("0350").unwrap();
        assert_eq!(p.as_str(), "0350");
        assert_eq!(p.to_string(), "0350");
        assert_eq!(serde_json::to_value(&p).unwrap(), json!("0350"));
        let back: Postnummer = serde_json::from_value(json!("0350")).unwrap();
        assert_eq!(back, p);
        assert!(serde_json::from_value::<Postnummer>(json!("12")).is_err());
    }
}
