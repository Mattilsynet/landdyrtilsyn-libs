use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use uuid::Uuid;

use super::client::{is_valid_digest, sha256_hex};
use super::protocol::{
    BeginRequest, BeginResponse, ChunkResponse, CommitResponse, DEFAULT_BASE_SUBJECT,
    DEFAULT_BEGIN_QUEUE, DEFAULT_CHUNK_SIZE, DEFAULT_MAX_BEGIN_REQUEST_SIZE,
    DEFAULT_MAX_CONTENT_TYPE_LENGTH, DEFAULT_MAX_FILENAME_LENGTH, DEFAULT_MAX_UPLOAD_ID_LENGTH,
    MAX_CHUNK_SIZE, ReceiverOperation, UploadErrorCode, begin_subject, is_valid_base_subject,
    is_valid_queue_name, parse_receiver_subject, receiver_subscription_subject,
};

const DEFAULT_MAX_UPLOAD_SIZE: u64 = 100 * 1024 * 1024;
const DEFAULT_MAX_SESSIONS: usize = 100;
const DEFAULT_MAX_RESERVED_BYTES: u64 = 500 * 1024 * 1024;
const DEFAULT_SESSION_TTL: Duration = Duration::from_secs(10 * 60);
const DEFAULT_TOMBSTONE_TTL: Duration = Duration::from_secs(10 * 60);
const DEFAULT_MAX_TOMBSTONES: usize = 1_000;
const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(30);
const DEFAULT_SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, thiserror::Error)]
pub enum UploadStoreError {
    #[error("stored upload conflicts with the requested upload")]
    Conflict,
    #[error("upload store is unavailable")]
    Unavailable(#[source] BoxError),
}

impl UploadStoreError {
    pub fn unavailable(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Unavailable(Box::new(error))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct UploadDescriptor {
    pub upload_id: String,
    pub size: u64,
    pub sha256: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

impl fmt::Debug for UploadDescriptor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UploadDescriptor")
            .field("upload_id", &self.upload_id)
            .field("size", &self.size)
            .field("sha256", &self.sha256)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredUpload {
    pub size: u64,
    pub sha256: String,
}

#[derive(Clone)]
pub struct CompletedUpload {
    pub descriptor: UploadDescriptor,
    pub bytes: Bytes,
}

impl fmt::Debug for CompletedUpload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompletedUpload")
            .field("descriptor", &self.descriptor)
            .field("byte_length", &self.bytes.len())
            .finish()
    }
}

#[async_trait]
pub trait UploadStore: Send + Sync {
    async fn inspect(&self, upload_id: &str) -> Result<Option<StoredUpload>, UploadStoreError>;

