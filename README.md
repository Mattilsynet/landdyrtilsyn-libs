# landdyrtilsyn-libs
Felles kode for team landdyrtilsyn


## Hvordan ta ibruk
For og ta ibruk bibliotek her så må du legge til følgende linje i [dependecies] Cargo.toml filen din:
```toml
# Erstatt "pakkenavn" med navnet du vil gi pakken i prosjektet ditt
pakkenavn = { git = "ssh://git@github.com/Mattilsynet/landdyrtilsyn-libs" }
```
pakkenavn er navnet du vil gi pakken i ditt prosjekt, freks protobuf.

I tillegg må du legge til følgende linje i .zshrc filen din:
```bash
export CARGO_NET_GIT_FETCH_WITH_CLI=true
```

## Lokal sjekk av hele repoet

For å sjekke alle crates lokalt, kjør denne kommandoen fra repo-roten:

```bash
cargo check-all
```

Dette er en alias til:

```bash
cargo check --workspace --all-targets --all-features
```

## Før commit og push

Repoet installerer ikke git-hooks. Kjør kontrollene eksplisitt:

```bash
gitleaks git --staged --redact                              # før commit
gitleaks git --redact --log-opts="HEAD --not --remotes"      # før push
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo deny check advisories sources
cargo audit
```

Endringer som berører sikkerhetsrelevante flater gjennomgås også med uavhengig
security review via OpenCode. `deny.toml` og `.cargo/audit.toml` er fortsatt
aktive sikkerhetspolicyer.

## Hendvendelser

Spørsmål knyttet til koden eller prosjektet kan stilles som issues her på GitHub

# For Mattilsynet-ansatte
Interne hendvendelser kan sendes via slack i kanalen #landdyrtilsyn
