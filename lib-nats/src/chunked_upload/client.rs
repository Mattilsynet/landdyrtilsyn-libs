use std::fmt;
use std::time::Duration;

use async_nats::client::RequestErrorKind;
use bytes::Bytes;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

use super::protocol::{
    BeginRequest, BeginResponse, ChunkResponse, CommitResponse, DEFAULT_BASE_SUBJECT,
    DEFAULT_MAX_CONTENT_TYPE_LENGTH, DEFAULT_MAX_FILENAME_LENGTH, DEFAULT_MAX_UPLOAD_ID_LENGTH,
    MAX_CHUNK_SIZE, UploadErrorCode, begin_subject, chunk_subject, commit_subject,
    is_valid_base_subject,
};

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_SESSION_ATTEMPTS: usize = 3;
const DEFAULT_CHUNK_ATTEMPTS: usize = 2;
const DEFAULT_COMMIT_ATTEMPTS: usize = 3;

#[derive(Debug, Clone)]
pub struct ChunkedUploadClientConfig {
    pub base_subject: String,
    pub request_timeout: Duration,
    pub max_session_attempts: usize,
    pub chunk_attempts: usize,
    pub commit_attempts: usize,
}

impl Default for ChunkedUploadClientConfig {
    fn default() -> Self {
        Self {
            base_subject: DEFAULT_BASE_SUBJECT.to_string(),
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            max_session_attempts: DEFAULT_SESSION_ATTEMPTS,
            chunk_attempts: DEFAULT_CHUNK_ATTEMPTS,
            commit_attempts: DEFAULT_COMMIT_ATTEMPTS,
        }
    }
}

