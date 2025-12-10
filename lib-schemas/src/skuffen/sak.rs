use crate::error::{Result, SchemasError};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::skuffen::journalpost::JournalpostResponse;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SakResponse {
    pub sakstittel: String,
    pub saksbehandler: String,
    pub saksstatus: Saksstatus,
    pub unntatt_offentlighet: bool,
    pub saksnr: Saksnummer,
    pub kildesystem: String,
    pub lukket: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journalposter: Option<Vec<JournalpostResponse>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum SakKey {
    SkuffenId(Uuid),
    ArkivId(Saksnummer),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Saksstatus {
    UnderBehandling,
    Ferdig,
    Avsluttet,
}

impl Saksstatus {
    pub fn code(self) -> char {
        match self {
            Saksstatus::UnderBehandling => 'B',
            Saksstatus::Ferdig => 'F',
            Saksstatus::Avsluttet => 'A',
        }
    }

    pub fn from_char(c: char) -> Result<Self> {
        let saksstatus = match c {
            'B' => Self::UnderBehandling,
            'F' => Self::Ferdig,
            'A' => Self::Avsluttet,
            _ => {
                return Err(SchemasError::ParseError(
                    format!("Ukjent saksstatus: {c}").into(),
                ));
            }
        };
        Ok(saksstatus)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Saksnummer(String);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum SaksnummerError {
    UgyldigFormat,
    UgyldigÅr(u16),
    ManglerSekvensnummer,
}

impl fmt::Display for SaksnummerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SaksnummerError::UgyldigFormat => write!(f, "must be formatted as <YYYY>/<seq>"),
            SaksnummerError::UgyldigÅr(y) => write!(
                f,
                "invalid year {y}; expected a 4-digit year in [1000, 9999]"
            ),
            SaksnummerError::ManglerSekvensnummer => write!(f, "missing sequence after slash"),
        }
    }
}

impl std::error::Error for SaksnummerError {}

impl Saksnummer {
    /// Construct from separate year and sequence parts.
    /// The result will be formatted as "YYYY/<seq>"
    pub fn new_from_parts(year: u16, sequence: impl AsRef<str>) -> Result<Self> {
        if !(1000..=9999).contains(&year) {
            return Err(SaksnummerError::UgyldigÅr(year).into());
        }
        let seq = sequence.as_ref();
        if seq.is_empty() {
            return Err(SaksnummerError::ManglerSekvensnummer.into());
        }
        Ok(Self(format!("{year}/{seq}")))
    }

    /// Construct from a string of the form "YYYY/<seq>".
    /// - Year must be 4 digits and valid.
    /// - Sequence can be any non-empty string.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s = s.into();
        let parts: Vec<&str> = s.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(SaksnummerError::UgyldigFormat.into());
        }

        let year_str = parts[0];
        let seq_str = parts[1];

        let year: u16 = year_str
            .parse()
            .map_err(|_| SaksnummerError::UgyldigFormat)?;
        if !(1000..=9999).contains(&year) {
            return Err(SaksnummerError::UgyldigÅr(year).into());
        }

        if seq_str.is_empty() {
            return Err(SaksnummerError::ManglerSekvensnummer.into());
        }

        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn year(&self) -> u16 {
        self.0[0..4].parse().expect("validated year")
    }

    pub fn sequence(&self) -> &str {
        &self.0[5..]
    }
}

impl fmt::Display for Saksnummer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_parts_valid() {
        let s = Saksnummer::new_from_parts(2025, "ABC123").unwrap();
        assert_eq!(s.as_str(), "2025/ABC123");
        assert_eq!(s.year(), 2025);
        assert_eq!(s.sequence(), "ABC123");
    }

    #[test]
    fn from_parts_rejects_invalid_year_or_empty_seq() {
        assert_eq!(
            Saksnummer::new_from_parts(999, "foo").unwrap_err(),
            SaksnummerError::UgyldigÅr(999).into()
        );
        assert_eq!(
            Saksnummer::new_from_parts(2025, "").unwrap_err(),
            SaksnummerError::ManglerSekvensnummer.into()
        );
    }

    #[test]
    fn from_string_valid_formats() {
        let s = Saksnummer::new("2025/123456").unwrap();
        assert_eq!(s.year(), 2025);
        assert_eq!(s.sequence(), "123456");

        let s = Saksnummer::new("2025/ABC-XYZ").unwrap();
        assert_eq!(s.sequence(), "ABC-XYZ");

        let s = Saksnummer::new("2025/foo_bar").unwrap();
        assert_eq!(s.sequence(), "foo_bar");
    }

    #[test]
    fn from_string_invalid_formats() {
        assert_eq!(
            Saksnummer::new("2025").unwrap_err(),
            SaksnummerError::UgyldigFormat.into()
        );
        assert_eq!(
            Saksnummer::new("20a5/123").unwrap_err(),
            SaksnummerError::UgyldigFormat.into()
        );
        assert_eq!(
            Saksnummer::new("2025/").unwrap_err(),
            SaksnummerError::ManglerSekvensnummer.into()
        );
        assert_eq!(
            Saksnummer::new("999/abc").unwrap_err(),
            SaksnummerError::UgyldigÅr(999).into()
        );
    }

    #[test]
    fn display_roundtrip() {
        let s: Saksnummer = Saksnummer::new("2025/custom-seq").unwrap();
        assert_eq!(s.to_string(), "2025/custom-seq");
    }
}
