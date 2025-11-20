use axum::BoxError;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse as _, Response};
use bytes::Bytes;
use futures::future::BoxFuture;
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, decode_header};
use secrecy::{ExposeSecret, SecretString};
use std::{convert::Infallible, sync::Arc};
use tower::{Layer, Service};

use crate::{
    delegated_permissions::types::{Claims, JwkSet},
    error::{Error, Result},
};

#[derive(Clone, Debug)]
pub struct AuthConfig {
    tenant_id: String,
    client_id: SecretString,
}

impl AuthConfig {
    pub fn new(tenant_id: &str, client_id: SecretString) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            client_id,
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
        Ok(Self {
            tenant_id,
            client_id: SecretString::from(client_id),
        })
    }
}

#[derive(Debug, Clone)]
pub struct TokenValidator {
    config: Arc<AuthConfig>,
    pub(crate) jwk_set: JwkSet,
}

async fn fetch_jwks(jwks_url: &str) -> Result<JwkSet> {
    reqwest::get(jwks_url)
        .await?
        .json::<JwkSet>()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch JwkSet");
            Error::ReqwestError(e)
        })
}

impl TokenValidator {
    pub async fn new(config: AuthConfig) -> Result<Self> {
        let jwks_url = config.get_jwks_url().to_owned();
        let jwk_set = fetch_jwks(&jwks_url).await.map_err(|error| error)?;

        Ok(Self {
            config: Arc::new(config),
            jwk_set,
        })
    }

    pub async fn from_env() -> Result<Self> {
        Ok(Self::new(AuthConfig::from_env()?).await?)
    }

    async fn verify_token(&self, token: &str) -> Result<TokenData<Claims>> {
        let header = decode_header(token).map_err(|e| {
            tracing::error!("Failed to decode JWT header: {:?}", e);
            Error::JwtError(e)
        })?;
        let kid = header.kid.ok_or_else(|| {
            tracing::error!("JWT header missing 'kid' field!");
            Error::JwtError(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat.into())
        })?;

        let jwk = self
            .jwk_set
            .keys
            .iter()
            .find(|&k| k.kid == kid)
            .ok_or_else(|| {
                tracing::error!("No matching JWK found for kid: {}", kid);
                Error::JwtError(jsonwebtoken::errors::ErrorKind::InvalidIssuer.into())
            })?;

        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.config.client_id.expose_secret()]);
        validation.set_issuer(&[format!(
            "https://login.microsoftonline.com/{}/v2.0",
            self.config.tenant_id
        )]);

        let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
        Ok(token_data)
    }
}

#[derive(Clone, Debug)]
pub struct AuthLayer {
    validator: TokenValidator,
}

impl AuthLayer {
    pub async fn new(validator: TokenValidator) -> Self {
        Self { validator }
    }

    pub async fn from_env() -> Result<Self> {
        Ok(Self::new(TokenValidator::from_env().await?).await)
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
    validator: TokenValidator,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for Auth<S>
where
    S: Service<Request<ReqBody>, Response = axum::http::Response<ResBody>, Error = Infallible>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: axum::body::HttpBody<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<BoxError>,
{
    type Response = Response; // unified axum Response
    type Error = Infallible;
    type Future = BoxFuture<'static, std::result::Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let validator = self.validator.clone();

        Box::pin(async move {
            let token = match extract_bearer_token(&req) {
                Some(t) if !t.is_empty() => t,
                _ => return Ok(StatusCode::UNAUTHORIZED.into_response()),
            };

            match validator.verify_token(token).await {
                Ok(token_data) => {
                    // Attach claims to request extensions for downstream handlers
                    req.extensions_mut().insert(token_data.claims);
                }
                Err(e) => {
                    return Ok(e.status_code().into_response());
                }
            }

            let inner_response = inner.call(req).await?; // http::Response<B>
            Ok(inner_response.into_response()) // convert using IntoResponse
        })
    }
}

pub fn extract_bearer_token<B>(req: &Request<B>) -> Option<&str> {
    req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|auth_str| auth_str.strip_prefix("Bearer "))
}
