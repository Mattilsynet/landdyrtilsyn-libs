use crate::error::{Result, SchemasError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::command::{
    journalpost::{
        OpprettInngåendeJournalpost, OpprettInterntNotatJournalpost, OpprettUgåendeJournalpost,
    },
    sak::{AvsluttSak, OpprettSak},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Command {
    OpprettSak(OpprettSak),
    OpprettInngåendeJournalpost(OpprettInngåendeJournalpost),
    OpprettUtgåendeJournalpost(OpprettUgåendeJournalpost),
    OpprettInterntNotatJournalpost(OpprettInterntNotatJournalpost),
    AvsluttSak(AvsluttSak),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandEnvelope<T> {
    pub command_id: Uuid,
    pub correlation_id: Option<Uuid>,
    pub payload: T,
}

pub struct CommandSequence(Vec<CommandEnvelope<Command>>);

impl CommandSequence {
    pub fn new(commands: Vec<CommandEnvelope<Command>>) -> Result<Self> {
        if commands.is_empty() {
            return Err(SchemasError::ValidationError(
                "Command sequence must have at least 1 command".to_string(),
            ));
        }
        Ok(Self(commands))
    }

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CommandStatus {
    Pending,
    Blocked,
    Retrying,
    Ok,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
/// Kvittering fra request-reply: validering/idempotens sjekket og kommandoen er
/// enten akseptert (Ok) eller avvist (Error). Dette er ikke utførelsesresultat.
pub enum CommandReceipt {
    Ok { command_id: Uuid },
    Error { message: String, command_id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Asynkrone statusoppdateringer for en kommando som sendes på JetStream.
/// Brukes for å følge livssyklusen (validering, kjøring, retry, blokkert, osv.).
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
