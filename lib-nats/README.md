# lib-nats

Verktoy for Core NATS og JetStream.

## Sessionbasert chunked upload

Protokollen bruker Core NATS request/reply og er laget for idempotent handtering av Core NATS sin at-most-once-levering. Base-subject og begin-queue er konfigurerbare; standardene er `chunked-upload` og `chunked-upload-receivers`.

### Wire-kontrakt

Subjects for `{base}`:

- `{base}.begin`
- `{base}.receiver.{receiver_id}.session.{session_id}.chunk.{index}`
- `{base}.receiver.{receiver_id}.session.{session_id}.commit`

Begin er UTF-8 JSON:

```json
{"upload_id":"media-123","size":2500123,"sha256":"<64 lowercase hex>","filename":"optional.pdf","content_type":"application/pdf"}
```

Svar er tagget med `status`:

- `{"status":"ready","receiver_id":"<uuid>","session_id":"<uuid>","chunk_size":2000000,"session_ttl_ms":600000}`
- `{"status":"already_stored","upload_id":"media-123"}`
- `{"status":"error","code":"<stable_code>"}`

Chunks sendes sekvensielt, en request om gangen, som ra bytes. Indeksen ligger i subject. Svar er `{"status":"accepted","index":0}` eller et error-svar. Identiske duplikater aksepteres; konfliktende duplikater, feil rekkefolge og feil chunklengde avvises. Error ved feil rekkefolge inkluderer `expected_index`.

Commit har tom payload. Serveren verifiserer full storrelse og SHA-256 for den kaller den durable `UploadStore`. Svar er `{"status":"stored","upload_id":"media-123"}` bare etter vellykket, idempotent lagring. Et bounded tombstone-vindu gjor commit-retry trygt nar svaret forsvinner. Ukjente JSON-felt avvises. Rust-feiltekst sendes aldri pa wire.

### Request/reply og utrulling

Hver server oppretter en UUIDv4 `receiver_id`, subscriber uten queue pa sitt receiver-subject og flusher dette for den joiner begin-queue. Bare begin bruker queue group. Session-state finnes for `ready` sendes.

Klienten retryer samme chunk ved timeout for den starter en ny session fra begynnelsen. Commit retryes fordi timeout er tvetydig; deretter gjores ny begin, og store-inspect avgjor `already_stored`. Nar en receiver forsvinner, velges en ny via begin-queue. Core NATS antas ikke a vaere durable.

Ved shutdown stoppes begin-subscription forst. Receiver-subscription betjenes i `shutdown_grace`, slik at pagaende sessions kan fullfores. Ved rolling deploy kan etablerte sessions derfor bli pa sin receiver, mens nye sessions fordeles til resterende replicas. Denne protokollen har ingen v1/v2-token og er ikke wire-kompatibel med den gamle headerprotokollen; bruk et nytt base-subject dersom gamle og nye responders ma leve samtidig.

### Rust-API

```rust,no_run
use std::sync::Arc;
use bytes::Bytes;
use lib_nats::chunked_upload::{
    ChunkedUploadClient, ChunkedUploadClientConfig, ChunkedUploadServer,
    ChunkedUploadServerConfig, UploadRequest, UploadStore,
};

# async fn example(
#     nats: async_nats::Client,
#     store: Arc<dyn UploadStore>,
# ) -> Result<(), Box<dyn std::error::Error>> {
let server = ChunkedUploadServer::new(
    nats.clone(),
    ChunkedUploadServerConfig::default(),
    store,
)?;
tokio::spawn(server.run(std::future::pending()));

let client = ChunkedUploadClient::new(nats, ChunkedUploadClientConfig::default());
let receipt = client.upload(UploadRequest {
    upload_id: "media-123".to_string(),
    bytes: Bytes::from_static(b"data"),
    filename: None,
    content_type: Some("application/octet-stream".to_string()),
}).await?;
# let _ = receipt;
# Ok(())
# }
```

`UploadStore` er en objekt-safe async trait med `inspect(upload_id)` og `store(CompletedUpload)`. Store-implementasjonen ma lagre atomisk og idempotent pa `upload_id`: samme storrelse/digest skal lykkes, mens annet innhold for samme ID skal gi `UploadStoreError::Conflict`.

### Standardgrenser

- Chunk size: 2 000 000 bytes, maksimum 8 000 000
- Upload size: 100 MiB
- Aktive sessions: 100
- Reserverte bytes: 500 MiB
- Session TTL: 10 minutter, med periodisk opprydding
- Commit tombstones: 1 000 i 10 minutter
- Upload ID: 256 bytes, filename: 1 024 bytes, content type: 255 bytes
- Shutdown grace: 5 sekunder

NATS-serverens `max_payload` ma vaere minst konfigurert chunk size; standard chunk size krever dermed minst 2 000 000 bytes.

### Cross-account

Eksporter begge Core NATS service-subjects fra serverkontoen:

- `{base}.begin`
- `{base}.receiver.>`

Importer begge til klientkontoen med samme lokale `{base}`-prefiks. Dynamiske receiver/session-subjects ma ikke omskrives ulikt mellom de to importene. Vanlige NATS service-import replies brukes for request/reply.

## Object Store

`object_store.rs` wrapper NATS JetStream Object Store for opplasting og nedlasting av bytes.
