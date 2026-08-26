use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const DEFAULT_BASE_SUBJECT: &str = "chunked-upload";
pub const DEFAULT_BEGIN_QUEUE: &str = "chunked-upload-receivers";
pub const DEFAULT_CHUNK_SIZE: usize = 2_000_000;
pub const MAX_CHUNK_SIZE: usize = 8_000_000;
pub(crate) const DEFAULT_MAX_UPLOAD_ID_LENGTH: usize = 256;
pub(crate) const DEFAULT_MAX_FILENAME_LENGTH: usize = 1_024;
pub(crate) const DEFAULT_MAX_CONTENT_TYPE_LENGTH: usize = 255;
pub(crate) const DEFAULT_MAX_BEGIN_REQUEST_SIZE: usize = 8 * 1_024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadErrorCode {
    InvalidRequest,
    InvalidSubject,
    InvalidUploadId,
    InvalidDigest,
    InvalidMetadata,
    UploadTooLarge,
    CapacityExceeded,
    UploadConflict,
    SessionNotFound,
    OutOfOrder,
    ChunkLengthMismatch,
    ChunkConflict,
    IncompleteUpload,
    SizeMismatch,
    DigestMismatch,
    StoreUnavailable,
    Internal,
}

impl UploadErrorCode {
    pub fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::CapacityExceeded
                | Self::SessionNotFound
                | Self::StoreUnavailable
                | Self::Internal
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BeginRequest {
    pub upload_id: String,
    pub size: u64,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum BeginResponse {
    Ready {
        receiver_id: Uuid,
        session_id: Uuid,
        chunk_size: u64,
        session_ttl_ms: u64,
    },
    AlreadyStored {
        upload_id: String,
    },
    Error {
        code: UploadErrorCode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ChunkResponse {
    Accepted {
        index: u64,
    },
    Error {
        code: UploadErrorCode,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_index: Option<u64>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum CommitResponse {
    Stored { upload_id: String },
    Error { code: UploadErrorCode },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReceiverOperation {
    Chunk { session_id: Uuid, index: u64 },
    Commit { session_id: Uuid },
}

pub(crate) fn begin_subject(base: &str) -> String {
    format!("{base}.begin")
}

pub(crate) fn receiver_subscription_subject(base: &str, receiver_id: Uuid) -> String {
    format!("{base}.receiver.{receiver_id}.>")
}

pub(crate) fn chunk_subject(base: &str, receiver_id: Uuid, session_id: Uuid, index: u64) -> String {
    format!("{base}.receiver.{receiver_id}.session.{session_id}.chunk.{index}")
}

pub(crate) fn commit_subject(base: &str, receiver_id: Uuid, session_id: Uuid) -> String {
    format!("{base}.receiver.{receiver_id}.session.{session_id}.commit")
}

pub(crate) fn parse_receiver_subject(
    base: &str,
    receiver_id: Uuid,
    subject: &str,
) -> Result<ReceiverOperation, UploadErrorCode> {
    let prefix = format!("{base}.receiver.{receiver_id}.session.");
    let remainder = subject
        .strip_prefix(&prefix)
        .ok_or(UploadErrorCode::InvalidSubject)?;
    let tokens: Vec<&str> = remainder.split('.').collect();

    match tokens.as_slice() {
        [session_id, "commit"] => Ok(ReceiverOperation::Commit {
            session_id: parse_canonical_uuid(session_id)?,
        }),
        [session_id, "chunk", index] => Ok(ReceiverOperation::Chunk {
            session_id: parse_canonical_uuid(session_id)?,
            index: parse_canonical_index(index)?,
        }),
        _ => Err(UploadErrorCode::InvalidSubject),
    }
}

pub(crate) fn is_valid_base_subject(subject: &str) -> bool {
    !subject.is_empty()
        && subject.len() <= 512
        && subject.split('.').all(|token| {
            !token.is_empty()
                && !token.contains(['*', '>'])
                && !token.chars().any(char::is_whitespace)
        })
}

pub(crate) fn is_valid_queue_name(queue: &str) -> bool {
    !queue.is_empty()
        && queue.len() <= 512
        && !queue.contains(['*', '>'])
        && !queue.chars().any(char::is_whitespace)
}

fn parse_canonical_uuid(value: &str) -> Result<Uuid, UploadErrorCode> {
    let id = Uuid::parse_str(value).map_err(|_| UploadErrorCode::InvalidSubject)?;
    if id.to_string() != value {
        return Err(UploadErrorCode::InvalidSubject);
    }
    Ok(id)
}

fn parse_canonical_index(value: &str) -> Result<u64, UploadErrorCode> {
    let index = value
        .parse::<u64>()
        .map_err(|_| UploadErrorCode::InvalidSubject)?;
    if index.to_string() != value {
        return Err(UploadErrorCode::InvalidSubject);
    }
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chunk_and_commit_subjects() {
        let receiver_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let chunk = chunk_subject("files.upload", receiver_id, session_id, 12);
        let commit = commit_subject("files.upload", receiver_id, session_id);

        assert_eq!(
            parse_receiver_subject("files.upload", receiver_id, &chunk),
            Ok(ReceiverOperation::Chunk {
                session_id,
                index: 12
            })
        );
        assert_eq!(
            parse_receiver_subject("files.upload", receiver_id, &commit),
            Ok(ReceiverOperation::Commit { session_id })
        );
    }

    #[test]
    fn rejects_noncanonical_and_foreign_subjects() {
        let receiver_id = Uuid::new_v4();
        let other_receiver = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let uppercase_session = session_id.to_string().to_uppercase();
        let cases = [
            chunk_subject("files", other_receiver, session_id, 0),
            format!("files.receiver.{receiver_id}.session.{session_id}.chunk.01"),
            format!("files.receiver.{receiver_id}.session.{uppercase_session}.commit"),
            format!("files.receiver.{receiver_id}.session.{session_id}.chunk"),
            format!("files.receiver.{receiver_id}.session.{session_id}.commit.extra"),
        ];

        for subject in cases {
            assert_eq!(
                parse_receiver_subject("files", receiver_id, &subject),
                Err(UploadErrorCode::InvalidSubject)
            );
        }
    }

    #[test]
    fn validates_subject_configuration() {
        assert!(is_valid_base_subject("files.upload"));
        assert!(!is_valid_base_subject(""));
        assert!(!is_valid_base_subject("files.*"));
        assert!(!is_valid_base_subject("files..upload"));
        assert!(!is_valid_base_subject("files upload"));
        assert!(is_valid_queue_name("upload-workers"));
        assert!(!is_valid_queue_name("upload workers"));
    }

    #[test]
    fn wire_json_is_stable_and_rejects_unknown_fields() {
        let receiver_id =
            Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("test UUID is valid");
        let session_id =
            Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("test UUID is valid");
        let ready = BeginResponse::Ready {
            receiver_id,
            session_id,
            chunk_size: 2_000_000,
            session_ttl_ms: 600_000,
        };
        let error = ChunkResponse::Error {
            code: UploadErrorCode::OutOfOrder,
            expected_index: Some(3),
        };

        assert_eq!(
            serde_json::to_string(&ready).expect("response serializes"),
            r#"{"status":"ready","receiver_id":"11111111-1111-4111-8111-111111111111","session_id":"22222222-2222-4222-8222-222222222222","chunk_size":2000000,"session_ttl_ms":600000}"#
        );
        assert_eq!(
            serde_json::to_string(&error).expect("response serializes"),
            r#"{"status":"error","code":"out_of_order","expected_index":3}"#
        );
        assert!(
            serde_json::from_str::<BeginRequest>(
                r#"{"upload_id":"a","size":1,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","extra":true}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<CommitResponse>(
                r#"{"status":"stored","upload_id":"a","extra":true}"#
            )
            .is_err()
        );
    }

    #[test]
    fn error_code_retryability_is_explicit() {
        assert!(UploadErrorCode::SessionNotFound.is_retryable());
        assert!(UploadErrorCode::StoreUnavailable.is_retryable());
        assert!(!UploadErrorCode::ChunkConflict.is_retryable());
        assert!(!UploadErrorCode::UploadConflict.is_retryable());
    }

    #[test]
    fn all_error_code_wire_values_are_stable() {
        let cases = [
            (UploadErrorCode::InvalidRequest, r#""invalid_request""#),
            (UploadErrorCode::InvalidSubject, r#""invalid_subject""#),
            (UploadErrorCode::InvalidUploadId, r#""invalid_upload_id""#),
            (UploadErrorCode::InvalidDigest, r#""invalid_digest""#),
            (UploadErrorCode::InvalidMetadata, r#""invalid_metadata""#),
            (UploadErrorCode::UploadTooLarge, r#""upload_too_large""#),
            (UploadErrorCode::CapacityExceeded, r#""capacity_exceeded""#),
            (UploadErrorCode::UploadConflict, r#""upload_conflict""#),
            (UploadErrorCode::SessionNotFound, r#""session_not_found""#),
            (UploadErrorCode::OutOfOrder, r#""out_of_order""#),
            (
                UploadErrorCode::ChunkLengthMismatch,
                r#""chunk_length_mismatch""#,
            ),
            (UploadErrorCode::ChunkConflict, r#""chunk_conflict""#),
            (UploadErrorCode::IncompleteUpload, r#""incomplete_upload""#),
            (UploadErrorCode::SizeMismatch, r#""size_mismatch""#),
            (UploadErrorCode::DigestMismatch, r#""digest_mismatch""#),
            (UploadErrorCode::StoreUnavailable, r#""store_unavailable""#),
            (UploadErrorCode::Internal, r#""internal""#),
        ];

        for (code, expected) in cases {
            assert_eq!(
                serde_json::to_string(&code).expect("error code serializes"),
                expected
            );
        }
    }
}
