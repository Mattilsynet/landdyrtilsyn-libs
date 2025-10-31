use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeonorgeResponse {
    #[serde(rename = "adresser")]
    pub addresses: Vec<AddressResult>,
    #[serde(rename = "totaltAntallTreff")]
    pub total_hits: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressResult {
    #[serde(rename = "adressetekst")]
    pub address_text: String,

    #[serde(rename = "adressenavn")]
    pub street_name: Option<String>,

    #[serde(rename = "nummer")]
    pub number: Option<AddressNumber>,

    #[serde(rename = "postnummer")]
    pub postal_code: Option<String>,

    #[serde(rename = "poststed")]
    pub city: Option<String>,

    #[serde(rename = "kommunenavn")]
    pub municipality: Option<String>,

    #[serde(rename = "kommunenummer")]
    pub municipality_number: Option<String>,

    #[serde(rename = "representasjonspunkt")]
    pub koordinater: Option<Koordinater>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddressNumber {
    // Geonorge sometimes returns just an integer, e.g., "nummer": 12
    Number(i32),
    // Or a structured form with optional letter, e.g., { husnummer: 12, bokstav: "b" }
    Detailed {
        husnummer: Option<i32>,
        bokstav: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Koordinater {
    #[serde(rename = "lat")]
    pub latitude: f64,

    #[serde(rename = "lon")]
    pub longitude: f64,
}

impl AddressResult {
    pub fn full_address(&self) -> String {
        self.address_text.clone()
    }

    pub fn get_koordinater(&self) -> Option<(f64, f64)> {
        self.koordinater.as_ref().map(|c| (c.latitude, c.longitude))
    }
}
