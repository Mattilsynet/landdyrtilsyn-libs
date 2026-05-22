use crate::error::{Result, SchemasError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::command::{
    journalpost::{
        OpprettInngåendeJournalpost, OpprettInterntNotatJournalpost, OpprettUgåendeJournalpost,
    },
    sak::{AvsluttSak, OpprettSak, SettSaksansvarlig},
};

/// Commands støttet av Skuffen command API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Command {
    OpprettSak(OpprettSak),
    OpprettInngåendeJournalpost(OpprettInngåendeJournalpost),
    OpprettUtgåendeJournalpost(OpprettUgåendeJournalpost),
    OpprettInterntNotatJournalpost(OpprettInterntNotatJournalpost),
    AvsluttSak(AvsluttSak),
    SettSaksansvarlig(SettSaksansvarlig),
}

/// Envelope for commands inkl. ids brukt for idempotency og tracing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandEnvelope<T> {
    pub command_id: Uuid,
    pub correlation_id: Option<Uuid>,
    pub payload: T,
}

/// Non-empty liste med commands som kjøres som en sequence.
pub struct CommandSequence(Vec<CommandEnvelope<Command>>);

impl CommandSequence {
    /// Lag en sequence, reject empty lists.
    pub fn new(commands: Vec<CommandEnvelope<Command>>) -> Result<Self> {
        if commands.is_empty() {
            return Err(SchemasError::ValidationError(
                "Command sequence must have at least 1 command".to_string(),
            ));
        }
        Ok(Self(commands))
    }

    /// Returner underlying vector av command envelopes.
    pub fn into_inner(self) -> Vec<CommandEnvelope<Command>> {
        self.0
    }
}

impl TryFrom<Vec<CommandEnvelope<Command>>> for CommandSequence {
    type Error = SchemasError;

    fn try_from(value: Vec<CommandEnvelope<Command>>) -> Result<Self> {
        Self::new(value)
    }
}

impl IntoIterator for CommandSequence {
    type Item = CommandEnvelope<Command>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Command execution status values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CommandStatus {
    Pending,
    Blocked,
    Retrying,
    Ok,
    Error,
}

/// Receipt fra request-reply: validation/idempotency sjekket og command er
/// accepted (Ok) eller rejected (Error). Dette er ikke execution result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CommandReceipt {
    Ok { command_id: Uuid },
    Error { message: String, command_id: Uuid },
}

/// Kvittering fra `arkiv.arkiver` request-reply.
///
/// Bruker bevisst serde default externally tagged enum-shape, f.eks.
/// `{ "Ok": { "command_ids": [...] } }`, slik at wire-formatet speiler
/// Rust-varianten direkte. Dette avviker fra `CommandReceipt`, som er en
/// separat singular receipt-type.
///
/// `Ok` betyr at hele command sequence er mottatt og akseptert for videre
/// prosessering. `command_ids` inneholder alle command ids i den aksepterte
/// sekvensen i innsendt rekkefølge. `Error` betyr at hele sekvensen ble
/// avvist; ingen partial acceptance uttrykkes i denne typen.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArkiveringKvittering {
    Ok { command_ids: Vec<Uuid> },
    Error { message: String },
}

/// Asynkrone command status updates publisert på JetStream.
/// Brukes for å følge lifecycle events (validation, execution, retry, blocked, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandStatusEvent {
    pub command_id: Uuid,
    pub status: CommandStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempt: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::ArkiveringKvittering;
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn arkivering_kvittering_ok_uses_default_externally_tagged_shape() {
        let first_command_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let second_command_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        let kvittering = ArkiveringKvittering::Ok {
            command_ids: vec![first_command_id, second_command_id],
        };

        let value = serde_json::to_value(&kvittering).unwrap();

        assert_eq!(
            value,
            json!({
                "Ok": {
                    "command_ids": [
                        "00000000-0000-0000-0000-000000000001",
                        "00000000-0000-0000-0000-000000000002"
                    ]
                }
            })
        );

        let roundtrip: ArkiveringKvittering = serde_json::from_value(value).unwrap();
        assert_eq!(roundtrip, kvittering);
    }

    #[test]
    fn arkivering_kvittering_error_uses_default_externally_tagged_shape() {
        let kvittering = ArkiveringKvittering::Error {
            message: "Invalid command sequence".to_string(),
        };

        let value = serde_json::to_value(&kvittering).unwrap();

        assert_eq!(
            value,
            json!({
                "Error": {
                    "message": "Invalid command sequence"
                }
            })
        );

        let roundtrip: ArkiveringKvittering = serde_json::from_value(value).unwrap();
        assert_eq!(roundtrip, kvittering);
    }
}
