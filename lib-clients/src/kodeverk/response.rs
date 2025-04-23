use crate::arkiv::response::Kodeverk;
use serde::{Deserialize, Serialize};

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
