use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Tilgang {
    pub tilgangskode: String,
    pub tilgangshjemmel: String,
}
