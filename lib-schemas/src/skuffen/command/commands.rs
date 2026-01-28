use crate::error::{Result, SchemasError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::{
    command::{
        journalpost::{
            OpprettInngåendeJournalpost, OpprettInterntNotatJournalpost, OpprettUgåendeJournalpost,
        },
        sak::OpprettSak,
    },
    query::queries::SakKey,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Kommando {
    OpprettSak(OpprettSak),
    OpprettInngåendeJournalpost(OpprettInngåendeJournalpost),
    OpprettUtgåendeJournalpost(OpprettUgåendeJournalpost),
    OpprettInterntNotatJournalpost(OpprettInterntNotatJournalpost),
    AvsluttSak(SakKey),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandEnvelope<T> {
    pub command_id: Uuid,
    pub correlation_id: Option<Uuid>,
    pub payload: T,
}

pub struct CommandSequence(Vec<CommandEnvelope<Kommando>>);

impl CommandSequence {
    pub fn new(commands: Vec<CommandEnvelope<Kommando>>) -> Result<Self> {
        if commands.is_empty() {
            return Err(SchemasError::ValidationError(
                "Command sequence must have at least 1 command".to_string(),
            ));
        }
        Ok(Self(commands))
    }

    pub fn into_inner(self) -> Vec<CommandEnvelope<Kommando>> {
        self.0
    }
}

impl TryFrom<Vec<CommandEnvelope<Kommando>>> for CommandSequence {
    type Error = SchemasError;

    fn try_from(value: Vec<CommandEnvelope<Kommando>>) -> Result<Self> {
        Self::new(value)
    }
}

impl IntoIterator for CommandSequence {
    type Item = CommandEnvelope<Kommando>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
