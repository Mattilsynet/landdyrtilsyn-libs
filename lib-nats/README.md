# lib-nats

Verktøy for NATS/JetStream. Dette biblioteket brukes på tvers av team, og chunked upload er lagt opp som en wire protocol med adskilt sender og mottaker.

## Chunked uploads

NATS har en maksimumsstørrelse for payload, og i praksis gir payload over noen få MB dårlig ytelse. Derfor bruker vi en chunked upload protocol over NATS headers. Chunk size er 2 MB (decimal, base-10) som standard og er begrenset til 8 MB (decimal) for å holde seg under anbefalt 8 MB max_payload og JetStream 32 MiB enforced limit.

### Wire format

Chunked uploads identifiseres av headers. Dette gjør det mulig å sende og motta på tvers av applikasjoner.

Obligatoriske headers:

- `X-Payload-Type`: `chunked-upload`
- `X-Chunked-Upload-Id`: UUID string
- `X-Chunk-Index`: zero-based index
- `X-Chunk-Count`: total chunk count
- `X-Total-Size`: full payload size i bytes

Valgfrie headers:

- `X-Filename`
- `X-Content-Type`

Payload bytes er selve chunk-bytene.

### Rust inngangspunkt

- Sender: `lib-nats/src/chunked_upload/sender.rs` (`publish_chunked_bytes`)
- Mottaker: `lib-nats/src/chunked_upload/receiver.rs` (`ChunkedUploadAssembler`, `UploadLimits`)
- Felles protocol: `lib-nats/src/chunked_upload/protocol.rs`

### Hvordan det fungerer

- Sender deler payload i chunks og publiserer hver chunk til et Subject.
- Hver chunk har protocol headers som beskriver upload id, indeks og total størrelse.
- Mottaker samler chunks per `X-Chunked-Upload-Id` og setter sammen når alle er mottatt.
- Ferdig payload returneres som `ChunkedPayload` med valgfri filename/content-type metadata.

### Limits og TTL

`ChunkedUploadAssembler` validerer payload og beskytter mot ubegrenset minnebruk. Ved feil slettes state for upload-id, og `push` returnerer `Error::FetchError`.

Standardgrenser:

- max upload size: 100 MB
- max chunk count: 2000
- max inflight uploads: 100
- max inflight bytes: 500 MB
- max chunk size: 8 MB (matcher `MAX_CHUNK_SIZE`)
- TTL for incomplete uploads: 10 minutter

Egendefinerte grenser:

```rust
use lib_nats::chunked_upload::receiver::ChunkedUploadAssembler;
use lib_nats::chunked_upload::UploadLimits;

let limits = UploadLimits {
    max_upload_size: 100 * 1024 * 1024,
    max_chunk_count: 2_000,
    max_inflight_uploads: 100,
    max_inflight_bytes: 500 * 1024 * 1024,
    max_chunk_size: 8 * 1024 * 1024,
    ttl: std::time::Duration::from_secs(10 * 60),
};

let mut assembler = ChunkedUploadAssembler::with_limits(limits);
```

## Object Store

Wrapper for NATS JetStream Object Store, med enkle funksjoner for opplasting og nedlasting.

### Rust inngangspunkt

- `lib-nats/src/object_store.rs`

### Eksempler

```rust
use lib_nats::object_store;

let jetstream = lib_nats::create_jetstream_instance(client).await;
let store = object_store::get_or_create_object_store(
    &jetstream,
    async_nats::jetstream::object_store::Config {
        bucket: "saker".to_string(),
        ..Default::default()
    },
)
.await?;

object_store::store_bytes(&store, "fil.txt", b"data").await?;
let bytes = object_store::fetch_bytes(&store, "fil.txt").await?;
```
