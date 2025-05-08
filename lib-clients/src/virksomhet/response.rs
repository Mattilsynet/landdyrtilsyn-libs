use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Virksomhet {
    #[serde(rename = "organisasjonsnummer")]
    pub organisasjonsnummer: Option<String>,

    #[serde(rename = "virksomhetNavn")]
    pub virksomhet_navn: Option<String>,

    #[serde(rename = "beliggenhetsadresse")]
    pub beliggenhetsadresse: Option<Adresse>,

    #[serde(rename = "postadresse")]
    pub postadresse: Option<Adresse>,

    #[serde(rename = "kontaktperson")]
    pub kontaktperson: Option<DagligLeder>,

    #[serde(rename = "organisasjonsform")]
    pub organisasjonsform: Option<String>,

    #[serde(rename = "organisasjonsformKode")]
    pub organisasjonsform_kode: Option<String>,

    #[serde(rename = "slettedato")]
    pub slettedato: Option<DateTime<Utc>>,

    #[serde(rename = "overordnetInfo")]
    pub overordnet_info: Option<OverordnetInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VirksomhetFilter {
    pub orgnummer: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DagligLeder {
    pub rolle: Option<String>,
    pub navn: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Adresse {
    pub postnummer: Option<String>,
    pub poststed: Option<String>,
    pub adresse: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OverordnetInfo {
    pub organisasjonsnummer: Option<String>,
    pub telefonnummer: Option<String>,
    pub mobiltelefonnummer: Option<String>,
    pub epostadresse: Option<String>,
    pub hjemmesideadresse: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Underenhet {
    /// Organisasjonsnummer for underenhet
    #[serde(rename = "organisasjonsnummer")]
    pub organisasjonsnummer: Option<String>,

    /// Organisasjonsnummer for overordnet enhet
    #[serde(rename = "overordnetEnhet")]
    pub overordnet_enhet: Option<String>,
}
