use serde::{Deserialize, Serialize};

/// Access restriction metadata for saker og journalposter.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Tilgang {
    /// Access code fra TILGANGSKODE code set.
    pub tilgangskode: String,
    /// Legal basis for exemption fra public access.
    /// Code fra TILGANGSHJEMMEL code set.
    pub tilgangshjemmel: String,
}
