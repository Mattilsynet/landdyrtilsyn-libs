use serde::{Deserialize, Serialize};

use crate::arkiv::v2::{journalpost::JournalpostId, sak::Saksnummer};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Query {
    HentSak { saksnummer: Saksnummer },
    HentJournalpost { journalpost_id: JournalpostId },
}
