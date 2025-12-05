use thiserror::Error;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = core::result::Result<T, ApiError>;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed")]
    RequestError(#[from] reqwest::Error),
    #[error("Received non-200 response: {0}")]
    Non200Response(String),
    #[error("Failed to get token: {0}")]
    TokenError(String),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    #[error("Client error in {resource}: {error_message}")]
    ClientError {
        resource: String,
        error_message: String,
    },
    #[error("Validation Error error in {0}")]
    ValidationError(String),
    #[error("Authentication error: {error_message}")]
    AuthError { error_message: String },
}
