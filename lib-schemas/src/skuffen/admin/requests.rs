use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Oppslag av én kommando med tilhørende nåværende operasjonsrader.
///
/// `utfort_av` er selvdeklarert attribusjon, ikke autentisering. Verdien
/// logges, men lagres ikke. Blankhet håndheves ved transportgrensen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HentAdminCommandRequestV1 {
    pub utfort_av: String,
    pub command_id: Uuid,
}

/// Oppslag av én sak med materialisert lokal state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HentAdminSakRequestV1 {
    pub utfort_av: String,
    pub key: AdminSakKeyV1,
}

/// Nøkkelen saken slås opp med. `skuffen_id` er intern identitet og eksponeres
/// bevisst; den er nødvendig for reparasjon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum AdminSakKeyV1 {
    SkuffenId(Uuid),
    ClientReference(Uuid),
    ArkivId(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn command_request_bruker_snake_case_toppnivaa() {
        let command_id = Uuid::new_v4();
        let request = HentAdminCommandRequestV1 {
            utfort_av: "test-operator".to_string(),
            command_id,
        };

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({ "utfort_av": "test-operator", "command_id": command_id })
        );
    }

    #[test]
    fn sak_request_stotter_alle_tre_nokkelvariantene() {
        let id = Uuid::new_v4();

        for (key, expected) in [
            (
                AdminSakKeyV1::SkuffenId(id),
                json!({ "type": "skuffenId", "value": id }),
            ),
            (
                AdminSakKeyV1::ClientReference(id),
                json!({ "type": "clientReference", "value": id }),
            ),
            (
                AdminSakKeyV1::ArkivId("2026/12345".to_string()),
                json!({ "type": "arkivId", "value": "2026/12345" }),
            ),
        ] {
            let request = HentAdminSakRequestV1 {
                utfort_av: "test-operator".to_string(),
                key: key.clone(),
            };

            assert_eq!(
                serde_json::to_value(&request).unwrap(),
                json!({ "utfort_av": "test-operator", "key": expected })
            );
            assert_eq!(
                serde_json::from_value::<HentAdminSakRequestV1>(
                    json!({ "utfort_av": "test-operator", "key": expected })
                )
                .unwrap()
                .key,
                key
            );
        }
    }

    #[test]
    fn manglende_utfort_av_avvises() {
        let command_id = Uuid::new_v4();
        assert!(
            serde_json::from_value::<HentAdminCommandRequestV1>(
                json!({ "command_id": command_id })
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<HentAdminSakRequestV1>(
                json!({ "key": { "type": "skuffenId", "value": command_id } })
            )
            .is_err()
        );
    }

    #[test]
    fn ukjent_felt_avvises_bade_toppnivaa_og_i_key() {
        let id = Uuid::new_v4();

        assert!(
            serde_json::from_value::<HentAdminCommandRequestV1>(json!({
                "utfort_av": "test-operator",
                "command_id": id,
                "commandId": id
            }))
            .is_err()
        );

        assert!(
            serde_json::from_value::<HentAdminSakRequestV1>(json!({
                "utfort_av": "test-operator",
                "key": { "type": "skuffenId", "value": id, "extra": true }
            }))
            .is_err()
        );
    }
}
