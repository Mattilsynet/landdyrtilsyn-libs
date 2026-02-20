use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_nats::Message;

use crate::chunked_upload::protocol::{
    assemble_chunks, parse_chunk_info, ChunkInfo, ChunkedPayload, MAX_CHUNK_SIZE,
};
use crate::error::{Error, Result};

const DEFAULT_MAX_UPLOAD_SIZE_BYTES: usize = 100 * 1024 * 1024;
const DEFAULT_MAX_INFLIGHT_UPLOADS: usize = 100;
const DEFAULT_MAX_INFLIGHT_BYTES: usize = 500 * 1024 * 1024;
const DEFAULT_MAX_CHUNK_COUNT: u32 = 2_000;
const DEFAULT_UPLOAD_TTL: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone, Copy)]
pub struct UploadLimits {
    pub max_upload_size: usize,
    pub max_chunk_count: u32,
    pub max_inflight_uploads: usize,
    pub max_inflight_bytes: usize,
    pub max_chunk_size: usize,
    pub ttl: Duration,
}

impl Default for UploadLimits {
    fn default() -> Self {
        Self {
            max_upload_size: DEFAULT_MAX_UPLOAD_SIZE_BYTES,
            max_chunk_count: DEFAULT_MAX_CHUNK_COUNT,
            max_inflight_uploads: DEFAULT_MAX_INFLIGHT_UPLOADS,
            max_inflight_bytes: DEFAULT_MAX_INFLIGHT_BYTES,
            max_chunk_size: MAX_CHUNK_SIZE,
            ttl: DEFAULT_UPLOAD_TTL,
        }
    }
}

#[derive(Debug)]
pub struct ChunkedUploadAssembler {
    uploads: HashMap<String, PendingUpload>,
    inflight_bytes: usize,
    limits: UploadLimits,
}

impl ChunkedUploadAssembler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limits(limits: UploadLimits) -> Self {
        Self {
            uploads: HashMap::new(),
            inflight_bytes: 0,
            limits,
        }
    }

    pub fn push(&mut self, message: &Message) -> Result<Option<ChunkedPayload>> {
        let info = match message.headers.as_ref() {
            Some(headers) => match parse_chunk_info(headers)? {
                Some(info) => info,
                None => return Ok(None),
            },
            None => return Ok(None),
        };

        self.prune_expired();
        if let Err(err) = self.validate_header(&info, message.payload.len()) {
            self.remove(&info.upload_id);
            return Err(err);
        }

        let now = Instant::now();
        let entry = self
            .uploads
            .entry(info.upload_id.clone())
            .or_insert_with(|| PendingUpload::new(&info, now));

        if !entry.matches(&info) {
            self.remove(&info.upload_id);
            return Err(Error::FetchError("Chunk metadata mismatch".to_string()));
        }
        if info.chunk_index >= entry.chunks.len() {
            self.remove(&info.upload_id);
            return Err(Error::FetchError("Chunk index out of range".to_string()));
        }
        if entry.seen[info.chunk_index] {
            entry.last_seen = now;
            return Ok(None);
        }
        if entry.received_bytes + message.payload.len() > entry.total_size {
            self.remove(&info.upload_id);
            return Err(Error::FetchError(
                "Received data exceeds declared total size".to_string(),
            ));
        }
        if self.inflight_bytes + message.payload.len() > self.limits.max_inflight_bytes {
            self.remove(&info.upload_id);
            return Err(Error::FetchError(
                "Server in-flight memory limit reached".to_string(),
            ));
        }

        entry.chunks[info.chunk_index] = Some(message.payload.to_vec());
        entry.seen[info.chunk_index] = true;
        entry.received += 1;
        entry.received_bytes += message.payload.len();
        entry.last_seen = now;
        self.inflight_bytes += message.payload.len();

        if entry.received == entry.chunk_count as usize {
            let assembled = assemble_chunks(&entry.chunks, entry.chunk_count, entry.total_size)?;
            if assembled.len() != entry.total_size {
                self.remove(&info.upload_id);
                return Err(Error::FetchError(
                    "Assembled size does not match declared total".to_string(),
                ));
            }

            let payload = ChunkedPayload {
                upload_id: info.upload_id.clone(),
                data: assembled,
                filename: entry.filename.clone(),
                content_type: entry.content_type.clone(),
            };
            self.remove(&info.upload_id);
            return Ok(Some(payload));
        }

        Ok(None)
    }

    pub fn remove(&mut self, upload_id: &str) {
        if let Some(state) = self.uploads.remove(upload_id) {
            self.inflight_bytes = self.inflight_bytes.saturating_sub(state.received_bytes);
        }
    }

    fn prune_expired(&mut self) {
        let now = Instant::now();
        let expired: Vec<String> = self
            .uploads
            .iter()
            .filter(|(_, state)| now.duration_since(state.last_seen) > self.limits.ttl)
            .map(|(upload_id, _)| upload_id.clone())
            .collect();

        for upload_id in expired {
            self.remove(&upload_id);
        }
    }

    fn validate_header(&self, info: &ChunkInfo, chunk_len: usize) -> Result<()> {
        if info.chunk_count == 0 {
            return Err(Error::FetchError("Invalid chunk count".to_string()));
        }
        if info.total_size == 0 {
            return Err(Error::FetchError("Invalid total size".to_string()));
        }
        if info.total_size > self.limits.max_upload_size {
            return Err(Error::FetchError("Upload size exceeds max".to_string()));
        }
        if info.chunk_count > self.limits.max_chunk_count {
            return Err(Error::FetchError("Chunk count exceeds max".to_string()));
        }
        if info.chunk_index >= info.chunk_count as usize {
            return Err(Error::FetchError("Chunk index out of range".to_string()));
        }
        if chunk_len == 0 {
            return Err(Error::FetchError("Empty chunk payload".to_string()));
        }
        if chunk_len > self.limits.max_chunk_size {
            return Err(Error::FetchError("Chunk size exceeds max".to_string()));
        }
        if chunk_len > info.total_size {
            return Err(Error::FetchError(
                "Chunk size exceeds declared total size".to_string(),
            ));
        }
        if !self.uploads.contains_key(&info.upload_id)
            && self.uploads.len() >= self.limits.max_inflight_uploads
        {
            return Err(Error::FetchError("Too many in-flight uploads".to_string()));
        }
        Ok(())
    }
}

impl Default for ChunkedUploadAssembler {
    fn default() -> Self {
        Self::with_limits(UploadLimits::default())
    }
}

#[derive(Debug)]
struct PendingUpload {
    chunk_count: u32,
    total_size: usize,
    filename: Option<String>,
    content_type: Option<String>,
    received: usize,
    received_bytes: usize,
    chunks: Vec<Option<Vec<u8>>>,
    seen: Vec<bool>,
    last_seen: Instant,
}

impl PendingUpload {
    fn new(info: &ChunkInfo, now: Instant) -> Self {
        Self {
            chunk_count: info.chunk_count,
            total_size: info.total_size,
            filename: info.filename.clone(),
            content_type: info.content_type.clone(),
            received: 0,
            received_bytes: 0,
            chunks: vec![None; info.chunk_count as usize],
            seen: vec![false; info.chunk_count as usize],
            last_seen: now,
        }
    }

    fn matches(&self, info: &ChunkInfo) -> bool {
        self.chunk_count == info.chunk_count
            && self.total_size == info.total_size
            && self.filename == info.filename
            && self.content_type == info.content_type
    }
}
