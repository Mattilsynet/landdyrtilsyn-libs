use async_nats::HeaderMap;
use async_nats::header::HeaderValue;
use uuid::Uuid;

use crate::error::{Error, Result};

pub const DEFAULT_CHUNK_SIZE: usize = 2_000_000;
pub const MAX_CHUNK_SIZE: usize = 8_000_000;
pub const HEADER_UPLOAD_ID: &str = "X-Chunked-Upload-Id";
pub const HEADER_CHUNK_INDEX: &str = "X-Chunk-Index";
pub const HEADER_CHUNK_COUNT: &str = "X-Chunk-Count";
pub const HEADER_TOTAL_SIZE: &str = "X-Total-Size";
pub const HEADER_FILENAME: &str = "X-Filename";
pub const HEADER_CONTENT_TYPE: &str = "X-Content-Type";
pub const HEADER_PAYLOAD_TYPE: &str = "X-Payload-Type";
pub const PAYLOAD_TYPE_CHUNK: &str = "chunked-upload";

#[derive(Debug, Clone)]
pub struct UploadMetadata {
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChunkedPayload {
    pub upload_id: String,
    pub data: Vec<u8>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ChunkedUploadConfig {
    pub chunk_size: usize,
}

impl Default for ChunkedUploadConfig {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }
}

impl ChunkedUploadConfig {
    pub fn with_chunk_size(chunk_size: usize) -> Result<Self> {
        if chunk_size == 0 {
            return Err(Error::ConfigError(
                "Chunk size must be greater than 0".to_string(),
            ));
        }
        if chunk_size > MAX_CHUNK_SIZE {
            return Err(Error::ConfigError(format!(
                "Chunk size {} exceeds max {}",
                chunk_size, MAX_CHUNK_SIZE
            )));
        }
        Ok(Self { chunk_size })
    }
}

#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub upload_id: String,
    pub chunk_index: usize,
    pub chunk_count: u32,
    pub total_size: usize,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

pub fn build_chunk_headers(
    upload_id: &str,
    chunk_index: u32,
    chunk_count: u32,
    total_size: usize,
    metadata: &UploadMetadata,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(HEADER_PAYLOAD_TYPE, PAYLOAD_TYPE_CHUNK);
    headers.insert(HEADER_UPLOAD_ID, upload_id);
    headers.insert(HEADER_CHUNK_INDEX, HeaderValue::from(chunk_index));
    headers.insert(HEADER_CHUNK_COUNT, HeaderValue::from(chunk_count));
    headers.insert(HEADER_TOTAL_SIZE, HeaderValue::from(total_size));
    if let Some(filename) = metadata.filename.as_ref() {
        headers.insert(HEADER_FILENAME, filename.as_str());
    }
    if let Some(content_type) = metadata.content_type.as_ref() {
        headers.insert(HEADER_CONTENT_TYPE, content_type.as_str());
    }
    headers
}

pub fn parse_chunk_info(headers: &HeaderMap) -> Result<Option<ChunkInfo>> {
    let payload_type = headers
        .get_last(HEADER_PAYLOAD_TYPE)
        .map(|value| value.as_str());

    if payload_type != Some(PAYLOAD_TYPE_CHUNK) {
        return Ok(None);
    }

    let upload_id = read_header_uuid(headers, HEADER_UPLOAD_ID)?;
    let chunk_count = read_header_u32(headers, HEADER_CHUNK_COUNT)?;
    let total_size = read_header_usize(headers, HEADER_TOTAL_SIZE)?;
    let chunk_index = read_header_usize(headers, HEADER_CHUNK_INDEX)?;
    let filename = headers
        .get_last(HEADER_FILENAME)
        .map(|value| value.as_str().to_string());
    let content_type = headers
        .get_last(HEADER_CONTENT_TYPE)
        .map(|value| value.as_str().to_string());

    Ok(Some(ChunkInfo {
        upload_id,
        chunk_index,
        chunk_count,
        total_size,
        filename,
        content_type,
    }))
}

pub fn is_chunked_headers(headers: &HeaderMap) -> bool {
    headers
        .get_last(HEADER_PAYLOAD_TYPE)
        .map(|value| value.as_str())
        == Some(PAYLOAD_TYPE_CHUNK)
}

pub fn split_payload(payload: &[u8], chunk_size: usize) -> Result<Vec<Vec<u8>>> {
    if chunk_size == 0 {
        return Err(Error::ConfigError(
            "Chunk size must be greater than 0".to_string(),
        ));
    }
    if payload.is_empty() {
        return Err(Error::PublishError("Payload must not be empty".to_string()));
    }
    Ok(payload
        .chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect())
}

pub fn assemble_chunks(
    chunks: &[Option<Vec<u8>>],
    chunk_count: u32,
    total_size: usize,
) -> Result<Vec<u8>> {
    let mut assembled = Vec::with_capacity(total_size);
    for chunk in chunks.iter().take(chunk_count as usize) {
        let chunk = chunk
            .as_ref()
            .ok_or_else(|| Error::FetchError("Missing chunk".to_string()))?;
        assembled.extend_from_slice(chunk);
    }
    Ok(assembled)
}

fn read_header(headers: &HeaderMap, key: &str) -> Result<String> {
    headers
        .get_last(key)
        .map(|value| value.as_str().to_string())
        .ok_or_else(|| Error::FetchError(format!("Missing header {}", key)))
}

fn read_header_uuid(headers: &HeaderMap, key: &str) -> Result<String> {
    let value = read_header(headers, key)?;
    Uuid::parse_str(&value)
        .map_err(|_| Error::FetchError(format!("Invalid UUID for header {}", key)))
        .map(|uuid| uuid.to_string())
}

fn read_header_u32(headers: &HeaderMap, key: &str) -> Result<u32> {
    let value = read_header(headers, key)?;
    value
        .parse()
        .map_err(|_| Error::FetchError(format!("Failed to parse header {} value {}", key, value)))
}

fn read_header_usize(headers: &HeaderMap, key: &str) -> Result<usize> {
    let value = read_header(headers, key)?;
    value
        .parse()
        .map_err(|_| Error::FetchError(format!("Failed to parse header {} value {}", key, value)))
}
