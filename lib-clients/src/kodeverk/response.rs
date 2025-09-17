use crate::arkiv::response::Kodeverk;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use reqwest_middleware::reqwest::StatusCode;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct RelatedCode {
    pub code_string: String,
    pub code_type: String,
    pub display_names: DisplayNames,
    pub filter: Option<String>,
    pub valid: bool,
    pub version_data: Option<String>,
}

impl RelatedCode {
    pub fn to_kodeverk(&self) -> Kodeverk {
        Kodeverk {
            id: format!("{}${}", self.code_type.clone(), self.code_string.clone()),
            beskrivelse: self.display_names.no.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayNames {
    pub no: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct Embedded {
    pub related_code_list: Vec<RelatedCode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KodeverkResponse {
    pub _embedded: Embedded,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Code {
    children: Option<Vec<Code>>,
    #[serde(rename = "codeString")]
    pub code_string: String,
    #[serde(rename = "codeType")]
    pub code_type: String,
    #[serde(rename = "displayNames")]
    pub display_names: Option<HashMap<String, String>>,
    filter: Option<String>,
    valid: Option<bool>,
    #[serde(rename = "versionData")]
    version_data: Option<String>,
    #[serde(rename = "parentid")]
    parent_id: Option<String>,
}

impl Code {
    pub fn to_kodeverk(&self) -> Kodeverk {
        let beskrivelse = self
            .display_names
            .as_ref()
            .and_then(|map| map.get("no").cloned())
            .or_else(|| self.display_names.as_ref()?.values().next().cloned());
        Kodeverk {
            id: format!("{}${}", self.code_type.clone(), self.code_string.clone()),
            beskrivelse: beskrivelse.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Error)]
pub enum KodeverkError {
    #[error("http status {status}: {body}")]
    Http { status: StatusCode, body: String },
    #[error("parse error: {0}")]
    Parse(String),
    #[error("client error: {0}")]
    Client(String),
}

pub type KodeverkResult<T> = std::result::Result<T, KodeverkError>;

