use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod sak;
pub mod tilgangshjemmel;
pub mod tilgangskoder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Landkode(pub String);

#[derive(Debug)]
pub enum LandkodeError {
    InvalidLength,
    InvalidCharacters,
}

impl FromStr for Landkode {
    type Err = LandkodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(LandkodeError::InvalidLength);
        }

        if !s.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(LandkodeError::InvalidCharacters);
        }

        Ok(Landkode(s.to_string()))
    }
}

impl fmt::Display for Landkode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Landkode {
    pub fn new(s: &str) -> Result<Self, LandkodeError> {
        s.parse()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Saksaar(pub String);

impl fmt::Display for Saksaar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum SaksaarError {
    InvalidLength,
    InvalidCharacters,
}

impl FromStr for Saksaar {
    type Err = SaksaarError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(SaksaarError::InvalidLength);
        }

        if !s.chars().all(|c| c.is_ascii_digit()) {
            return Err(SaksaarError::InvalidCharacters);
        }

        Ok(Saksaar(s.to_string()))
    }
}

impl Saksaar {
    pub fn new(s: &str) -> Result<Self, SaksaarError> {
        s.parse()
    }
}

/**
* SaksTittel benyttes på opprettelse av sak i arkiv
*/
const SAKSTITTEL_MAX_LENGTH: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaksTittel(pub String);

#[derive(Debug)]
pub enum SaksTittelError {
    Empty,
    TooLong,
}

impl FromStr for SaksTittel {
    type Err = SaksTittelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(SaksTittelError::Empty);
        }

        if trimmed.len() > SAKSTITTEL_MAX_LENGTH {
            return Err(SaksTittelError::TooLong);
        }

        Ok(SaksTittel(trimmed.to_string()))
    }
}

impl fmt::Display for SaksTittel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
