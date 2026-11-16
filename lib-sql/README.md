# lib-sql

Oppretter og konfigurerer en Postgres connection pool fra miljøvariabler.

`DbPool` er fortsatt samme `Pool<Postgres>`-type som SQLx eksponerer, og
`get_database_pool` returnerer fortsatt samme SQLx-feiltype.

Biblioteket bruker `sqlx-core` og `sqlx-postgres` direkte, uten SQLx-fasaden.
Dette holder SQLite og ubrukte SQLx-macro-avhengigheter ute av bibliotekets
dependency-graf. Tokio og Rustls med ring/webpki-roots beholdes.

Applikasjoner som bruker SQLx-makroer, for eksempel `sqlx::migrate!`, må
aktivere `macros` på sin egen SQLx-avhengighet. Biblioteket aktiverer ikke
den featuren indirekte.
