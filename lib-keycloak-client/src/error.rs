use thiserror::Error;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed")]
    RequestError(#[from] reqwest::Error),
    #[error("Received non-200 response: {0}")]
    Non200Response(String),
    #[error("Failed to get token: {0}")]
    TokenError(String),
}
