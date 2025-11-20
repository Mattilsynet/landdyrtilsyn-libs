use axum::http::StatusCode;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MissingEnvVariable(String),
    MissingTokenOnRequest,
    JwtError(jsonwebtoken::errors::Error),
    ReqwestError(reqwest::Error),
    InternalError(Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::MissingEnvVariable(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::MissingTokenOnRequest => StatusCode::UNAUTHORIZED,
            Error::JwtError(_) => StatusCode::UNAUTHORIZED,
            Error::ReqwestError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        self.status_code().into_response()
    }
}

impl<E> From<E> for Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        Self::InternalError(Box::new(err))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingEnvVariable(var) => write!(f, "Missing env variable: {var}"),
            Error::MissingTokenOnRequest => write!(f, "Missing bearer token on request"),
            Error::JwtError(e) => write!(f, "JWT error: {e}"),
            Error::ReqwestError(e) => write!(f, "HTTP client error: {e}"),
            Error::InternalError(e) => write!(f, "Internal error: {e}"),
        }
    }
}
