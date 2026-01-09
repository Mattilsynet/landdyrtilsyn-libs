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
