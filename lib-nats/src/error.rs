use std::env::VarError;

use async_nats::{
    ConnectErrorKind,
    jetstream::{
        consumer::pull::BatchErrorKind,
        context::{CreateKeyValueErrorKind, GetStreamErrorKind},
        stream::ConsumerErrorKind,
    },
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConnectionError(String),
    ConfigError(String),
    ConsumerError(String),
    StreamError(String),
    FetchError(String),
    NotFoundError(String),
    PublishError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConnectionError(e) => write!(f, "Failed to connect to NATS: {}", e),
            Error::ConfigError(e) => write!(f, "Failed to configure NATS client: {}", e),
            Error::ConsumerError(e) => write!(f, "Failed to create NATS consumer: {}", e),
            Error::StreamError(e) => write!(f, "Failed to get NATS stream: {}", e),
            Error::FetchError(e) => write!(f, "Failed to fetch message from NATS: {}", e),
            Error::NotFoundError(e) => write!(f, "Message not found {}", e),
            Error::PublishError(e) => write!(f, "Failed to publish: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<async_nats::error::Error<async_nats::jetstream::context::CreateStreamErrorKind>>
    for Error
{
    fn from(
        value: async_nats::error::Error<async_nats::jetstream::context::CreateStreamErrorKind>,
    ) -> Self {
        Self::StreamError(value.to_string())
    }
}

impl From<async_nats::error::Error<CreateKeyValueErrorKind>> for Error {
    fn from(value: async_nats::error::Error<CreateKeyValueErrorKind>) -> Self {
        Self::ConfigError(value.to_string())
    }
}

impl From<async_nats::error::Error<ConnectErrorKind>> for Error {
    fn from(value: async_nats::error::Error<ConnectErrorKind>) -> Self {
        Self::ConnectionError(value.to_string())
    }
}

impl From<VarError> for Error {
    fn from(value: VarError) -> Self {
        Self::ConfigError(value.to_string())
    }
}

impl From<async_nats::error::Error<ConsumerErrorKind>> for Error {
    fn from(value: async_nats::error::Error<ConsumerErrorKind>) -> Self {
        Self::ConsumerError(value.to_string())
    }
}

impl From<async_nats::error::Error<GetStreamErrorKind>> for Error {
    fn from(value: async_nats::error::Error<GetStreamErrorKind>) -> Self {
        Self::StreamError(value.to_string())
    }
}

impl From<async_nats::error::Error<BatchErrorKind>> for Error {
    fn from(value: async_nats::error::Error<BatchErrorKind>) -> Self {
        Self::FetchError(value.to_string())
    }
}

impl From<async_nats::error::Error<async_nats::client::PublishErrorKind>> for Error {
    fn from(value: async_nats::error::Error<async_nats::client::PublishErrorKind>) -> Self {
        Self::PublishError(value.to_string())
    }
}
