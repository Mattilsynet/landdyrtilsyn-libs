# lib-schemas

Shared schema types for Mattilsynet libraries. Denne craten gir serializable
domain-objekter, valideringshelpers, og Command/Query payloads brukt av tjenester
som integrerer med Skuffen og relaterte systemer.

## Modules

- `skuffen`: Commands, queries, og delte domain types brukt av Skuffen-API.
- `typer`: Common identifier/value types med valideringshelpers.
- `error`: Delte error types brukt ved parsing og validering.

## Usage

Enable `skuffen` when importing the crate if you need Skuffen domain types:

```toml
lib-schemas = { version = "0.1", features = ["skuffen"] }
```

```rust
use lib_schemas::skuffen::sak::Saksnummer;

let saksnummer = Saksnummer::new("2025/ABC123")?;
assert_eq!(saksnummer.year(), 2025);
```

```rust
use lib_schemas::typer::personnummer::Personnummer;

let pnr = Personnummer::new("01010101006")?;
assert_eq!(pnr.as_str(), "01010101006");
```

## Error handling

De fleste parsing helpers returnerer `lib_schemas::error::Result<T>` som wrapper
`SchemasError` og gir varianter for validerings- og parse-feil.
