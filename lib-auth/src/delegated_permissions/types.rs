use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub(crate) struct Claims {
    aud: String,
    exp: usize,
    iat: usize,
    iss: String,
    nbf: usize,
    sub: String,
    name: String,
    preferred_username: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct JwkSet {
    pub(crate) keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub(crate) struct Jwk {
    pub(crate) kid: String,
    pub(crate) n: String,
    pub(crate) e: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct UserData {
    pub navn: String,
    pub epost: String,
}
