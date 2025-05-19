use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Ansatt {
    pub brukernavn: String,
    pub navn: String,
    pub tittel: Option<String>,
    pub telefonnummer: Option<String>,
    #[serde(rename(deserialize = "kontorId"))]
    pub kontor_id: Option<String>,
    #[serde(rename(deserialize = "kontorNavn"))]
    pub kontor_navn: Option<String>,
    #[serde(rename(deserialize = "seksjonId"))]
    pub seksjon_id: Option<String>,
    #[serde(rename(deserialize = "avdelingId"))]
    pub avdeling_id: Option<String>,
    #[serde(rename(deserialize = "regionId"))]
    pub region_id: Option<String>,
    #[serde(rename(deserialize = "orgenhetId"))]
    pub orgenhet_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Orgenhet {
    pub id: String,
    #[serde(alias = "username")]
    pub user_name: Option<String>,
    pub name: String,
    pub active: Option<bool>,
    #[serde(alias = "tittel")]
    pub title: Option<String>,
    #[serde(alias = "telefon")]
    pub phone_number: Option<String>,
    #[serde(alias = "orgunittype")]
    pub org_unit_type: Option<String>,
    pub children: Option<Vec<Orgenhet>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Type {
    Region,
    Avdeling,
    Seksjon,
    Annen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct Kontor {
    id: String,
    pub kortnavn: String,
    pub navn: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    kontor_type: Type,
    seksjon_id: Option<String>,
    avdeling_id: Option<String>,
    region_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Region {
    pub id: String,
    pub kortnavn: String,
    pub navn: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Avdeling {
    pub id: String,
    pub kortnavn: String,
    pub navn: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Seksjon {
    pub id: String,
    pub kortnavn: String,
    pub navn: String,
}
