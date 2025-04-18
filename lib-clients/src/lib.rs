pub mod auth;
pub mod bilde;
pub mod client;
pub mod config;
pub mod error;
pub mod orgenhet;

#[cfg(feature = "orgenhet")]
pub use orgenhet::{orgenhet_client::OrgEnhetClient, response::*};

#[cfg(feature = "bilde")]
pub use bilde::{bilde_client::BildeClient, response::ImageMetaData};
