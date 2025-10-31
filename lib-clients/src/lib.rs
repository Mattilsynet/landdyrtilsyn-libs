pub mod arkiv;
pub mod auth;
pub mod bilde;
pub mod client;
pub mod config;
pub mod document_generator;
pub mod ejb;
pub mod error;
pub mod kodeverk;
pub mod koordinat;
pub mod orgenhet;
pub mod tilsynskvittering;
pub mod virksomhet;

#[cfg(feature = "kodeverk")]
pub use kodeverk::{
    kodeverk_client::CodeParams, kodeverk_client::KodeverkClient, response::Code,
    response::KodeverkResponse, response::RelatedCode,
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
    response::Kodeverk,
};

#[cfg(feature = "virksomhet")]
pub use virksomhet::{
    respone::Underenhet, response::Virksomhet, virksomhet_client::VirksomhetClient,
};

#[cfg(feature = "tilsynskvittering")]
pub use tilsynskvittering::{
    response::TilsynsobjektKvittering, tilsynskvittering_client::TilsynskvitteringClient,
};

#[cfg(feature = "ejb")]
pub use ejb::{
    ejb_client::EjbClient, response_begrensninger::Begrensning, response_tilfeller::Sykdomstilfelle,
};

#[cfg(feature = "koordinat")]
pub use koordinat::{
    koordinat_client::KoordinatClient, response::AddressResult, response::GeonorgeResponse,
    response::Koordinater,
};
