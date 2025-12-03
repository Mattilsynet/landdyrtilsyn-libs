use crate::skuffen::command::journalpost::{
    OpprettInngåendeJurnalpost, OpprettInterntNotatJurnalpost, OpprettUgåendeJurnalpost,
};

#[derive(Debug)]
pub enum Kommando {
    OpprettSak, //TODO
    OpprettInngåendeJournalpost(OpprettInngåendeJurnalpost),
    OpprettUtgåendeJournalpost(OpprettUgåendeJurnalpost),
    OpprettInterntNotatJournalpost(OpprettInterntNotatJurnalpost),
}
