use crate::error::{ParseError, Result, SchemasError};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Tilgangskode som uttrykker at et objekt er skjermet.
///
/// Kontrakten uttrykker intensjon, ikke vendor-mekanikk: koden er en validert,
/// ikke-tom verdi. Validering skjer ved konstruksjon og ved deserialisering,
/// slik at en skjerming aldri kan mangle en kode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct Tilgangskode(String);

impl Tilgangskode {
    /// Lag en validert tilgangskode. Non-empty er eneste invariant på
    /// kontraktsnivå; kodeverket eies av arkivet, ikke av wire-kontrakten.
    pub fn new(kode: impl Into<String>) -> Result<Self> {
        let kode = kode.into();
        if kode.trim().is_empty() {
            return Err(SchemasError::ParseError(ParseError::Message(
                "tilgangskode er tom".to_string(),
            )));
        }
        Ok(Self(kode))
    }

    /// Returner rå tilgangskode-string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Tilgangskode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Tilgangskode {
    type Error = SchemasError;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl From<Tilgangskode> for String {
    fn from(value: Tilgangskode) -> Self {
        value.0
    }
}

/// Rettslig hjemmel for skjerming.
///
/// Skjerming uten hjemmel skal være urepresenterbar: en [`Tilgangskode`]
/// opptrer aldri alene i kontrakten (se [`Tilgjengelighet::Skjermet`]), og
/// hjemmelen er alltid en validert, ikke-tom verdi.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct Tilgangshjemmel(String);

impl Tilgangshjemmel {
    /// Lag en validert tilgangshjemmel. Non-empty er eneste invariant på
    /// kontraktsnivå.
    pub fn new(hjemmel: impl Into<String>) -> Result<Self> {
        let hjemmel = hjemmel.into();
        if hjemmel.trim().is_empty() {
            return Err(SchemasError::ParseError(ParseError::Message(
                "tilgangshjemmel er tom".to_string(),
            )));
        }
        Ok(Self(hjemmel))
    }

    /// Returner rå tilgangshjemmel-string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Tilgangshjemmel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Tilgangshjemmel {
    type Error = SchemasError;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl From<Tilgangshjemmel> for String {
    fn from(value: Tilgangshjemmel) -> Self {
        value.0
    }
}

/// Tilgjengeligheten til en sak eller journalpost.
///
/// Modellen uttrykker *intensjon*, ikke vendor-mekanikk. Vi eksponerer ikke
/// person/unntattOffentlighet-booleans, GENERELL/DIG-flagg eller andre
/// arkivspesifikke felter. Enten er noe offentlig, eller så er det skjermet
/// *med* en hjemmel — skjerming uten hjemmel er dermed urepresenterbart.
///
/// Serialiseres eksternt tagget (serde default), jf. SKU-0004:
/// - `Offentlig` -> `"Offentlig"`
/// - `Skjermet`  -> `{ "Skjermet": { "tilgangskode": ..., "tilgangshjemmel": ... } }`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tilgjengelighet {
    /// Objektet er offentlig tilgjengelig.
    Offentlig,
    /// Objektet er skjermet med tilhørende hjemmel.
    Skjermet {
        tilgangskode: Tilgangskode,
        tilgangshjemmel: Tilgangshjemmel,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tilgangskode_validering_og_bare_string_serde() {
        assert!(Tilgangskode::new("").is_err());
        assert!(Tilgangskode::new("   ").is_err());
        let kode = Tilgangskode::new("UO").unwrap();
        assert_eq!(kode.as_str(), "UO");
        assert_eq!(kode.to_string(), "UO");
        assert_eq!(serde_json::to_value(&kode).unwrap(), json!("UO"));
        let back: Tilgangskode = serde_json::from_value(json!("UO")).unwrap();
        assert_eq!(back, kode);
        assert!(serde_json::from_value::<Tilgangskode>(json!("")).is_err());
    }

    #[test]
    fn tilgangshjemmel_validering_og_bare_string_serde() {
        assert!(Tilgangshjemmel::new("").is_err());
        let h = Tilgangshjemmel::new("Offl. § 13").unwrap();
        assert_eq!(h.as_str(), "Offl. § 13");
        assert_eq!(serde_json::to_value(&h).unwrap(), json!("Offl. § 13"));
        let back: Tilgangshjemmel = serde_json::from_value(json!("Offl. § 13")).unwrap();
        assert_eq!(back, h);
    }

    #[test]
    fn tilgjengelighet_offentlig_er_bare_streng() {
        let t = Tilgjengelighet::Offentlig;
        let value = serde_json::to_value(&t).unwrap();
        assert_eq!(value, json!("Offentlig"));
        let back: Tilgjengelighet = serde_json::from_value(value).unwrap();
        assert_eq!(back, t);
    }

    #[test]
    fn tilgjengelighet_skjermet_er_eksternt_tagget() {
        let t = Tilgjengelighet::Skjermet {
            tilgangskode: Tilgangskode::new("UO").unwrap(),
            tilgangshjemmel: Tilgangshjemmel::new("Offl. § 13").unwrap(),
        };
        let value = serde_json::to_value(&t).unwrap();
        assert_eq!(
            value,
            json!({
                "Skjermet": {
                    "tilgangskode": "UO",
                    "tilgangshjemmel": "Offl. § 13"
                }
            })
        );
        let back: Tilgjengelighet = serde_json::from_value(value).unwrap();
        assert_eq!(back, t);
    }

    #[test]
    fn skjermet_uten_gyldig_kode_avvises_ved_deserialisering() {
        let bad = json!({
            "Skjermet": { "tilgangskode": "", "tilgangshjemmel": "Offl. § 13" }
        });
        assert!(serde_json::from_value::<Tilgjengelighet>(bad).is_err());
    }
}
