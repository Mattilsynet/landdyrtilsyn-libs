//! Admin read-kontrakten for Skuffen.
//!
//! Admin read viser den lokale tilstanden Skuffen faktisk vil bruke dersom en
//! operasjon kjøres på nytt. Den er ikke en hendelsestidslinje og ikke en
//! generell databasekonsoll; klientvendte lifecycle- og avvisningshendelser
//! tilhører status-streamen.
//!
//! Request-typene er strenge: ukjente felt avvises, slik at en skrivefeil i et
//! CLI-kall ikke ignoreres i stillhet. Response-typene er permissive og
//! rapporterer lagrede koder og fritekst som strings, slik at historisk eller
//! reparasjonstrengende state alltid kan vises.

pub mod requests;
pub mod responses;

pub use requests::{AdminSakKeyV1, HentAdminCommandRequestV1, HentAdminSakRequestV1};
pub use responses::{
    AdminCommandResponseV1, AdminCommandUtfallV1, AdminDokumentV1, AdminEntitetIdentitetV1,
    AdminJournalpostV1, AdminKorrespondansepartV1, AdminOperasjonDetaljerV1,
    AdminOperasjonEntitetV1, AdminOperasjonSammendragV1, AdminSakFaktaV1, AdminSakResponseV1,
};
