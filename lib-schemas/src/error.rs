use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "skuffen")]
use crate::skuffen::sak::{SaksnummerError, SakstittelError};

/// Convenience alias for boxed errors, kompatibel med async boundaries.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Result type brukt på tvers av schema crate.
pub type Result<T> = core::result::Result<T, SchemasError>;

/// Errors produsert av validation eller parsing av schema types.
#[derive(Error, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum SchemasError {
    /// Validation errors som ikke nodvendigvis betyr parse failure.
    #[error("Validation Error error in {0}")]
    ValidationError(String),
    /// Parsing errors for domain-specific format.
    #[error("Parse Error error in {0}")]
    ParseError(#[from] ParseError),
}

/// Parse errors for spesifikke schema fields.
#[derive(Error, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ParseError {
    /// Invalid sakstittel value.
    #[cfg(feature = "skuffen")]
    #[error("Sakstittel Error in {0}")]
    Sakstittel(#[from] SakstittelError),
    /// Invalid saksnummer value.
    #[cfg(feature = "skuffen")]
    #[error("Saksnummer Error in {0}")]
    Saksnummer(#[from] SaksnummerError),
    /// Generic parse error message.
    #[error("{0}")]
    Message(String),
}

impl From<String> for ParseError {
    fn from(s: String) -> Self {
        ParseError::Message(s)
    }
}

#[cfg(feature = "skuffen")]
impl From<SaksnummerError> for SchemasError {
    fn from(err: SaksnummerError) -> Self {
        SchemasError::ParseError(ParseError::Saksnummer(err))
    }
}

#[cfg(feature = "skuffen")]
impl From<SakstittelError> for SchemasError {
    fn from(err: SakstittelError) -> Self {
        SchemasError::ParseError(ParseError::Sakstittel(err))
    }
}
