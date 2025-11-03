pub mod arkiv;
pub mod error;
pub mod typer;

#[cfg(feature = "arkiv")]
pub use arkiv::{
    Landkode, SaksTittel, Saksaar, sak, tilgangshjemmel::Tilgangshjemmel,
    tilgangskoder::Tilgangskode,
};
