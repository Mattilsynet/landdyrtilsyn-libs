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

#[derive(Debug)]
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
