use thiserror::Error;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = core::result::Result<T, SchemasError>;

#[derive(Error, Debug)]
pub enum SchemasError {
    #[error("Validation Error error in {0}")]
    ValidationError(String),
}
