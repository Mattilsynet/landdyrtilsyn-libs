use serde::{Deserialize, Serialize};

/// Norsk organization number (9 digits).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct Organisasjonsnummer(String);

impl Organisasjonsnummer {
    /// Lag et validert organisasjonsnummer.
    pub fn new(orgnr: impl Into<String>) -> Result<Self, &'static str> {
        let orgnr = orgnr.into();
        if !Self::valider(&orgnr) {
            return Err("ugyldig organisasjonsnummer");
        }
        Ok(Self(orgnr))
    }

    /// Returner raw number string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Valider control digit med modulus 11.
    fn valider(orgnr: &str) -> bool {
        if orgnr.len() != 9 || !orgnr.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        let d: Vec<u32> = orgnr.chars().map(|c| c.to_digit(10).unwrap()).collect();
        let weights = [3, 2, 7, 6, 5, 4, 3, 2];
        let sum: u32 = weights.iter().zip(&d).map(|(w, d)| w * d).sum();
        let rem = sum % 11;
        let k = if rem == 0 { 0 } else { 11 - rem };

        k != 10 && k == d[8]
    }
}

impl TryFrom<String> for Organisasjonsnummer {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Organisasjonsnummer> for String {
    fn from(value: Organisasjonsnummer) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_orgnr() {
        assert!(Organisasjonsnummer::new("995298775").is_ok());
    }

    #[test]
    fn invalid_orgnr() {
        assert!(Organisasjonsnummer::new("995298776").is_err());
    }

    #[test]
    fn deserialisering_avviser_ugyldig() {
        assert!(serde_json::from_value::<Organisasjonsnummer>(serde_json::json!("tull")).is_err());
        assert!(
            serde_json::from_value::<Organisasjonsnummer>(serde_json::json!("995298775")).is_ok()
        );
    }
}
