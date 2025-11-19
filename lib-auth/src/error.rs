use std::fmt::{self};

use http::StatusCode;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MissingEnvVariable(String),
    MissingTokenOnRequest,
    JwtError(jsonwebtoken::errors::Error),
    ReqwestError(reqwest::Error),
    InternalError(Box<dyn std::error::Error + Send + Sync>),
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::MissingEnvVariable(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::MissingTokenOnRequest => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::JwtError(_) => StatusCode::UNAUTHORIZED.into_response(),
            Error::ReqwestError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
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
        write!(f, "{self}")
    }
}
