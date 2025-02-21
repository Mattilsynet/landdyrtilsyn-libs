use async_nats::Client;
use secrecy::{ExposeSecret, SecretString};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct NatsConfiguration {
    url: String,
    credentials: SecretString,
}

impl NatsConfiguration {
    pub async fn new(url: &str, credentials: &str) -> NatsConfiguration {
        NatsConfiguration {
            url: url.to_string(),
            credentials: SecretString::new(credentials.into()),
        }
    }

    pub async fn build() -> Result<NatsConfiguration> {
        let url = std::env::var("NATS_URL")?;
        let credentials = std::env::var("NATS_CREDENTIALS")?;

        Ok(NatsConfiguration {
            credentials: SecretString::new(credentials.into_boxed_str()),
            url,
        })
    }
}

pub async fn create_client(config: NatsConfiguration) -> Result<Client> {
    let client = async_nats::ConnectOptions::with_credentials(config.credentials.expose_secret())
        .expect("Failed to parse static Nats credentials")
        .connect(config.url)
        .await?;
    Ok(client)
}

pub async fn create_jetstream_instance(client: Client) -> async_nats::jetstream::Context {
    async_nats::jetstream::new(client)
}
