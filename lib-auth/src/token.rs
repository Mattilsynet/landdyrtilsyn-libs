pub struct TokenVerify {
    auth_config: AzureAuthConfiguration,
    jwk_set: JwkSet,
}

impl TokenVerify {
    pub fn new() -> Self {}
    pub fn verify(&self) {}
}
