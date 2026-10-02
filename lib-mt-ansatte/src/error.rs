use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid id: {0}")]
    InvalidId(String),
    #[error("stream lookup failed: {0}")]
    Stream(#[from] async_nats::jetstream::context::GetStreamError),
    #[error("message lookup failed: {0}")]
    Message(#[from] async_nats::jetstream::stream::LastRawMessageError),
    #[error("payload deserialization failed: {0}")]
    Payload(#[from] serde_json::Error),
}
