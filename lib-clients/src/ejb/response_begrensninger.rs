use crate::ejb::response_tilfeller::date_format;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const ALLOWED_FIELDS_BEGRENSNINGER: &[&str] = &[
    "idstring",
    "version",
    "typeid",
    "handlingsloepref",
    "soeknadref",
    "gbrnummerref",
    "begrensningaarsakid",
    "aarsakid",
    "createddate",
    "beskrivelse",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponseBegrensninger {
    pub results: Vec<Begrensning>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Begrensning {
    pub idstring: String,
    pub version: i32,
    pub typeid: String,
    #[serde(with = "date_format")]
    pub fradato: Option<DateTime<Utc>>,
    #[serde(with = "date_format")]
    pub tildato: Option<DateTime<Utc>>,
    pub handlingsloepref: Option<String>,
    pub soeknadref: Option<String>,
    pub begrensningsaarsakid: String,
    #[serde(with = "date_format")]
    pub createddate: Option<DateTime<Utc>>,
    #[serde(with = "date_format")]
    pub lastmodifieddate: Option<DateTime<Utc>>,
    pub gbrnummerref: Option<String>,
    pub aarsakid: String,
    pub beskrivelse: String,
}
