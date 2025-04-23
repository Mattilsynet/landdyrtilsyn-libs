pub mod arkiv;
pub mod auth;
pub mod bilde;
pub mod client;
pub mod config;
pub mod document_generator;
pub mod error;
pub mod kodeverk;
pub mod orgenhet;

#[cfg(feature = "kodeverk")]
pub use kodeverk::{
    kodeverk_client::KodeverkClient, response::KodeverkResponse, response::RelatedCode,
};

#[cfg(feature = "dokument_generator")]
pub use document_generator::{
    document_generator_client::DokumentGeneratorClient, response::Avsender,
    response::InterntDokument, response::InterntDokumentBuilder, response::VedleggDokument,
};

#[cfg(feature = "orgenhet")]
pub use orgenhet::{orgenhet_client::OrgEnhetClient, response::*};

#[cfg(feature = "bilde")]
pub use bilde::{bilde_client::BildeClient, response::ImageMetaData};

#[cfg(feature = "arkiv")]
pub use arkiv::{
    arkiv_client::ArkivClient, response::ArkivClientJournalpost, response::ArkivClientSak,
};
