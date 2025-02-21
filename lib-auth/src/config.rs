use crate::error::Result;
use secrecy::SecretString;
#[derive(Debug, Clone)]
pub struct AzureAuthConfiguration {
    pub client_id: SecretString,
    pub tenant_id: String,
    pub client_secret: SecretString,
}
impl AzureAuthConfiguration {
    pub fn new(client_id: &str, tenant_id: &str, client_secret: &str) -> AzureAuthConfiguration {
        AzureAuthConfiguration {
            client_id: SecretString::new(client_id.into()),
            tenant_id: tenant_id.to_string(),
            client_secret: SecretString::new(client_secret.into()),
        }
    }

    pub fn build() -> Result<AzureAuthConfiguration> {
        let client_id = std::env::var("AZURE_CLIENT_ID").expect("Expected env AZURE_CLIENT_ID");
        let tenant_id = std::env::var("AZURE_TENANT_ID").expect("Expected env AZURE_TENANT_ID");
        let client_secret =
            std::env::var("AZURE_CLIENT_SECRET").expect("Expected env AZURE_CLIENT_SECRET");
        Ok(AzureAuthConfiguration {
            client_id: SecretString::new(client_id.into()),
            tenant_id,
            client_secret: SecretString::new(client_secret.into()),
        })
    }
}

impl Default for AzureAuthConfiguration {
    fn default() -> Self {
        Self {
            client_id: SecretString::new(String::default().into()),
            tenant_id: String::default(),
            client_secret: SecretString::new(String::default().into()),
        }
    }
}
