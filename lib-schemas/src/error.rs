use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::skuffen::sak::{SaksnummerError, SakstittelError};
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = core::result::Result<T, SchemasError>;

#[derive(Error, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum SchemasError {
    #[error("Validation Error error in {0}")]
    ValidationError(String),
    #[error("Parse Error error in {0}")]
    ParseError(#[from] ParseError),
}

#[derive(Error, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ParseError {
    #[error("Sakstittel Error in {0}")]
    Sakstittel(#[from] SakstittelError),
    #[error("Saksnummer Error in {0}")]
    Saksnummer(#[from] SaksnummerError),
    #[error("{0}")]
    Message(String),
}

impl From<String> for ParseError {
    fn from(s: String) -> Self {
        ParseError::Message(s)
    }
}

impl From<SaksnummerError> for SchemasError {
    fn from(err: SaksnummerError) -> Self {
        SchemasError::ParseError(ParseError::Saksnummer(err))
    }
}

impl From<SakstittelError> for SchemasError {
    fn from(err: SakstittelError) -> Self {
        SchemasError::ParseError(ParseError::Sakstittel(err))
    }
}
