use std::fmt::{self};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MissingEnvVariable(String),
    MissingTokenOnRequest(String),

    InternalError(Box<dyn std::error::Error + Send + Sync>),
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
