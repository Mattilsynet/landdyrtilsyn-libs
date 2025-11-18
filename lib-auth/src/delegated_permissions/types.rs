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
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct UserData {
    pub navn: String,
    pub epost: String,
}
