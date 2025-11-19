use std::sync::Arc;

use futures::future::BoxFuture;
use reqwest::{Request, Response};
use secrecy::SecretString;
use tower::{Layer, Service};

use crate::{
    delegated_permissions::types::Claims,
    error::{Error, Result},
};

#[derive(Clone, Debug)]
pub struct AuthConfig {
    tenant_id: String,
    client_id: SecretString,
    client_secret: SecretString,
}

impl AuthConfig {
    pub fn new(tenant_id: &str, client_id: SecretString, client_secret: SecretString) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            client_id,
            client_secret,
        }
    }

    pub fn get_jwks_url(&self) -> String {
        format!(
            "https://login.microsoftonline.com/{}/discovery/v2.0/keys",
            self.tenant_id
        )
    }

    pub fn from_env() -> Result<Self> {
        let tenant_id = std::env::var("AZURE_TENANT_ID")
            .map_err(|_| Error::MissingEnvVariable("AZURE_TENANT_ID is missing".to_string()))?;
        let client_id = std::env::var("AZURE_CLIENT_ID")
            .map_err(|_| Error::MissingEnvVariable("AZURE_CLIENT_ID is missing".to_string()))?;
        let client_secret = std::env::var("AZURE_CLIENT_SECRET")
            .map_err(|_| Error::MissingEnvVariable("AZURE_CLIENT_SECRET is missing".to_string()))?;

        Ok(Self {
            tenant_id,
            client_id: SecretString::from(client_id),
            client_secret: SecretString::from(client_secret),
        })
    }
}

// TokenValidator
#[derive(Debug, Clone)]
pub struct EntraIdValidator {
    config: Arc<AuthConfig>,
    http: reqwest::Client,
    jwks: String,
}

impl EntraIdValidator {
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config: Arc::new(config),
            http: reqwest::Client::new(),
            jwks: String::default(),
        }
    }

    pub fn from_env() -> Result<Self> {
        Ok(Self::new(AuthConfig::from_env()?))
    }

    pub async fn verify_token(&self, token: &str) -> Result<Claims> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct AuthLayer {
    validator: EntraIdValidator,
}

impl AuthLayer {
    pub fn new(validator: EntraIdValidator) -> Self {
        Self { validator }
    }

    pub fn from_env() -> Result<Self> {
        Ok(Self::new(EntraIdValidator::from_env()?))
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = Auth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Auth {
            inner,
            validator: self.validator.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Auth<S> {
    inner: S,
    validator: EntraIdValidator,
}

impl<S> Service<Request> for Auth<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Error>,
{
    type Response = S::Response;
    type Error = Error;
    type Future = BoxFuture<'static, std::result::Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let inner = self.inner.clone();
        let validator = self.validator.clone();

        Box::pin(async move {
            let token = extract_bearer_token(&req)
                .ok_or_else(|| Error::MissingTokenOnRequest("Heihei".to_string()))?;

            validator.verify_token(token_str).await?;

            inner.call(req).await.map_err(Into::into)
        })
    }
}

pub fn extract_bearer_token(req: &Request) -> Option<&str> {
    req.headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|auth_str| auth_str.strip_prefix("Bearer "))
}
