use serde::Deserialize;

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
    pub seksjon_id: Option<String>,
    #[serde(rename(deserialize = "avdelingId"))]
    pub avdeling_id: Option<String>,
    #[serde(rename(deserialize = "regionId"))]
    pub region_id: Option<String>,
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
