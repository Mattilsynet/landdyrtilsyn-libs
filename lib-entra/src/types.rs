use crate::error::EntraError;
use serde::{Deserialize, Serialize};

pub type Result<T> = core::result::Result<T, EntraError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphUserMemberOf {
    pub id: String,
}

pub(crate) const GRAPH_USER_SELECT_FIELDS: &str =
    "id,displayName,mail,userPrincipalName,givenName,surname,jobTitle,employeeId";

#[derive(Debug, Deserialize, Clone)]
pub struct GraphUser {
    pub id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub mail: Option<String>,
    #[serde(rename = "userPrincipalName")]
    pub user_principal_name: Option<String>,
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    pub surname: Option<String>,
    #[serde(rename = "jobTitle")]
    pub job_title: Option<String>,
    #[serde(rename = "employeeId")]
    pub employeeid: Option<String>,
    #[serde(rename = "memberOf")]
    pub groups: Option<Vec<GraphUserMemberOf>>,
    pub photo: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GraphUserSearchResponse {
    #[serde(rename = "@odata.context")]
    pub odata_context: Option<String>,
    #[serde(rename = "@odata.count")]
    pub odata_count: Option<u32>,
    pub value: Vec<GraphUser>,
}

#[derive(Debug, Clone)]
pub(crate) struct OboConfig {
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
}

impl OboConfig {
    pub fn from_env() -> Result<Self> {
        let tenant_id = std::env::var("AZURE_TENANT_ID")
            .map_err(|_| EntraError::MissingEnv("AZURE_TENANT_ID".into()))?;
        let client_id = std::env::var("AZURE_CLIENT_ID")
            .map_err(|_| EntraError::MissingEnv("AZURE_CLIENT_ID".into()))?;
        let client_secret = std::env::var("AZURE_CLIENT_SECRET")
            .map_err(|_| EntraError::MissingEnv("AZURE_CLIENT_SECRET".into()))?;
        Ok(Self {
            tenant_id,
            client_id,
            client_secret,
        })
    }
}