    async fn store(&self, upload: CompletedUpload) -> Result<(), UploadStoreError>;
}

#[derive(Debug, Clone)]
pub struct ChunkedUploadServerConfig {
    pub base_subject: String,
    pub begin_queue: String,
    pub chunk_size: usize,
    pub session_ttl: Duration,
    pub max_upload_size: u64,
    pub max_sessions: usize,
    pub max_reserved_bytes: u64,
    pub max_upload_id_length: usize,
    pub max_filename_length: usize,
    pub max_content_type_length: usize,
    pub max_begin_request_size: usize,
    pub tombstone_ttl: Duration,
    pub max_tombstones: usize,
    pub cleanup_interval: Duration,
    pub shutdown_grace: Duration,
}

impl Default for ChunkedUploadServerConfig {
    fn default() -> Self {
        Self {
            base_subject: DEFAULT_BASE_SUBJECT.to_string(),
            begin_queue: DEFAULT_BEGIN_QUEUE.to_string(),
            chunk_size: DEFAULT_CHUNK_SIZE,
            session_ttl: DEFAULT_SESSION_TTL,
            max_upload_size: DEFAULT_MAX_UPLOAD_SIZE,
            max_sessions: DEFAULT_MAX_SESSIONS,
            max_reserved_bytes: DEFAULT_MAX_RESERVED_BYTES,
            max_upload_id_length: DEFAULT_MAX_UPLOAD_ID_LENGTH,
            max_filename_length: DEFAULT_MAX_FILENAME_LENGTH,
            max_content_type_length: DEFAULT_MAX_CONTENT_TYPE_LENGTH,
            max_begin_request_size: DEFAULT_MAX_BEGIN_REQUEST_SIZE,
            tombstone_ttl: DEFAULT_TOMBSTONE_TTL,
            max_tombstones: DEFAULT_MAX_TOMBSTONES,
            cleanup_interval: DEFAULT_CLEANUP_INTERVAL,
            shutdown_grace: DEFAULT_SHUTDOWN_GRACE,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChunkedUploadServerError {
    #[error("invalid chunked upload server configuration: {0}")]
    Configuration(&'static str),
    #[error("failed to subscribe for chunked uploads")]
    Subscribe(#[source] async_nats::SubscribeError),
    #[error("failed to flush chunked upload subscriptions")]
    Flush(#[source] async_nats::client::FlushError),
    #[error("failed to unsubscribe a chunked upload subscription")]
    Unsubscribe(#[source] async_nats::UnsubscribeError),
    #[error("a chunked upload subscription closed unexpectedly")]
    SubscriptionClosed,
}

pub struct ChunkedUploadServer {
    client: async_nats::Client,
    config: ChunkedUploadServerConfig,
    store: Arc<dyn UploadStore>,
    receiver_id: Uuid,
    sessions: SessionManager,
}

impl fmt::Debug for ChunkedUploadServer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChunkedUploadServer")
            .field("config", &self.config)
            .field("receiver_id", &self.receiver_id)
            .finish_non_exhaustive()
    }
}

impl ChunkedUploadServer {
    pub fn new(
        client: async_nats::Client,
        config: ChunkedUploadServerConfig,
        store: Arc<dyn UploadStore>,
    ) -> Result<Self, ChunkedUploadServerError> {
        validate_config(&config)?;
        let receiver_id = Uuid::new_v4();
        let sessions = SessionManager::new(&config);
        Ok(Self {
            client,
            config,
            store,
            receiver_id,
            sessions,
        })
    }

    pub fn receiver_id(&self) -> Uuid {
        self.receiver_id
    }

    pub async fn run<F>(mut self, shutdown: F) -> Result<(), ChunkedUploadServerError>
    where
        F: Future<Output = ()> + Send,
    {
        let receiver_subject =
            receiver_subscription_subject(&self.config.base_subject, self.receiver_id);
        let mut receiver_subscription = self
            .client
            .subscribe(receiver_subject)
            .await
            .map_err(ChunkedUploadServerError::Subscribe)?;
        self.client
            .flush()
            .await
            .map_err(ChunkedUploadServerError::Flush)?;

        let mut begin_subscription = self
            .client
            .queue_subscribe(
                begin_subject(&self.config.base_subject),
                self.config.begin_queue.clone(),
            )
            .await
            .map_err(ChunkedUploadServerError::Subscribe)?;
        self.client
            .flush()
            .await
            .map_err(ChunkedUploadServerError::Flush)?;

        let mut cleanup = tokio::time::interval(self.config.cleanup_interval);
        cleanup.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                biased;
                () = &mut shutdown => break,
                message = begin_subscription.next() => {
                    let message = message.ok_or(ChunkedUploadServerError::SubscriptionClosed)?;
                    self.handle_begin(message).await;
                }
                message = receiver_subscription.next() => {
                    let message = message.ok_or(ChunkedUploadServerError::SubscriptionClosed)?;
                    self.handle_receiver(message).await;
                }
                _ = cleanup.tick() => self.sessions.prune(Instant::now()),
            }
        }

        begin_subscription
            .unsubscribe()
            .await
            .map_err(ChunkedUploadServerError::Unsubscribe)?;
        self.client
            .flush()
            .await
            .map_err(ChunkedUploadServerError::Flush)?;

        let grace_deadline = tokio::time::Instant::now()
            .checked_add(self.config.shutdown_grace)
            .ok_or(ChunkedUploadServerError::Configuration(
                "shutdown grace is too large",
            ))?;
        loop {
            tokio::select! {
                biased;
                _ = tokio::time::sleep_until(grace_deadline) => break,
                message = receiver_subscription.next() => {
                    match message {
                        Some(message) => self.handle_receiver(message).await,
                        None => break,
                    }
                }
                _ = cleanup.tick() => self.sessions.prune(Instant::now()),
            }
        }

        receiver_subscription
            .unsubscribe()
            .await
            .map_err(ChunkedUploadServerError::Unsubscribe)?;
        self.client
            .flush()
            .await
            .map_err(ChunkedUploadServerError::Flush)?;
        Ok(())
    }

    async fn handle_begin(&mut self, message: async_nats::Message) {
        let Some(reply) = message.reply else {
            return;
        };
        self.sessions.prune(Instant::now());

        let request = if message.payload.len() <= self.config.max_begin_request_size {
            serde_json::from_slice::<BeginRequest>(&message.payload).ok()
        } else {
            None
        };
        let Some(request) = request else {
            self.respond(
                reply,
                &BeginResponse::Error {
                    code: UploadErrorCode::InvalidRequest,
                },
            )
            .await;
            return;
        };
        let descriptor = UploadDescriptor {
            upload_id: request.upload_id,
            size: request.size,
            sha256: request.sha256,
            filename: request.filename,
            content_type: request.content_type,
        };
        if let Err(code) = validate_descriptor(&descriptor, &self.config) {
            self.respond(reply, &BeginResponse::Error { code }).await;
            return;
        }

        match self.store.inspect(&descriptor.upload_id).await {
            Ok(Some(stored)) if stored.matches(&descriptor) => {
                self.sessions.remove_upload(&descriptor.upload_id);
                self.respond(
                    reply,
                    &BeginResponse::AlreadyStored {
                        upload_id: descriptor.upload_id,
                    },
                )
                .await;
            }
            Ok(Some(_)) => {
                self.sessions.remove_upload(&descriptor.upload_id);
                self.respond(
                    reply,
                    &BeginResponse::Error {
                        code: UploadErrorCode::UploadConflict,
                    },
                )
                .await;
            }
            Err(UploadStoreError::Conflict) => {
                self.sessions.remove_upload(&descriptor.upload_id);
                self.respond(
                    reply,
                    &BeginResponse::Error {
                        code: UploadErrorCode::UploadConflict,
                    },
                )
                .await;
            }
            Err(UploadStoreError::Unavailable(_)) => {
                tracing::warn!(receiver_id = %self.receiver_id, "upload store inspect failed");
                self.respond(
                    reply,
                    &BeginResponse::Error {
                        code: UploadErrorCode::StoreUnavailable,
                    },
                )
                .await;
            }
            Ok(None) => {
                let now = Instant::now();
                match self.sessions.begin(descriptor, now) {
                    Ok(session_id) => {
                        let session_ttl_ms =
                            u64::try_from(self.config.session_ttl.as_millis()).unwrap_or(u64::MAX);
                        self.respond(
                            reply,
                            &BeginResponse::Ready {
                                receiver_id: self.receiver_id,
                                session_id,
                                chunk_size: self.config.chunk_size as u64,
                                session_ttl_ms,
                            },
                        )
                        .await;
                    }
                    Err(code) => self.respond(reply, &BeginResponse::Error { code }).await,
                }
            }
        }
    }

    async fn handle_receiver(&mut self, message: async_nats::Message) {
        let Some(reply) = message.reply else {
            return;
        };
        let now = Instant::now();
        self.sessions.prune(now);
        let operation = parse_receiver_subject(
            &self.config.base_subject,
            self.receiver_id,
            message.subject.as_str(),
        );

        match operation {
            Ok(ReceiverOperation::Chunk { session_id, index }) => {
                let response = match self
                    .sessions
                    .chunk(session_id, index, &message.payload, now)
                {
                    Ok(()) => ChunkResponse::Accepted { index },
                    Err(failure) => ChunkResponse::Error {
                        code: failure.code,
                        expected_index: failure.expected_index,
                    },
                };
                self.respond(reply, &response).await;
            }
            Ok(ReceiverOperation::Commit { session_id }) => {
                if !message.payload.is_empty() {
                    self.respond(
                        reply,
                        &CommitResponse::Error {
                            code: UploadErrorCode::InvalidRequest,
                        },
                    )
                    .await;
                    return;
                }
                self.handle_commit(reply, session_id, now).await;
            }
            Err(code) => {
                self.respond(
                    reply,
                    &ChunkResponse::Error {
                        code,
                        expected_index: None,
                    },
                )
                .await;
            }
        }
    }

    async fn handle_commit(&mut self, reply: async_nats::Subject, session_id: Uuid, now: Instant) {
        let upload = match self.sessions.prepare_commit(session_id, now) {
            CommitPreparation::Stored(upload_id) => {
                self.respond(reply, &CommitResponse::Stored { upload_id })
                    .await;
                return;
            }
            CommitPreparation::Upload(upload) => upload,
            CommitPreparation::Error(code) => {
                self.respond(reply, &CommitResponse::Error { code }).await;
                return;
            }
        };
        let upload_id = upload.descriptor.upload_id.clone();

        match self.store.store(upload).await {
            Ok(()) => {
                self.sessions
                    .mark_stored(session_id, upload_id.clone(), Instant::now());
                self.respond(reply, &CommitResponse::Stored { upload_id })
                    .await;
            }
            Err(UploadStoreError::Conflict) => {
                self.sessions.remove_session(session_id);
                self.respond(
                    reply,
                    &CommitResponse::Error {
                        code: UploadErrorCode::UploadConflict,
                    },
                )
                .await;
            }
            Err(UploadStoreError::Unavailable(_)) => {
                tracing::warn!(receiver_id = %self.receiver_id, "upload store write failed");
                self.sessions.touch_session(session_id, Instant::now());
                self.respond(
                    reply,
                    &CommitResponse::Error {
                        code: UploadErrorCode::StoreUnavailable,
                    },
                )
                .await;
            }
        }
    }

    async fn respond<T: serde::Serialize>(&self, reply: async_nats::Subject, response: &T) {
        let payload = match serde_json::to_vec(response) {
            Ok(payload) => Bytes::from(payload),
            Err(_) => {
                tracing::error!(receiver_id = %self.receiver_id, "failed to encode upload response");
                return;
            }
        };
        if self.client.publish(reply, payload).await.is_err() {
            tracing::warn!(receiver_id = %self.receiver_id, "failed to publish upload response");
        }
    }
}

impl StoredUpload {
    fn matches(&self, descriptor: &UploadDescriptor) -> bool {
        self.size == descriptor.size && self.sha256 == descriptor.sha256
    }
}

fn validate_config(config: &ChunkedUploadServerConfig) -> Result<(), ChunkedUploadServerError> {
    if !is_valid_base_subject(&config.base_subject) {
        return Err(ChunkedUploadServerError::Configuration(
            "invalid base subject",
        ));
    }
    if !is_valid_queue_name(&config.begin_queue) {
        return Err(ChunkedUploadServerError::Configuration(
            "invalid begin queue",
        ));
    }
    if config.chunk_size == 0 || config.chunk_size > MAX_CHUNK_SIZE {
        return Err(ChunkedUploadServerError::Configuration(
            "chunk size must be between 1 and 8,000,000 bytes",
        ));
    }
    if config.session_ttl.is_zero()
        || config.tombstone_ttl.is_zero()
        || config.cleanup_interval.is_zero()
    {
        return Err(ChunkedUploadServerError::Configuration(
            "TTL and cleanup durations must be greater than zero",
        ));
    }
    if config.session_ttl.as_millis() == 0 || u64::try_from(config.session_ttl.as_millis()).is_err()
    {
        return Err(ChunkedUploadServerError::Configuration(
            "session TTL must be representable as positive milliseconds",
        ));
    }
    if config.max_sessions == 0
        || config.max_tombstones == 0
        || config.max_upload_id_length == 0
        || config.max_begin_request_size == 0
    {
        return Err(ChunkedUploadServerError::Configuration(
            "capacity and metadata limits must be greater than zero",
        ));
    }
    if usize::try_from(config.max_upload_size).is_err()
        || usize::try_from(config.max_reserved_bytes).is_err()
    {
        return Err(ChunkedUploadServerError::Configuration(
            "byte limits exceed the platform address space",
        ));
    }
    Ok(())
}

fn validate_descriptor(
    descriptor: &UploadDescriptor,
    config: &ChunkedUploadServerConfig,
) -> Result<(), UploadErrorCode> {
    if descriptor.upload_id.is_empty()
        || descriptor.upload_id.len() > config.max_upload_id_length
        || descriptor.upload_id.chars().any(char::is_control)
    {
        return Err(UploadErrorCode::InvalidUploadId);
    }
    if descriptor.size > config.max_upload_size || usize::try_from(descriptor.size).is_err() {
        return Err(UploadErrorCode::UploadTooLarge);
    }
    if !is_valid_digest(&descriptor.sha256) {
        return Err(UploadErrorCode::InvalidDigest);
    }
    if descriptor
        .filename
        .as_ref()
        .is_some_and(|value| value.len() > config.max_filename_length)
        || descriptor
            .content_type
            .as_ref()
            .is_some_and(|value| value.len() > config.max_content_type_length)
    {
        return Err(UploadErrorCode::InvalidMetadata);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SessionFailure {
    code: UploadErrorCode,
    expected_index: Option<u64>,
}

impl SessionFailure {
    fn new(code: UploadErrorCode) -> Self {
        Self {
            code,
            expected_index: None,
        }
    }

    fn out_of_order(expected_index: u64) -> Self {
        Self {
            code: UploadErrorCode::OutOfOrder,
            expected_index: Some(expected_index),
        }
    }
}

struct SessionManager {
    sessions: HashMap<Uuid, Session>,
    tombstones: HashMap<Uuid, Tombstone>,
    reserved_bytes: u64,
    limits: SessionLimits,
}

#[derive(Clone, Copy)]
struct SessionLimits {
    chunk_size: usize,
    session_ttl: Duration,
    max_sessions: usize,
    max_reserved_bytes: u64,
    tombstone_ttl: Duration,
    max_tombstones: usize,
}

impl SessionManager {
    fn new(config: &ChunkedUploadServerConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            tombstones: HashMap::new(),
            reserved_bytes: 0,
            limits: SessionLimits {
                chunk_size: config.chunk_size,
                session_ttl: config.session_ttl,
                max_sessions: config.max_sessions,
                max_reserved_bytes: config.max_reserved_bytes,
                tombstone_ttl: config.tombstone_ttl,
                max_tombstones: config.max_tombstones,
            },
        }
    }

    fn begin(
        &mut self,
        descriptor: UploadDescriptor,
        now: Instant,
    ) -> Result<Uuid, UploadErrorCode> {
        if let Some((session_id, session)) = self
            .sessions
            .iter_mut()
            .find(|(_, session)| session.descriptor.upload_id == descriptor.upload_id)
        {
            if session.descriptor != descriptor {
                return Err(UploadErrorCode::UploadConflict);
            }
            session.last_activity = now;
            return Ok(*session_id);
        }
        if self.sessions.len() >= self.limits.max_sessions {
            return Err(UploadErrorCode::CapacityExceeded);
        }
        let reserved = self
            .reserved_bytes
            .checked_add(descriptor.size)
            .filter(|reserved| *reserved <= self.limits.max_reserved_bytes)
            .ok_or(UploadErrorCode::CapacityExceeded)?;
        let session_id = Uuid::new_v4();
        self.sessions.insert(
            session_id,
            Session::new(descriptor, self.limits.chunk_size, now),
        );
        self.reserved_bytes = reserved;
        Ok(session_id)
    }

    fn chunk(
        &mut self,
        session_id: Uuid,
        index: u64,
        payload: &[u8],
        now: Instant,
    ) -> Result<(), SessionFailure> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| SessionFailure::new(UploadErrorCode::SessionNotFound))?;
        session.accept_chunk(index, payload, now)
    }

    fn prepare_commit(&mut self, session_id: Uuid, now: Instant) -> CommitPreparation {
        if let Some(tombstone) = self.tombstones.get(&session_id) {
            return CommitPreparation::Stored(tombstone.upload_id.clone());
        }
        let validation = match self.sessions.get_mut(&session_id) {
            Some(session) => session.prepare_commit(now),
            None => return CommitPreparation::Error(UploadErrorCode::SessionNotFound),
        };
        if matches!(
            validation,
            CommitPreparation::Error(
                UploadErrorCode::SizeMismatch | UploadErrorCode::DigestMismatch
            )
        ) {
            self.remove_session(session_id);
        }
        validation
    }

    fn mark_stored(&mut self, session_id: Uuid, upload_id: String, now: Instant) {
        self.remove_session(session_id);
        self.prune_tombstones(now);
        if self.tombstones.len() >= self.limits.max_tombstones
            && let Some(oldest) = self
                .tombstones
                .iter()
                .min_by_key(|(_, tombstone)| tombstone.created_at)
                .map(|(session_id, _)| *session_id)
        {
            self.tombstones.remove(&oldest);
        }
        self.tombstones.insert(
            session_id,
            Tombstone {
                upload_id,
                created_at: now,
            },
        );
    }

    fn remove_session(&mut self, session_id: Uuid) {
        if let Some(session) = self.sessions.remove(&session_id) {
            self.reserved_bytes = self.reserved_bytes.saturating_sub(session.descriptor.size);
        }
    }

    fn remove_upload(&mut self, upload_id: &str) {
        let session_id = self
            .sessions
            .iter()
            .find(|(_, session)| session.descriptor.upload_id == upload_id)
            .map(|(session_id, _)| *session_id);
        if let Some(session_id) = session_id {
            self.remove_session(session_id);
        }
    }

    fn touch_session(&mut self, session_id: Uuid, now: Instant) {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.last_activity = now;
        }
    }

    fn prune(&mut self, now: Instant) {
        let expired: Vec<Uuid> = self
            .sessions
            .iter()
            .filter(|(_, session)| elapsed(now, session.last_activity) >= self.limits.session_ttl)
            .map(|(session_id, _)| *session_id)
            .collect();
        for session_id in expired {
            self.remove_session(session_id);
        }
        self.prune_tombstones(now);
    }

    fn prune_tombstones(&mut self, now: Instant) {
        self.tombstones
            .retain(|_, tombstone| elapsed(now, tombstone.created_at) < self.limits.tombstone_ttl);
    }
}

struct Session {
    descriptor: UploadDescriptor,
    chunk_size: usize,
    expected_index: u64,
    bytes: SessionBytes,
    last_activity: Instant,
}

enum SessionBytes {
    Receiving(Vec<u8>),
    Complete(Bytes),
}

impl SessionBytes {
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Receiving(bytes) => bytes,
            Self::Complete(bytes) => bytes,
        }
    }
}

impl Session {
    fn new(descriptor: UploadDescriptor, chunk_size: usize, now: Instant) -> Self {
        Self {
            descriptor,
            chunk_size,
            expected_index: 0,
            bytes: SessionBytes::Receiving(Vec::new()),
            last_activity: now,
        }
    }

