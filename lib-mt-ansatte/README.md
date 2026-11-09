# lib-mt-ansatte

Slår opp ansatte og enhetsnavn i JetStream-streamen `ansatte`, som er en
replika av Entra.

## Bruk

Sende inn den eksisterende NATS-klienten fra applikasjonen. Biblioteket kobler
ikke til NATS selv og eier ikke tilkobling eller tilgang.

```rust
let ansatte = lib_mt_ansatte::MtAnsatte::new(nats_client.clone());

let ansatt = ansatte.ansatt("99990001").await?;
let navn = ansatte.ansatt_navn("99990001").await?;
let enhet = ansatte.enhet_navn("M12345").await?;
```

- `ansatt` søker på `ansatte.*.<employee-id>` og returnerer hele Entra-brukeren
  (`Ansatt`) fra nyeste melding.
- `ansatt_navn` er `displayName` fra `ansatt`.
- `enhet_navn` går gjennom subjectene på `ansatte.<department-id>.*` og
  returnerer `department` fra første ansatt som har det.

Hvert kall gjør oppslag mot JetStream. Tilkobling, credentials og
NATS-rettigheter til `ansatte` må applikasjonen håndtere.

## Tilgang

Det er team slakt som eier strømmen så bruk krever en export av strømmen fra slakt sin konto og en import i kontoen den skal brukes.
Slakt sitt repo: https://github.com/Mattilsynet/kniv/tree/main/jobs/entra-replikator

## Semantikk

Entra-replikatoren publiserer ansatte på `ansatte.<koststed>.<employee-id>`,
eller `ansatte.ukjent.<employee-id>` uten koststed. Når en ansatt bytter
koststed, får det gamle subjectet en tom melding (header `deleted: true`) før
ansatten publiseres på det nye. En ansatt som forsvinner fra Entra får
`accountEnabled: false` på sitt gjeldende subject.

- Ansattoppslag bruker meldingen med høyest streamsekvens blant ansattens
  subjects, også `ukjent`. Nyeste melding vinner selv om vedkommende har byttet
  enhet.
- Er nyeste ansattmelding tom, er ansatten midt i en flytting, og oppslaget
  feiler. Eldre poster brukes ikke.
- `accountEnabled: false` hindrer ikke oppslag.
- Enhetsoppslag hopper over tomme meldinger (ansatte som har flyttet ut).
- Ingen treff gir `Ok(None)`. Manglende felt eller `null` gir `None` i feltet;
  `ansatt_navn` og `enhet_navn` regner også tom streng som `None`.

Oppslag gjøres med streamens rålese-API (`STREAM.MSG.GET`) og, for enheter,
subjectlisten i `STREAM.INFO`. Biblioteket bruker ikke
consumers og oppretter, endrer eller sletter ikke streamen.
