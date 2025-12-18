use crate::{
    error::{Result, SchemasError},
    skuffen::tilgang::Tilgang,
};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

use crate::skuffen::journalpost::JournalpostResponse;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SakResponse {
    pub sakstittel: Sakstittel, //TODO: Det er egene regler for sakstittel, spesielt ved skjerming. Lag
    //en en egen type for denne med validering i new()
    pub saksbehandler: String,
    pub saksstatus: Saksstatus,
    pub tilgang: Tilgang,
    pub ordningsverdi: Ordningsverdi,
    pub sak_key: SakKeyResponse,
    pub kildesystem: String,
    pub lukket: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journalposter: Option<Vec<JournalpostResponse>>,
}

/**
* SaksTittel benyttes på opprettelse av sak i arkiv
*/
const SAKSTITTEL_MAX_LENGTH: usize = 256;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Sakstittel(pub String);

impl Sakstittel {
    pub fn uo_tittel(&self) -> Sakstittel {
        Sakstittel("*****".to_string())
    }
}

impl FromStr for Sakstittel {
    type Err = SchemasError;

    fn from_str(s: &str) -> Result<Self> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(SakstittelError::Empty.into());
        }

        if trimmed.len() > SAKSTITTEL_MAX_LENGTH {
            return Err(SakstittelError::TooLong.into());
        }

        Ok(Sakstittel(trimmed.to_string()))
    }
}

impl TryFrom<&str> for Sakstittel {
    type Error = SchemasError;
    fn try_from(value: &str) -> Result<Self> {
        value.parse()
    }
}

impl TryFrom<String> for Sakstittel {
    type Error = SchemasError;
    fn try_from(value: String) -> Result<Self> {
        value.as_str().parse()
    }
}

impl fmt::Display for Sakstittel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Ordningsverdi(String);

impl Ordningsverdi {
    pub fn new(&self, s: String) -> Result<Self> {
        // non-empty
        if s.is_empty() {
            return Err(SchemasError::ParseError(
                "string is empty".to_string().into(),
            ));
        }

        // only digits or '-'
        if !s.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return Err(SchemasError::ParseError(
                format!("invalid character in '{s}'").into(),
            ));
        }

        // max 1 '-'
        let hyphen_count = s.chars().filter(|&c| c == '-').count();
        if hyphen_count > 1 {
            return Err(SchemasError::ParseError(
                "more than one '-' found".to_string().into(),
            ));
        }

        Ok(Ordningsverdi(s))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SakKeyResponse {
    skuffen_id: Uuid,
    arkiv_id: Saksnummer,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum SakstittelError {
    Empty,
    TooLong,
}

impl fmt::Display for SakstittelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SakstittelError::Empty => write!(f, "Sakstittel er tom."),
            SakstittelError::TooLong => write!(
                f,
                "Sakstittel er for lang. Max lengde: {SAKSTITTEL_MAX_LENGTH}"
            ),
        }
    }
}

impl std::error::Error for SakstittelError {}

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
