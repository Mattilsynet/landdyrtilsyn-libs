use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::{dokument::DokumentId, journalpost::JournalpostId, sak::Saksnummer};

/// Client-facing lifecycle event for Skuffen processing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SkuffenStatusEventV1 {
    pub command_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Uuid>,
    pub phase: SkuffenStatusPhase,
    pub status: SkuffenStatus,
    pub terminal: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<SkuffenStatusErrorCode>,
    /// Client-safe message only. Skal ikke lekke interne states eller stacktraces.
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempt: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saksnummer: Option<Saksnummer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journalpost_id: Option<JournalpostId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dokument_id: Option<Vec<DokumentId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Processing phase exposed to clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkuffenStatusPhase {
    Ingest,
    Validate,
    Execution,
}

/// Simple client-facing outcome for a given phase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkuffenStatus {
    Pending,
    Ok,
    Blocked,
    Retrying,
    Error,
}

/// Client-safe error codes. Disse er bevisst coarse-grained.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkuffenStatusErrorCode {
    DuplicateRequest,
    InvalidRequest,
    NotFound,
    Conflict,
    PrerequisitePending,
    TemporaryUnavailable,
    ProcessingFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn serializes_success_event_with_generated_ids() {
        let event = SkuffenStatusEventV1 {
            command_id: Uuid::parse_str("123e4567-e89b-12d3-a456-426614174009").unwrap(),
            correlation_id: Some(Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap()),
            phase: SkuffenStatusPhase::Execution,
            status: SkuffenStatus::Ok,
            terminal: true,
            error_code: None,
            message: "Journalpost opprettet.".to_string(),
            attempt: Some(1),
            saksnummer: Some(Saksnummer::new("2026/123").unwrap()),
            journalpost_id: Some(JournalpostId("jp-123".to_string())),
            dokument_id: Some(vec![
                DokumentId("dok-1".to_string()),
                DokumentId("dok-2".to_string()),
            ]),
            timestamp: Some("2026-01-01T12:00:00Z".to_string()),
        };

        let value = serde_json::to_value(&event).unwrap();

        assert_eq!(
            value,
            json!({
                "command_id": "123e4567-e89b-12d3-a456-426614174009",
                "correlation_id": "123e4567-e89b-12d3-a456-426614174000",
                "phase": "execution",
                "status": "ok",
                "terminal": true,
                "message": "Journalpost opprettet.",
                "attempt": 1,
                "saksnummer": "2026/123",
                "journalpost_id": "jp-123",
                "dokument_id": ["dok-1", "dok-2"],
                "timestamp": "2026-01-01T12:00:00Z"
            })
        );
    }

    #[test]
    fn serializes_error_event_without_optional_ids() {
        let event = SkuffenStatusEventV1 {
            command_id: Uuid::parse_str("123e4567-e89b-12d3-a456-426614174010").unwrap(),
            correlation_id: Some(Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap()),
            phase: SkuffenStatusPhase::Validate,
            status: SkuffenStatus::Error,
            terminal: true,
            error_code: Some(SkuffenStatusErrorCode::InvalidRequest),
            message: "Request could not be validated.".to_string(),
            attempt: None,
            saksnummer: None,
            journalpost_id: None,
            dokument_id: None,
            timestamp: None,
        };

        let value = serde_json::to_value(&event).unwrap();

        assert_eq!(
            value,
            json!({
                "command_id": "123e4567-e89b-12d3-a456-426614174010",
                "correlation_id": "123e4567-e89b-12d3-a456-426614174001",
                "phase": "validate",
                "status": "error",
                "terminal": true,
                "error_code": "INVALID_REQUEST",
                "message": "Request could not be validated."
            })
        );
    }

    #[test]
    fn deserializes_expected_shape() {
        let value = json!({
            "command_id": "123e4567-e89b-12d3-a456-426614174011",
            "correlation_id": "123e4567-e89b-12d3-a456-426614174002",
            "phase": "ingest",
            "status": "pending",
            "terminal": false,
            "message": "Request accepted.",
            "saksnummer": "2026/999",
            "journalpost_id": "jp-999",
            "dokument_id": ["dok-9"]
        });

        let event: SkuffenStatusEventV1 = serde_json::from_value(value).unwrap();

        assert_eq!(
            event,
            SkuffenStatusEventV1 {
                command_id: Uuid::parse_str("123e4567-e89b-12d3-a456-426614174011").unwrap(),
                correlation_id: Some(
                    Uuid::parse_str("123e4567-e89b-12d3-a456-426614174002").unwrap(),
                ),
                phase: SkuffenStatusPhase::Ingest,
                status: SkuffenStatus::Pending,
                terminal: false,
                error_code: None,
                message: "Request accepted.".to_string(),
                attempt: None,
                saksnummer: Some(Saksnummer::new("2026/999").unwrap()),
                journalpost_id: Some(JournalpostId("jp-999".to_string())),
                dokument_id: Some(vec![DokumentId("dok-9".to_string())]),
                timestamp: None,
            }
        );
    }

    #[test]
    fn serializes_without_correlation_id_when_missing() {
        let event = SkuffenStatusEventV1 {
            command_id: Uuid::parse_str("123e4567-e89b-12d3-a456-426614174012").unwrap(),
            correlation_id: None,
            phase: SkuffenStatusPhase::Execution,
            status: SkuffenStatus::Retrying,
            terminal: false,
            error_code: Some(SkuffenStatusErrorCode::TemporaryUnavailable),
            message: "Temporary issue while processing command.".to_string(),
            attempt: Some(2),
            saksnummer: None,
            journalpost_id: None,
            dokument_id: None,
            timestamp: None,
        };

        let value = serde_json::to_value(&event).unwrap();

        assert_eq!(
            value,
            json!({
                "command_id": "123e4567-e89b-12d3-a456-426614174012",
                "phase": "execution",
                "status": "retrying",
                "terminal": false,
                "error_code": "TEMPORARY_UNAVAILABLE",
                "message": "Temporary issue while processing command.",
                "attempt": 2
            })
        );
    }

    #[test]
    fn rejects_unknown_fields() {
        let value = json!({
            "command_id": "123e4567-e89b-12d3-a456-426614174013",
            "correlation_id": "123e4567-e89b-12d3-a456-426614174003",
            "phase": "execution",
            "status": "error",
            "terminal": true,
            "error_code": "PROCESSING_FAILED",
            "message": "Request could not be completed.",
            "internal_state": "do-not-leak"
        });

        let error = serde_json::from_value::<SkuffenStatusEventV1>(value).unwrap_err();

        let error_message = error.to_string();
        assert!(error_message.contains("unknown field `internal_state`"));
    }

    #[test]
    fn dokument_id_serializes_as_string() {
        let value = serde_json::to_value(DokumentId("dok-42".to_string())).unwrap();

        assert_eq!(value, Value::String("dok-42".to_string()));
    }
}
