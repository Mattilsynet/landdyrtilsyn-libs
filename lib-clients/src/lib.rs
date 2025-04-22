pub mod auth;
pub mod bilde;
pub mod client;
pub mod config;
pub mod error;
pub mod orgenhet;
pub mod arkiv;

#[cfg(feature = "orgenhet")]
pub use orgenhet::{orgenhet_client::OrgEnhetClient, response::*};

#[cfg(feature = "bilde")]
pub use bilde::{bilde_client::BildeClient, response::ImageMetaData};

#[cfg(feature = "arkiv")]
pub use arkiv::{arkiv_client::ArkivClient, response::ArkivClientJournalpost, response::ArkivClientSak};