    fn accept_chunk(
        &mut self,
        index: u64,
        payload: &[u8],
        now: Instant,
    ) -> Result<(), SessionFailure> {
        if index < self.expected_index {
            let Some((start, length)) = self.chunk_range(index) else {
                return Err(SessionFailure::new(UploadErrorCode::ChunkConflict));
            };
            let end = start + length;
            if self.bytes.as_slice().get(start..end) == Some(payload) {
                self.last_activity = now;
                return Ok(());
            }
            return Err(SessionFailure::new(UploadErrorCode::ChunkConflict));
        }
        if index > self.expected_index {
            return Err(SessionFailure::out_of_order(self.expected_index));
        }

        let Some((_, expected_length)) = self.chunk_range(index) else {
            return Err(SessionFailure::new(UploadErrorCode::ChunkLengthMismatch));
        };
        if payload.len() != expected_length {
            return Err(SessionFailure::new(UploadErrorCode::ChunkLengthMismatch));
        }
        let SessionBytes::Receiving(bytes) = &mut self.bytes else {
            return Err(SessionFailure::new(UploadErrorCode::ChunkConflict));
        };
        bytes
            .try_reserve_exact(payload.len())
            .map_err(|_| SessionFailure::new(UploadErrorCode::CapacityExceeded))?;
        bytes.extend_from_slice(payload);
        self.expected_index += 1;
        self.last_activity = now;
        Ok(())
    }

