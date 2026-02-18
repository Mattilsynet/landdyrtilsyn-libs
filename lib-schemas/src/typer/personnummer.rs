use serde::{Deserialize, Serialize};

/// Norsk national identification number (11 digits).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Personnummer(String);

impl Personnummer {
    /// Lag et validert personnummer.
    pub fn new(pnr: impl Into<String>) -> Result<Self, &'static str> {
        let pnr = pnr.into();
        if !Self::valider(&pnr) {
            return Err("ugyldig personnummer");
        }
        Ok(Self(pnr))
    }

    /// Returner raw number string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Valider control digits (K1/K2) for et fødselsnummer.
    ///
    /// Ref: http://www.fnrinfo.no/Teknisk/KontrollsifferSjekk.aspx
    pub fn valider(fnr: &str) -> bool {
        if fnr.len() != 11 || !fnr.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        let d: Vec<u32> = fnr.chars().map(|c| c.to_digit(10).unwrap()).collect();

        // k1-sjekk: 10 første sifre
        let weights1 = [3, 7, 6, 1, 8, 9, 4, 5, 2, 1];
        let sum1: u32 = weights1.iter().zip(d.iter()).map(|(w, d)| w * d).sum();
        if !sum1.is_multiple_of(11) {
            return false;
        }

        // k2-sjekk: alle 11 sifre
        let weights2 = [5, 4, 3, 2, 7, 6, 5, 4, 3, 2, 1];
        let sum2: u32 = weights2.iter().zip(d.iter()).map(|(w, d)| w * d).sum();
        if !sum2.is_multiple_of(11) {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_personnummer() {
        assert!(Personnummer::new("01010101006").is_ok());
    }

    #[test]
    fn invalid_personnummer() {
        assert!(Personnummer::new("01010101007").is_err());
    }
}
