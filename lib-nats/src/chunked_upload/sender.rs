use async_nats::jetstream::Context;
use async_nats::jetstream::context::PublishErrorKind;
use async_nats::jetstream::message::PublishMessage;
use bytes::Bytes;
use uuid::Uuid;

use crate::chunked_upload::protocol::{ChunkedUploadConfig, UploadMetadata, build_chunk_headers};
use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub upload_id: String,
    pub chunk_count: u32,
    pub total_size: usize,
}

pub async fn publish_chunked_bytes(
    context: &Context,
    subject: String,
    payload: &[u8],
    metadata: UploadMetadata,
    config: ChunkedUploadConfig,
) -> Result<UploadResult> {
    let chunk_size = config.chunk_size;
    if payload.is_empty() {
        return Err(Error::PublishError("Payload must not be empty".to_string()));
    }

    let subject = subject;
    let upload_id = Uuid::new_v4().to_string();
    let total_size = payload.len();
    let chunk_count = ((total_size + chunk_size - 1) / chunk_size) as u32;

    for (index, chunk) in payload.chunks(chunk_size).enumerate() {
        publish_chunk(
            context,
            subject.clone(),
            chunk,
            &upload_id,
            index as u32,
            chunk_count,
            total_size,
            &metadata,
        )
        .await?;
    }

    Ok(UploadResult {
        upload_id: upload_id.clone(),
        chunk_count,
        total_size,
    })
}

async fn publish_chunk(
    context: &Context,
    subject: String,
    payload: &[u8],
    upload_id: &str,
    chunk_index: u32,
    chunk_count: u32,
    total_size: usize,
    metadata: &UploadMetadata,
) -> Result<()> {
    let headers = build_chunk_headers(upload_id, chunk_index, chunk_count, total_size, metadata);
    let payload_bytes = Bytes::from(payload.to_vec());
    let publish = PublishMessage::build()
        .headers(headers)
        .payload(payload_bytes);
    context
        .send_publish(subject, publish)
        .await
        .map_err(|err| match err.kind() {
            PublishErrorKind::TimedOut => {
                Error::PublishError("Chunk publish timed out".to_string())
            }
            _ => Error::PublishError(err.to_string()),
        })?
        .await
        .map_err(|err| Error::PublishError(err.to_string()))?;
    Ok(())
}
