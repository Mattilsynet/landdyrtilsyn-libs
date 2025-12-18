use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Email {
    pub message: Option<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub subject: String,
    pub body: Option<Body>,
    #[serde(rename = "toRecipients")]
    pub to_recipients: Vec<Recipient>,
    #[serde(rename = "ccRecipients")]
    pub cc_recipients: Vec<Recipient>,
    #[serde(rename = "internetMessageHeaders")]
    pub internet_message_headers: Vec<InternetMessageHeader>,
    pub attachments: Vec<Attachment>,
    pub flag: Option<Flag>,
    #[serde(rename = "saveToSentItems")]
    pub save_to_sent_items: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Body {
    #[serde(rename = "contentType")]
    pub content_type: String, // expected values: "Text" | "HTML"
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Recipient {
    #[serde(rename = "emailAddress")]
    pub email_address: Option<EmailAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmailAddress {
    pub address: String, // validation (isEmail) was in proto, not enforced here.
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attachment {
    #[serde(rename = "@odata.type")]
    pub o_data_type: String,
    pub name: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "contentBytes")]
    pub content_bytes: String, // base64? keep as provided
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InternetMessageHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DateTime {
    #[serde(rename = "dateTime")]
    pub date_time: String,
    #[serde(rename = "timeZone")]
    pub time_zone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Flag {
    #[serde(rename = "flagStatus")]
    pub flag_status: String,
    #[serde(rename = "startDateTime")]
    pub start_date_time: Option<DateTime>,
    #[serde(rename = "dueDateTime")]
    pub due_date_time: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    pub status: Status,
    #[serde(rename = "timeStamp")]
    pub time_stamp: Option<DateTime>,
    #[serde(rename = "errorMessage")]
    pub error_message: String,
    #[serde(rename = "streamName")]
    pub stream_name: String,
    #[serde(rename = "sequenceNumber")]
    pub sequence_number: u64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    #[default]
    StatusUnspecified,
    StatusSuccess,
    StatusError,
}
