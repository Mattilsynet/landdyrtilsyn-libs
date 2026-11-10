use crate::error::ApiError;
use reqwest::StatusCode;
use thiserror::Error;

pub type HrngResult<T> = std::result::Result<T, HrngError>;

/// Response bodies and serde messages are deliberately left out: they can
/// contain personal data.
#[derive(Error, Debug)]
pub enum HrngError {
    #[error("lookup returned http status {status}")]
    Http { status: StatusCode },
    #[error("lookup was rejected as unauthorized, also after token refresh")]
    Auth,
    #[error("lookup request failed")]
    Request(#[source] ApiError),
    #[error("failed to parse lookup response at {path}")]
    Parse { path: String },
}
