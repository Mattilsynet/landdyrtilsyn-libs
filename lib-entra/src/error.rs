use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum EntraError {
    #[error("network error: {0}")]
    Network(String),
    #[error("unauthorized (token invalid or expired)")]
    Unauthorized,
    #[error("forbidden (insufficient permissions)")]
    Forbidden,
    #[error("unexpected status {status}: {body}")]
    UnexpectedResponse { status: StatusCode, body: String },
    #[error("deserialize error: {0}")]
    Deserialize(String),
    #[error("obo exchange failed: {0}")]
    Obo(String),
    #[error("missing env var: {0}")]
    MissingEnv(String),
}
