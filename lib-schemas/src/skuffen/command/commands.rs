use crate::skuffen::command::{
    journalpost::{
        OpprettInngåendeJurnalpost, OpprettInterntNotatJurnalpost, OpprettUgåendeJurnalpost,
    },
    sak::OpprettSak,
};

#[derive(Debug)]
pub enum Kommando {
    OpprettSak(OpprettSak),
    OpprettInngåendeJournalpost(OpprettInngåendeJurnalpost),
    OpprettUtgåendeJournalpost(OpprettUgåendeJurnalpost),
    OpprettInterntNotatJournalpost(OpprettInterntNotatJurnalpost),
}