    fn prepare_commit(&mut self, now: Instant) -> CommitPreparation {
        self.last_activity = now;
        let expected_chunks = self.chunk_count();
        if self.expected_index != expected_chunks {
            return CommitPreparation::Error(UploadErrorCode::IncompleteUpload);
        }
        if self.bytes.as_slice().len() as u64 != self.descriptor.size {
            return CommitPreparation::Error(UploadErrorCode::SizeMismatch);
        }
        if sha256_hex(self.bytes.as_slice()) != self.descriptor.sha256 {
            return CommitPreparation::Error(UploadErrorCode::DigestMismatch);
        }
        if let SessionBytes::Receiving(bytes) = &mut self.bytes {
            self.bytes = SessionBytes::Complete(Bytes::from(std::mem::take(bytes)));
        }
        let SessionBytes::Complete(bytes) = &self.bytes else {
            return CommitPreparation::Error(UploadErrorCode::Internal);
        };
        CommitPreparation::Upload(CompletedUpload {
            descriptor: self.descriptor.clone(),
            bytes: bytes.clone(),
        })
    }

    fn chunk_count(&self) -> u64 {
        if self.descriptor.size == 0 {
            0
        } else {
            self.descriptor.size.div_ceil(self.chunk_size as u64)
        }
    }

    fn chunk_range(&self, index: u64) -> Option<(usize, usize)> {
        let chunk_count = self.chunk_count();
        if index >= chunk_count {
            return None;
        }
        let chunk_size = self.chunk_size as u64;
        let start = index.checked_mul(chunk_size)?;
        let length = if index + 1 < chunk_count {
            chunk_size
        } else {
            self.descriptor.size.checked_sub(start)?
        };
        Some((usize::try_from(start).ok()?, usize::try_from(length).ok()?))
    }
}

enum CommitPreparation {
    Stored(String),
    Upload(CompletedUpload),
    Error(UploadErrorCode),
}

struct Tombstone {
    upload_id: String,
    created_at: Instant,
}

fn elapsed(now: Instant, earlier: Instant) -> Duration {
    now.checked_duration_since(earlier).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(upload_id: &str, bytes: &[u8]) -> UploadDescriptor {
        UploadDescriptor {
            upload_id: upload_id.to_string(),
            size: bytes.len() as u64,
            sha256: sha256_hex(bytes),
            filename: Some("not-logged.txt".to_string()),
            content_type: Some("text/plain".to_string()),
        }
    }

    fn config(chunk_size: usize) -> ChunkedUploadServerConfig {
        ChunkedUploadServerConfig {
            chunk_size,
            max_upload_size: 100,
            max_reserved_bytes: 100,
            session_ttl: Duration::from_secs(10),
            tombstone_ttl: Duration::from_secs(5),
            cleanup_interval: Duration::from_secs(1),
            ..ChunkedUploadServerConfig::default()
        }
    }

    #[test]
    fn validates_descriptor_size_digest_and_metadata() {
        let config = config(4);
        let mut descriptor = descriptor("id", b"abc");
        assert!(validate_descriptor(&descriptor, &config).is_ok());

        descriptor.sha256 = "A".repeat(64);
        assert_eq!(
            validate_descriptor(&descriptor, &config),
            Err(UploadErrorCode::InvalidDigest)
        );
        descriptor.sha256 = "a".repeat(64);
        descriptor.size = 101;
        assert_eq!(
            validate_descriptor(&descriptor, &config),
            Err(UploadErrorCode::UploadTooLarge)
        );
        descriptor.size = 3;
        descriptor.filename = Some("x".repeat(config.max_filename_length + 1));
        assert_eq!(
            validate_descriptor(&descriptor, &config),
            Err(UploadErrorCode::InvalidMetadata)
        );
    }

    #[test]
    fn begin_is_idempotent_and_reserves_declared_size() {
        let now = Instant::now();
        let mut sessions = SessionManager::new(&config(4));
        let descriptor = descriptor("id", b"abcdef");
        let first = sessions
            .begin(descriptor.clone(), now)
            .expect("session begins");
        let duplicate = sessions
            .begin(descriptor, now)
            .expect("duplicate begin succeeds");

        assert_eq!(first, duplicate);
        assert_eq!(sessions.sessions.len(), 1);
        assert_eq!(sessions.reserved_bytes, 6);
    }

    #[test]
    fn begin_rejects_conflicts_and_capacity_overflow() {
        let now = Instant::now();
        let mut capacity_config = config(4);
        capacity_config.max_sessions = 1;
        capacity_config.max_reserved_bytes = 6;
        let mut sessions = SessionManager::new(&capacity_config);
        sessions
            .begin(descriptor("id", b"abcdef"), now)
            .expect("first session begins");

        assert_eq!(
            sessions.begin(descriptor("id", b"different"), now),
            Err(UploadErrorCode::UploadConflict)
        );
        assert_eq!(
            sessions.begin(descriptor("other", b"a"), now),
            Err(UploadErrorCode::CapacityExceeded)
        );
    }

    #[test]
    fn begin_enforces_reserved_byte_capacity() {
        let now = Instant::now();
        let mut capacity_config = config(4);
        capacity_config.max_sessions = 2;
        capacity_config.max_reserved_bytes = 6;
        let mut sessions = SessionManager::new(&capacity_config);
        sessions
            .begin(descriptor("first", b"abcdef"), now)
            .expect("first session begins");

        assert_eq!(
            sessions.begin(descriptor("second", b"a"), now),
            Err(UploadErrorCode::CapacityExceeded)
        );
        assert_eq!(sessions.sessions.len(), 1);
        assert_eq!(sessions.reserved_bytes, 6);
    }

    #[test]
    fn durable_upload_discovery_releases_stale_session_capacity() {
        let now = Instant::now();
        let mut sessions = SessionManager::new(&config(4));
        sessions
            .begin(descriptor("stored-elsewhere", b"abcdef"), now)
            .expect("session begins");

        sessions.remove_upload("stored-elsewhere");
        assert!(sessions.sessions.is_empty());
        assert_eq!(sessions.reserved_bytes, 0);
    }

    #[test]
    fn accepts_sequential_chunks_and_identical_duplicates() {
        let now = Instant::now();
        let mut sessions = SessionManager::new(&config(4));
        let session_id = sessions
            .begin(descriptor("id", b"abcdef"), now)
            .expect("session begins");

        assert_eq!(sessions.chunk(session_id, 0, b"abcd", now), Ok(()));
        assert_eq!(sessions.chunk(session_id, 0, b"abcd", now), Ok(()));
        assert_eq!(sessions.chunk(session_id, 1, b"ef", now), Ok(()));
        assert_eq!(sessions.sessions[&session_id].expected_index, 2);
    }

    #[test]
    fn rejects_conflicting_duplicate_and_out_of_order_chunk() {
        let now = Instant::now();
        let mut sessions = SessionManager::new(&config(4));
        let session_id = sessions
            .begin(descriptor("id", b"abcdef"), now)
            .expect("session begins");
        sessions
            .chunk(session_id, 0, b"abcd", now)
            .expect("first chunk succeeds");

        assert_eq!(
            sessions.chunk(session_id, 0, b"abce", now),
            Err(SessionFailure::new(UploadErrorCode::ChunkConflict))
        );
        assert_eq!(
            sessions.chunk(session_id, 2, b"", now),
            Err(SessionFailure::out_of_order(1))
        );
    }

    #[test]
    fn validates_nonfinal_and_final_chunk_lengths_exactly() {
        let now = Instant::now();
        let mut sessions = SessionManager::new(&config(4));
        let session_id = sessions
            .begin(descriptor("id", b"abcdef"), now)
            .expect("session begins");

        assert_eq!(
            sessions.chunk(session_id, 0, b"abc", now),
            Err(SessionFailure::new(UploadErrorCode::ChunkLengthMismatch))
        );
        sessions
            .chunk(session_id, 0, b"abcd", now)
            .expect("nonfinal chunk succeeds");
        assert_eq!(
            sessions.chunk(session_id, 1, b"e", now),
            Err(SessionFailure::new(UploadErrorCode::ChunkLengthMismatch))
        );
        sessions
            .chunk(session_id, 1, b"ef", now)
            .expect("final chunk succeeds");
    }

    #[test]
    fn exact_multiple_has_full_sized_final_chunk() {
        let now = Instant::now();
        let mut sessions = SessionManager::new(&config(4));
        let session_id = sessions
            .begin(descriptor("id", b"abcdefgh"), now)
            .expect("session begins");
        sessions
            .chunk(session_id, 0, b"abcd", now)
            .expect("first chunk succeeds");

        assert_eq!(
            sessions.chunk(session_id, 1, b"efg", now),
            Err(SessionFailure::new(UploadErrorCode::ChunkLengthMismatch))
        );
        assert_eq!(sessions.chunk(session_id, 1, b"efgh", now), Ok(()));
    }

    #[test]
    fn commit_requires_all_chunks_and_verifies_digest() {
        let now = Instant::now();
        let mut sessions = SessionManager::new(&config(4));
        let session_id = sessions
            .begin(descriptor("id", b"abcdef"), now)
            .expect("session begins");
        assert!(matches!(
            sessions.prepare_commit(session_id, now),
            CommitPreparation::Error(UploadErrorCode::IncompleteUpload)
        ));
        sessions
            .chunk(session_id, 0, b"abcd", now)
            .expect("first chunk succeeds");
        sessions
            .chunk(session_id, 1, b"ef", now)
            .expect("second chunk succeeds");
        sessions
            .sessions
            .get_mut(&session_id)
            .expect("session")
            .descriptor
            .sha256 = "0".repeat(64);

        assert!(matches!(
            sessions.prepare_commit(session_id, now),
            CommitPreparation::Error(UploadErrorCode::DigestMismatch)
        ));
        assert!(!sessions.sessions.contains_key(&session_id));
        assert_eq!(sessions.reserved_bytes, 0);
    }

    #[test]
    fn supports_empty_upload_commit() {
        let now = Instant::now();
        let mut sessions = SessionManager::new(&config(4));
        let session_id = sessions
            .begin(descriptor("empty", b""), now)
            .expect("empty session begins");

        match sessions.prepare_commit(session_id, now) {
            CommitPreparation::Upload(upload) => assert!(upload.bytes.is_empty()),
            _ => panic!("empty upload should be ready to store"),
        }
    }

    #[test]
    fn expires_sessions_and_releases_reserved_capacity() {
        let now = Instant::now();
        let mut sessions = SessionManager::new(&config(4));
        sessions
            .begin(descriptor("id", b"abcdef"), now)
            .expect("session begins");

        sessions.prune(now + Duration::from_secs(10));
        assert!(sessions.sessions.is_empty());
        assert_eq!(sessions.reserved_bytes, 0);
    }

    #[test]
    fn duplicate_commit_uses_bounded_expiring_tombstone() {
        let now = Instant::now();
        let mut tombstone_config = config(4);
        tombstone_config.max_tombstones = 1;
        let mut sessions = SessionManager::new(&tombstone_config);
        let first = sessions
            .begin(descriptor("first", b""), now)
            .expect("first session begins");
        sessions.mark_stored(first, "first".to_string(), now);
        assert!(matches!(
            sessions.prepare_commit(first, now),
            CommitPreparation::Stored(upload_id) if upload_id == "first"
        ));

        let second = sessions
            .begin(descriptor("second", b""), now)
            .expect("second session begins");
        sessions.mark_stored(second, "second".to_string(), now + Duration::from_secs(1));
        assert_eq!(sessions.tombstones.len(), 1);
        assert!(!sessions.tombstones.contains_key(&first));

        sessions.prune(now + Duration::from_secs(6));
        assert!(sessions.tombstones.is_empty());
    }
}
