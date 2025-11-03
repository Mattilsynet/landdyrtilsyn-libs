use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SakResponse {
    pub sakstittel: String,
    pub saksbehandler: String,
    pub saksstatus: String,
    pub unntatt_offentlighet: bool,
    pub saksnr: String,
    pub lukket: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Saksnummer(String);

#[derive(Debug, PartialEq, Eq)]
pub enum SaksnummerError {
    InvalidFormat,
    InvalidYear(u16),
    MissingSequence,
}

impl fmt::Display for SaksnummerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SaksnummerError::InvalidFormat => write!(f, "must be formatted as <YYYY>/<seq>"),
            SaksnummerError::InvalidYear(y) => write!(
                f,
                "invalid year {y}; expected a 4-digit year in [1000, 9999]"
            ),
            SaksnummerError::MissingSequence => write!(f, "missing sequence after slash"),
        }
    }
}

impl std::error::Error for SaksnummerError {}

impl Saksnummer {
    /// Construct from separate year and sequence parts.
    /// The result will be formatted as "YYYY/<seq>"
    pub fn new_from_parts(year: u16, sequence: impl AsRef<str>) -> Result<Self, SaksnummerError> {
        if !(1000..=9999).contains(&year) {
            return Err(SaksnummerError::InvalidYear(year));
        }
        let seq = sequence.as_ref();
        if seq.is_empty() {
            return Err(SaksnummerError::MissingSequence);
        }
        Ok(Self(format!("{year}/{seq}")))
    }

    /// Construct from a string of the form "YYYY/<seq>".
    /// - Year must be 4 digits and valid.
    /// - Sequence can be any non-empty string.
    pub fn new(s: impl Into<String>) -> Result<Self, SaksnummerError> {
        let s = s.into();
        let parts: Vec<&str> = s.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(SaksnummerError::InvalidFormat);
        }

        let year_str = parts[0];
        let seq_str = parts[1];

        if year_str.len() != 4 || !year_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(SaksnummerError::InvalidFormat);
        }

        let year: u16 = year_str
            .parse()
            .map_err(|_| SaksnummerError::InvalidFormat)?;
        if !(1000..=9999).contains(&year) {
            return Err(SaksnummerError::InvalidYear(year));
        }

        if seq_str.is_empty() {
            return Err(SaksnummerError::MissingSequence);
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

impl FromStr for Saksnummer {
    type Err = SaksnummerError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Saksnummer::new(s)
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
            SaksnummerError::InvalidYear(999)
        );
        assert_eq!(
            Saksnummer::new_from_parts(2025, "").unwrap_err(),
            SaksnummerError::MissingSequence
        );
    }

    #[test]
    fn from_string_valid_formats() {
        let s = Saksnummer::new("2025/123456").unwrap();
        assert_eq!(s.year(), 2025);
        assert_eq!(s.sequence(), "123456");

        let s = Saksnummer::new("2025/ABC-XYZ").unwrap();
        assert_eq!(s.sequence(), "ABC-XYZ");

        let s = "2025/foo_bar".parse::<Saksnummer>().unwrap();
        assert_eq!(s.sequence(), "foo_bar");
    }

    #[test]
    fn from_string_invalid_formats() {
        assert_eq!(
            Saksnummer::new("2025").unwrap_err(),
            SaksnummerError::InvalidFormat
        );
        assert_eq!(
            Saksnummer::new("20a5/123").unwrap_err(),
            SaksnummerError::InvalidFormat
        );
        assert_eq!(
            Saksnummer::new("2025/").unwrap_err(),
            SaksnummerError::MissingSequence
        );
        assert_eq!(
            Saksnummer::new("999/abc").unwrap_err(),
            SaksnummerError::InvalidYear(999)
        );
    }

    #[test]
    fn display_roundtrip() {
        let s: Saksnummer = "2025/custom-seq".parse().unwrap();
        assert_eq!(s.to_string(), "2025/custom-seq");
    }
}
