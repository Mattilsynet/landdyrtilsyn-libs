use crate::arkiv::SaksTittel;
use crate::arkiv::tilgangshjemmel::Tilgangshjemmel;
use crate::arkiv::tilgangskoder::Tilgangskode;
use crate::error::Result;
use crate::error::SchemasError;
use serde::{Deserialize, Serialize};
use crate::arkiv::Saksaar;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NySak {
    pub saksbehandler_id: Option<String>,
    pub mt_enhet: Option<String>,
    pub ordningsverdi: String,
    pub tittel: SaksTittel,
    pub skjermingshjemmel: Option<Tilgangshjemmel>,
    pub tilgangskode: Option<Tilgangskode>,
}

impl NySak {
    pub fn validate_skjerming(&self) -> Result<()> {
        let is_tittel_with_skjerming = self.tittel.0.contains('[');
        let has_both_skjerming_and_tilgangskode =
            self.tilgangskode.is_some() && self.skjermingshjemmel.is_some();

        if is_tittel_with_skjerming && !has_both_skjerming_and_tilgangskode {
            return Err(SchemasError::ValidationError(
                "En skjermet arkivsak må ha tilgangskode og skjermingshjemmel.".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sak {
    pub sekvensnummer: String,
    pub saksaar: Saksaar,
    pub tittel: SaksTittel,
    pub status: String,
    pub enhet_id: String,
    pub saksbehandler_id: Option<String>,
    pub skjermingshjemmel: Option<String>,
    pub tilgangskode: Option<String>,
    pub lukket: bool,
}
