//! Shared schema types brukt på tvers av Mattilsynet services.
//!
//! Inneholder domain-objekter, Command/Query payloads, og valideringshelpers.
pub mod error;
pub mod typer;

#[cfg(feature = "skuffen")]
pub mod skuffen;