#[derive(Clone)]
pub struct UploadRequest {
    pub upload_id: String,
    pub bytes: Bytes,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

impl fmt::Debug for UploadRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UploadRequest")
            .field("upload_id", &self.upload_id)
            .field("size", &self.bytes.len())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadReceipt {
    pub upload_id: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("invalid chunked upload configuration: {0}")]
    Configuration(&'static str),
    #[error("upload validation failed: {0:?}")]
    Validation(UploadErrorCode),
    #[error("upload was rejected: {code:?}")]
    Rejected {
        code: UploadErrorCode,
        expected_index: Option<u64>,
    },
    #[error("NATS request failed: {0}")]
    Transport(#[source] async_nats::RequestError),
    #[error("received an invalid chunked upload response")]
    InvalidResponse,
    #[error("chunked upload attempts were exhausted")]
    AttemptsExhausted,
}

impl UploadError {
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Rejected { code, .. } | Self::Validation(code) => code.is_retryable(),
            Self::Transport(error) => is_retryable_transport(error.kind()),
            Self::AttemptsExhausted => true,
            Self::Configuration(_) | Self::InvalidResponse => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkedUploadClient {
    client: async_nats::Client,
    config: ChunkedUploadClientConfig,
}

impl ChunkedUploadClient {
    pub fn new(client: async_nats::Client, config: ChunkedUploadClientConfig) -> Self {
        Self { client, config }
    }

    pub async fn upload(&self, request: UploadRequest) -> Result<UploadReceipt, UploadError> {
        self.validate_config()?;
        let size = u64::try_from(request.bytes.len())
            .map_err(|_| UploadError::Validation(UploadErrorCode::UploadTooLarge))?;
        let sha256 = sha256_hex(&request.bytes);
        let begin = BeginRequest {
            upload_id: request.upload_id.clone(),
            size,
            sha256: sha256.clone(),
            filename: request.filename.clone(),
            content_type: request.content_type.clone(),
        };
        validate_request(&begin)?;
        let receipt = UploadReceipt {
            upload_id: request.upload_id.clone(),
            size,
            sha256,
        };
        let begin_payload = serde_json::to_vec(&begin).map_err(|_| UploadError::InvalidResponse)?;
        let begin_subject = begin_subject(&self.config.base_subject);
        let mut last_error = None;
        let mut unresolved_commit = false;

        for _ in 0..self.config.max_session_attempts {
            let begin_response = match self
                .request_json::<BeginResponse>(
                    &begin_subject,
                    Bytes::copy_from_slice(&begin_payload),
                )
                .await
            {
                Ok(response) => response,
                Err(error) if error.is_retryable() => {
                    last_error = Some(error);
                    continue;
                }
                Err(error) => return Err(error),
            };

            let (receiver_id, session_id, chunk_size) = match begin_response {
                BeginResponse::AlreadyStored { upload_id } => {
                    if upload_id != request.upload_id {
                        return Err(UploadError::InvalidResponse);
                    }
                    return Ok(receipt);
                }
                BeginResponse::Ready {
                    receiver_id,
                    session_id,
                    chunk_size,
                    session_ttl_ms,
                } => {
                    let chunk_size =
                        usize::try_from(chunk_size).map_err(|_| UploadError::InvalidResponse)?;
                    if chunk_size == 0 || chunk_size > MAX_CHUNK_SIZE || session_ttl_ms == 0 {
                        return Err(UploadError::InvalidResponse);
                    }
                    unresolved_commit = false;
                    (receiver_id, session_id, chunk_size)
                }
                BeginResponse::Error { code } if code.is_retryable() => {
                    last_error = Some(UploadError::Rejected {
                        code,
                        expected_index: None,
                    });
                    continue;
                }
                BeginResponse::Error { code } => {
                    return Err(UploadError::Rejected {
                        code,
                        expected_index: None,
                    });
                }
            };

            let mut restart_session = false;
            for (index, chunk) in request.bytes.chunks(chunk_size).enumerate() {
                let index = u64::try_from(index)
                    .map_err(|_| UploadError::Validation(UploadErrorCode::UploadTooLarge))?;
                let subject =
                    chunk_subject(&self.config.base_subject, receiver_id, session_id, index);
                let mut accepted = false;

                for _ in 0..self.config.chunk_attempts {
                    match self
                        .request_json::<ChunkResponse>(&subject, Bytes::copy_from_slice(chunk))
                        .await
                    {
                        Ok(ChunkResponse::Accepted {
                            index: accepted_index,
                        }) => {
                            if accepted_index != index {
                                return Err(UploadError::InvalidResponse);
                            }
                            accepted = true;
                            break;
                        }
                        Ok(ChunkResponse::Error {
                            code: UploadErrorCode::SessionNotFound,
                            ..
                        }) => {
                            last_error = Some(UploadError::Rejected {
                                code: UploadErrorCode::SessionNotFound,
                                expected_index: None,
                            });
                            restart_session = true;
                            break;
                        }
                        Ok(ChunkResponse::Error {
                            code,
                            expected_index,
                        }) if code.is_retryable() => {
                            last_error = Some(UploadError::Rejected {
                                code,
                                expected_index,
                            });
                        }
                        Ok(ChunkResponse::Error {
                            code,
                            expected_index,
                        }) => {
                            return Err(UploadError::Rejected {
                                code,
                                expected_index,
                            });
                        }
                        Err(error) if error.is_retryable() => last_error = Some(error),
                        Err(error) => return Err(error),
                    }
                }

                if restart_session {
                    break;
                }
                if !accepted {
                    restart_session = true;
                    break;
                }
            }

            if restart_session {
                continue;
            }

            let commit_subject = commit_subject(&self.config.base_subject, receiver_id, session_id);
            unresolved_commit = true;
            for _ in 0..self.config.commit_attempts {
                match self
                    .request_json::<CommitResponse>(&commit_subject, Bytes::new())
                    .await
                {
                    Ok(CommitResponse::Stored { upload_id }) => {
                        if upload_id != request.upload_id {
                            return Err(UploadError::InvalidResponse);
                        }
                        return Ok(receipt);
                    }
                    Ok(CommitResponse::Error {
                        code: UploadErrorCode::SessionNotFound,
                    }) => {
                        last_error = Some(UploadError::Rejected {
                            code: UploadErrorCode::SessionNotFound,
                            expected_index: None,
                        });
                        restart_session = true;
                        break;
                    }
                    Ok(CommitResponse::Error { code }) if code.is_retryable() => {
                        last_error = Some(UploadError::Rejected {
                            code,
                            expected_index: None,
                        });
                    }
                    Ok(CommitResponse::Error { code }) => {
                        return Err(UploadError::Rejected {
                            code,
                            expected_index: None,
                        });
                    }
                    Err(error) if error.is_retryable() => last_error = Some(error),
                    Err(error) => return Err(error),
                }
            }

            if restart_session {
                continue;
            }
            // A commit response is ambiguous. Begin again so inspect can resolve a
            // successfully stored upload whose response was lost.
        }

        if unresolved_commit {
            match self
                .request_json::<BeginResponse>(
                    &begin_subject,
                    Bytes::copy_from_slice(&begin_payload),
                )
                .await
            {
                Ok(BeginResponse::AlreadyStored { upload_id })
                    if upload_id == request.upload_id =>
                {
                    return Ok(receipt);
                }
                Ok(BeginResponse::AlreadyStored { .. } | BeginResponse::Ready { .. }) => {}
                Ok(BeginResponse::Error { code }) if !code.is_retryable() => {
                    return Err(UploadError::Rejected {
                        code,
                        expected_index: None,
                    });
                }
                Ok(BeginResponse::Error { code }) => {
                    last_error = Some(UploadError::Rejected {
                        code,
                        expected_index: None,
                    });
                }
                Err(error) if error.is_retryable() => last_error = Some(error),
                Err(error) => return Err(error),
            }
        }

        Err(last_error.unwrap_or(UploadError::AttemptsExhausted))
    }

    async fn request_json<T: DeserializeOwned>(
        &self,
        subject: &str,
        payload: Bytes,
    ) -> Result<T, UploadError> {
        let request = async_nats::Request::new()
            .timeout(Some(self.config.request_timeout))
            .payload(payload);
        let response = self
            .client
            .send_request(subject.to_string(), request)
            .await
            .map_err(UploadError::Transport)?;
        serde_json::from_slice(&response.payload).map_err(|_| UploadError::InvalidResponse)
    }

    fn validate_config(&self) -> Result<(), UploadError> {
        if !is_valid_base_subject(&self.config.base_subject) {
            return Err(UploadError::Configuration("invalid base subject"));
        }
        if self.config.request_timeout.is_zero() {
            return Err(UploadError::Configuration(
                "request timeout must be greater than zero",
            ));
        }
        if self.config.max_session_attempts == 0
            || self.config.chunk_attempts == 0
            || self.config.commit_attempts == 0
        {
            return Err(UploadError::Configuration(
                "attempt counts must be greater than zero",
            ));
        }
        Ok(())
    }
}

fn validate_request(request: &BeginRequest) -> Result<(), UploadError> {
    if request.upload_id.is_empty()
        || request.upload_id.len() > DEFAULT_MAX_UPLOAD_ID_LENGTH
        || request.upload_id.chars().any(char::is_control)
    {
        return Err(UploadError::Validation(UploadErrorCode::InvalidUploadId));
    }
    if !is_valid_digest(&request.sha256) {
        return Err(UploadError::Validation(UploadErrorCode::InvalidDigest));
    }
    if request
        .filename
        .as_ref()
        .is_some_and(|value| value.len() > DEFAULT_MAX_FILENAME_LENGTH)
        || request
            .content_type
            .as_ref()
            .is_some_and(|value| value.len() > DEFAULT_MAX_CONTENT_TYPE_LENGTH)
    {
        return Err(UploadError::Validation(UploadErrorCode::InvalidMetadata));
    }
    Ok(())
}

pub(crate) fn is_valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

fn is_retryable_transport(kind: RequestErrorKind) -> bool {
    matches!(
        kind,
        RequestErrorKind::TimedOut | RequestErrorKind::NoResponders | RequestErrorKind::Other
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn begin_request(digest: &str) -> BeginRequest {
        BeginRequest {
            upload_id: "media-123".to_string(),
            size: 3,
            sha256: digest.to_string(),
            filename: None,
            content_type: None,
        }
    }

    #[test]
    fn validates_exact_lowercase_sha256() {
        let valid = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        assert!(validate_request(&begin_request(valid)).is_ok());
        assert_eq!(
            validate_request(&begin_request(&valid.to_uppercase())).unwrap_err_code(),
            UploadErrorCode::InvalidDigest
        );
        assert_eq!(
            validate_request(&begin_request(&valid[..63])).unwrap_err_code(),
            UploadErrorCode::InvalidDigest
        );
        assert_eq!(
            validate_request(&begin_request(&format!("{}g", &valid[..63]))).unwrap_err_code(),
            UploadErrorCode::InvalidDigest
        );
    }

    #[test]
    fn validates_upload_id_and_metadata_lengths() {
        let digest = "a".repeat(64);
        let mut request = begin_request(&digest);
        request.upload_id.clear();
        assert_eq!(
            validate_request(&request).unwrap_err_code(),
            UploadErrorCode::InvalidUploadId
        );

        request.upload_id = "id".to_string();
        request.filename = Some("x".repeat(DEFAULT_MAX_FILENAME_LENGTH + 1));
        assert_eq!(
            validate_request(&request).unwrap_err_code(),
            UploadErrorCode::InvalidMetadata
        );
    }

    #[test]
    fn hashes_bytes_as_lowercase_hex() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    trait ValidationResultExt {
        fn unwrap_err_code(self) -> UploadErrorCode;
    }

    impl ValidationResultExt for Result<(), UploadError> {
        fn unwrap_err_code(self) -> UploadErrorCode {
            match self.expect_err("validation should fail") {
                UploadError::Validation(code) => code,
                error => panic!("unexpected error: {error}"),
            }
        }
    }
}
