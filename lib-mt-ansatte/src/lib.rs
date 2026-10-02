pub use error::{Error, Result};

use async_nats::jetstream::Context;
use async_nats::jetstream::stream::LastRawMessageErrorKind;

mod error;

const STREAM: &str = "ansatte";
const SUBJECT_PREFIX: &str = "ansatte";

#[derive(Clone)]
pub struct MtAnsatte {
    jetstream: Context,
}

impl MtAnsatte {
    pub fn new(client: async_nats::Client) -> Self {
        Self {
            jetstream: async_nats::jetstream::new(client),
        }
    }

    pub async fn ansatt_navn(&self, employee_id: &str) -> Result<Option<String>> {
        let subject = format!("{SUBJECT_PREFIX}.*.{}", gyldig_token(employee_id)?);
        self.les(&subject, AnsattPayload::ansatt_navn).await
    }

    pub async fn enhet_navn(&self, departement_id: &str) -> Result<Option<String>> {
        let subject = format!("{SUBJECT_PREFIX}.{}.*", gyldig_token(departement_id)?);
        self.les(&subject, AnsattPayload::enhet_navn).await
    }

    async fn les(
        &self,
        subject: &str,
        velg: fn(&AnsattPayload) -> Option<&str>,
    ) -> Result<Option<String>> {
        let stream = self.jetstream.get_stream(STREAM).await?;
        let melding = match stream.get_last_raw_message_by_subject(subject).await {
            Ok(melding) => melding,
            Err(feil) if feil.kind() == LastRawMessageErrorKind::NoMessageFound => return Ok(None),
            Err(feil) => return Err(feil.into()),
        };
        let payload: AnsattPayload = serde_json::from_slice(&melding.payload)?;
        Ok(velg(&payload).map(str::to_string))
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnsattPayload {
    display_name: Option<String>,
    department: Option<String>,
}

impl AnsattPayload {
    fn ansatt_navn(&self) -> Option<&str> {
        self.display_name.as_deref().filter(|navn| !navn.is_empty())
    }

    fn enhet_navn(&self) -> Option<&str> {
        self.department.as_deref().filter(|navn| !navn.is_empty())
    }
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

    fn payload(json: serde_json::Value) -> AnsattPayload {
        serde_json::from_value(json).expect("valid payload")
    }

    #[test]
    fn leser_ansattnavn_og_enhet() {
        let ansatt = payload(serde_json::json!({
            "id": "00000000-1111-2222-3333-444444444444",
            "accountEnabled": true,
            "displayName": "Test Person",
            "employeeId": "99990001",
            "department": "Test Avdeling",
            "ignoredField": "verdi",
        }));
        assert_eq!(ansatt.ansatt_navn(), Some("Test Person"));
        assert_eq!(ansatt.enhet_navn(), Some("Test Avdeling"));
    }

    #[test]
    fn manglende_felter_gir_none() {
        let ansatt = payload(serde_json::json!({}));
        assert_eq!(ansatt.ansatt_navn(), None);
        assert_eq!(ansatt.enhet_navn(), None);
    }

    #[test]
    fn blanke_felter_gir_none() {
        let ansatt = payload(serde_json::json!({
            "displayName": "",
            "department": null,
        }));
        assert_eq!(ansatt.ansatt_navn(), None);
        assert_eq!(ansatt.enhet_navn(), None);
    }

    #[test]
    fn feil_felttype_feiler() {
        let result = serde_json::from_value::<AnsattPayload>(serde_json::json!({
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
