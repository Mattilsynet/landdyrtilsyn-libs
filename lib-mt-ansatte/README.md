# lib-mt-ansatte

Slår opp ansattnavn og enhetsnavn i JetStream-streamen `ansatte`.

## Bruk

Sende inn den eksisterende NATS-klienten fra applikasjonen. Biblioteket kobler
ikke til NATS selv og eier ikke tilkobling eller tilgang.

```rust
let ansatte = lib_mt_ansatte::MtAnsatte::new(nats_client.clone());

let navn = ansatte.ansatt_navn("99990001").await?;
let enhet = ansatte.enhet_navn("M12345").await?;
```

`ansatt_navn` søker på `ansatte.*.<employee-id>` og returnerer `displayName`.
`enhet_navn` søker på `ansatte.<department-id>.*` og returnerer `department`.

Hvert kall gjør et oppslag mot JetStream. Tilkobling, credentials og
NATS-rettigheter til `ansatte` må applikasjonen håndtere.

## Semantikk

JetStream returnerer meldingen med høyest streamsekvens blant alle subjects som
matcher det gitte filteret. Følgende er derfor oppsettet:

- Nyeste ansattmelding vinner selv om vedkommende har byttet enhet.
- En nyere melding på `ansatte.ukjent.*` vinner for ansattnavn.
- Enhetsoppslag returnerer enhet fra den nyeste meldingen tilknyttet enheten.
- `accountEnabled: false` hindrer ikke oppslag.
- Ingen treff gir `Ok(None)`. Manglende felt, `null` eller tomt navnfelt gir
  også `Ok(None)`.

Oppslag gjøres med streamens rålese-API (`STREAM.MSG.GET`), ikke consumers.
Biblioteket oppretter, endrer eller sletter ikke streamen.
