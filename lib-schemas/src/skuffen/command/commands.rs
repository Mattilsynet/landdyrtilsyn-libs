use crate::error::{Result, SchemasError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::command::{
    journalpost::{
        OpprettInngåendeJournalpost, OpprettInterntNotatJournalpost, OpprettUgåendeJournalpost,
    },
    sak::{AvsluttSak, OpprettSak},
};

/// Commands støttet av Skuffen command API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Command {
    OpprettSak(OpprettSak),
    OpprettInngåendeJournalpost(OpprettInngåendeJournalpost),
    OpprettUtgåendeJournalpost(OpprettUgåendeJournalpost),
    OpprettInterntNotatJournalpost(OpprettInterntNotatJournalpost),
    AvsluttSak(AvsluttSak),
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
