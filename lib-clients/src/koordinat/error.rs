use thiserror::Error;

pub type Result<T> = core::result::Result<T, GeonorgeError>;

#[derive(Error, Debug)]
pub enum GeonorgeError {
    #[error("HTTP request failed: {0}")]
    RequestError(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("No results found for address: {0}")]
    NoResults(String),

    #[error("Invalid address format: {0}")]
    InvalidAddress(String),

    #[error("API error: {0}")]
    ApiError(String),
}
