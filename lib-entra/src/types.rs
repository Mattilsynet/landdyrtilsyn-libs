use crate::error::EntraError;
use serde::{Deserialize, Serialize};

pub type Result<T> = core::result::Result<T, EntraError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphUserMemberOf {
    pub id: String,
}

pub(crate) const GRAPH_USER_SELECT_FIELDS: &str =
    "id,displayName,mail,userPrincipalName,givenName,surname,jobTitle,employeeId";

#[derive(Debug, Deserialize, Serialize, Clone)]
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
    #[serde(rename = "onPremisesExtensionAttributes")]
    pub on_premises_extension_attributes: Option<OnPremisesExtensionAttributes>,
    #[serde(rename = "memberOf")]
    pub groups: Option<Vec<GraphUserMemberOf>>,
    pub photo: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OnPremisesExtensionAttributes {
    #[serde(rename = "extensionAttribute6")]
    pub extension_attribute6: Option<String>,
    #[serde(rename = "extensionAttribute10")]
    pub extension_attribute10: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_extension_attributes() {
        let user: GraphUser = serde_json::from_value(serde_json::json!({
            "id": "user-1",
            "displayName": "Test User",
            "employeeId": "123",
            "onPremisesExtensionAttributes": {
                "extensionAttribute6": "456",
                "extensionAttribute10": "department"
            }
        }))
        .unwrap();

        assert_eq!(user.id.as_deref(), Some("user-1"));
        assert_eq!(user.display_name.as_deref(), Some("Test User"));
        assert_eq!(user.employeeid.as_deref(), Some("123"));
        assert!(user.mail.is_none());
        let attributes = user.on_premises_extension_attributes.unwrap();
        assert_eq!(attributes.extension_attribute6.as_deref(), Some("456"));
        assert_eq!(
            attributes.extension_attribute10.as_deref(),
            Some("department")
        );
        assert_eq!(
            serde_json::to_value(attributes).unwrap(),
            serde_json::json!({
                "extensionAttribute6": "456",
                "extensionAttribute10": "department"
            })
        );
    }

    #[test]
    fn deserialize_missing_or_null_optional_attributes() {
        for json in ["{}", r#"{"onPremisesExtensionAttributes":null}"#] {
            let user: GraphUser = serde_json::from_str(json).unwrap();
            assert!(user.on_premises_extension_attributes.is_none());
        }

        for json in [
            r#"{"onPremisesExtensionAttributes":{}}"#,
            r#"{"onPremisesExtensionAttributes":{"extensionAttribute6":null,"extensionAttribute10":null}}"#,
        ] {
            let user: GraphUser = serde_json::from_str(json).unwrap();
            let attributes = user.on_premises_extension_attributes.unwrap();
            assert!(attributes.extension_attribute6.is_none());
            assert!(attributes.extension_attribute10.is_none());
        }
    }
}
