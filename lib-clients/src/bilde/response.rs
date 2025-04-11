use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageMetaData {
    pub accuracy: Option<f32>,
    pub app: FotoApp,
    pub archived: Option<bool>,
    pub capture_time: Option<DateTime<Utc>>,

    #[serde(rename = "currentUser")]
    pub current_user: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "currentUserFullName")]
    pub current_user_full_name: Option<String>,

    pub description: Option<String>,
    #[serde(rename = "fileExtension")]
    pub file_extension: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", skip_deserializing)]
    #[serde(rename = "forDeletion")]
    pub for_deletion: Option<DateTime<Utc>>,
    pub id: String,
    #[serde(rename = "locationDescription")]
    pub location_description: Option<String>,
    #[serde(rename = "locationDescriptionCreationTime")]
    pub location_description_creation_time: Option<DateTime<Utc>>,
    #[serde(rename = "locationDescriptionUpdateTime")]
    pub location_description_update_time: Option<DateTime<Utc>>,
    #[serde(rename = "locationLatitude")]
    pub location_latitude: Option<f32>,
    #[serde(rename = "locationLongitude")]
    pub location_longitude: Option<f32>,
    #[serde(rename = "locationCopiedFromId")]
    pub location_copied_from_id: Option<String>,

    /// Brukerens samaccountName
    #[serde(skip_serializing)]
    #[serde(rename = "samaccountName")]
    pub samaccount_name: Option<String>,

    #[serde(skip_deserializing)]
    #[serde(rename = "shaValue")]
    pub sha_value: Option<String>,
}

/// Enum representing FotoApp in Rust
#[derive(Debug, Serialize, Deserialize)]
pub enum FotoApp {
    FOTO,
    MAKKS,
    MakksHk,
    TILSYNSKVITTERING,
}
