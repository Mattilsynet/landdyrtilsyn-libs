pub mod arkiv;
pub mod error;

#[cfg(feature = "arkiv")]
pub use arkiv::{
    Landkode, SaksTittel, Saksaar, sak, tilgangshjemmel::Tilgangshjemmel,
    tilgangskoder::Tilgangskode,
};
