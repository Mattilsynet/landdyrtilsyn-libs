pub use error::{Error, Result};

use async_nats::jetstream::Context;
use async_nats::jetstream::message::StreamMessage;
use async_nats::jetstream::stream::{LastRawMessageErrorKind, Stream};
use futures_util::TryStreamExt;
use serde::Deserialize;

mod error;

const STREAM: &str = "ansatte";
const SUBJECT_PREFIX: &str = "ansatte";

#[derive(Clone)]
pub struct MtAnsatte {
    jetstream: Context,
}

/// Entra-brukeren slik Entra-replikatoren publiserer den.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ansatt {
    pub id: Option<String>,
    pub account_enabled: Option<bool>,
    pub display_name: Option<String>,
    pub user_principal_name: Option<String>,
    pub job_title: Option<String>,
    pub employee_id: Option<String>,
    pub employee_type: Option<String>,
    pub mobile_phone: Option<String>,
    pub mail: Option<String>,
    pub office_location: Option<String>,
    pub department: Option<String>,
    pub on_premises_extension_attributes: Option<ExtensionAttributes>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionAttributes {
    /// Koststed.
    pub extension_attribute6: Option<String>,
    /// Lederens ansattnummer.
    pub extension_attribute8: Option<String>,
}

impl MtAnsatte {
    pub fn new(client: async_nats::Client) -> Self {
        Self {
            jetstream: async_nats::jetstream::new(client),
        }
    }

    /// Nyeste post for ansatten på tvers av enheter.
    pub async fn ansatt(&self, employee_id: &str) -> Result<Option<Ansatt>> {
        let filter = format!("{SUBJECT_PREFIX}.*.{}", gyldig_token(employee_id)?);
        let stream = self.jetstream.get_stream(STREAM).await?;
        match siste_melding(&stream, &filter).await? {
            Some(melding) => Ok(Some(serde_json::from_slice(&melding.payload)?)),
            None => Ok(None),
        }
    }

    pub async fn ansatt_navn(&self, employee_id: &str) -> Result<Option<String>> {
        Ok(self
            .ansatt(employee_id)
            .await?
            .and_then(|ansatt| ikke_tom(ansatt.display_name)))
    }

    /// `department` hos en ansatt på enheten. Tomme meldinger er ansatte som
    /// har flyttet ut, og hoppes over.
    pub async fn enhet_navn(&self, departement_id: &str) -> Result<Option<String>> {
        let filter = format!("{SUBJECT_PREFIX}.{}.*", gyldig_token(departement_id)?);
        let stream = self.jetstream.get_stream(STREAM).await?;
        let mut subjects = stream.info_with_subjects(&filter).await?;
        while let Some((subject, _)) = subjects.try_next().await? {
            let Some(melding) = siste_melding(&stream, &subject).await? else {
                continue;
            };
            if melding.payload.is_empty() {
                continue;
            }
            let ansatt: Ansatt = serde_json::from_slice(&melding.payload)?;
            if let Some(navn) = ikke_tom(ansatt.department) {
                return Ok(Some(navn));
            }
        }
        Ok(None)
    }
}

async fn siste_melding(stream: &Stream, subject: &str) -> Result<Option<StreamMessage>> {
    match stream.get_last_raw_message_by_subject(subject).await {
        Ok(melding) => Ok(Some(melding)),
        Err(feil) if feil.kind() == LastRawMessageErrorKind::NoMessageFound => Ok(None),
        Err(feil) => Err(feil.into()),
    }
}

fn ikke_tom(verdi: Option<String>) -> Option<String> {
    verdi.filter(|verdi| !verdi.is_empty())
}

fn gyldig_token(id: &str) -> Result<&str> {
    let ugyldig = id.is_empty()
        || id
            .chars()
            .any(|c| c == '.' || c == '*' || c == '>' || c.is_whitespace() || c.is_control());
    if ugyldig {
        return Err(Error::InvalidId(id.to_string()));
    }
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leser_hele_ansattposten() {
        let ansatt: Ansatt = serde_json::from_value(serde_json::json!({
            "id": "00000000-1111-2222-3333-444444444444",
            "accountEnabled": true,
            "displayName": "Test Person",
            "employeeId": "99990001",
            "department": "seksjon test",
            "onPremisesExtensionAttributes": {
                "extensionAttribute6": "M42210",
                "extensionAttribute8": "99990002",
            },
            "ignoredField": "verdi",
        }))
        .unwrap();

        assert_eq!(ansatt.account_enabled, Some(true));
        assert_eq!(ansatt.display_name.as_deref(), Some("Test Person"));
        assert_eq!(ansatt.department.as_deref(), Some("seksjon test"));
        assert_eq!(
            ansatt
                .on_premises_extension_attributes
                .and_then(|attributter| attributter.extension_attribute6)
                .as_deref(),
            Some("M42210")
        );
    }

    #[test]
    fn manglende_felter_gir_none() {
        let ansatt: Ansatt = serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(ansatt, Ansatt::default());
    }

    #[test]
    fn feil_felttype_feiler() {
        let result = serde_json::from_value::<Ansatt>(serde_json::json!({
            "displayName": 42,
        }));
        assert!(result.is_err());
    }

    #[test]
    fn avviser_ugyldige_ider() {
        for id in ["", "12.34", "a*b", "a>b", "med mellomrom", "kontroll\u{0}"] {
            assert!(gyldig_token(id).is_err(), "skal avvise {id:?}");
        }
    }

    #[test]
    fn godtar_gyldige_ider() {
        for id in ["99990001", "M12345", "000123"] {
            assert!(gyldig_token(id).is_ok(), "skal godta {id:?}");
        }
    }
}
