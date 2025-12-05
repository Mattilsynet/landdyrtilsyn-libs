pub mod error;
pub mod geonorge_client;
pub mod response;

pub use error::{GeonorgeError, Result};
pub use geonorge_client::GeoNorgeClient;
pub use response::{AddressResult, GeonorgeResponse, Koordinater};
