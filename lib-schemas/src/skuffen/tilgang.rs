use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Tilgang {
    /// Tilgangskode fra kodeverk TILGANGSKODE
    pub tilgangskode: String,
    /// Hjemmel for at saken skal være unntatt fra offentligheten.
    /// Kode fra kodeverk TILGANGSHJEMMEL
    pub tilgangshjemmel: String,
}
