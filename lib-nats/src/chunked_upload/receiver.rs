use std::collections::HashMap;

use async_nats::Message;

use crate::chunked_upload::protocol::{
    ChunkInfo, ChunkedPayload, assemble_chunks, parse_chunk_info,
};
use crate::error::{Error, Result};

#[derive(Debug, Default)]
pub struct ChunkedUploadAssembler {
    uploads: HashMap<String, PendingUpload>,
}

impl ChunkedUploadAssembler {
    pub fn push(&mut self, message: &Message) -> Result<Option<ChunkedPayload>> {
        let info = match message.headers.as_ref() {
            Some(headers) => match parse_chunk_info(headers)? {
                Some(info) => info,
                None => return Ok(None),
            },
            None => return Ok(None),
        };

        let entry = self
            .uploads
            .entry(info.upload_id.clone())
            .or_insert_with(|| PendingUpload {
                chunk_count: info.chunk_count,
                total_size: info.total_size,
                filename: info.filename.clone(),
                content_type: info.content_type.clone(),
                received: 0,
                chunks: vec![None; info.chunk_count as usize],
            });

        if !entry.matches(&info) {
            return Err(Error::FetchError("Chunk metadata mismatch".to_string()));
        }

        if info.chunk_index >= entry.chunks.len() {
            return Err(Error::FetchError("Chunk index out of range".to_string()));
        }

        if entry.chunks[info.chunk_index].is_none() {
            entry.chunks[info.chunk_index] = Some(message.payload.to_vec());
            entry.received += 1;
        }

        if entry.received == entry.chunk_count as usize {
            let assembled = assemble_chunks(&entry.chunks, entry.chunk_count, entry.total_size)?;
            let payload = ChunkedPayload {
                upload_id: info.upload_id.clone(),
                data: assembled,
                filename: entry.filename.clone(),
                content_type: entry.content_type.clone(),
            };
            self.uploads.remove(&info.upload_id);
            return Ok(Some(payload));
        }

        Ok(None)
    }

    pub fn remove(&mut self, upload_id: &str) {
        self.uploads.remove(upload_id);
    }
}

#[derive(Debug)]
struct PendingUpload {
    chunk_count: u32,
    total_size: usize,
    filename: Option<String>,
    content_type: Option<String>,
    received: usize,
    chunks: Vec<Option<Vec<u8>>>,
}

impl PendingUpload {
    fn matches(&self, info: &ChunkInfo) -> bool {
        self.chunk_count == info.chunk_count && self.total_size == info.total_size
    }
}
