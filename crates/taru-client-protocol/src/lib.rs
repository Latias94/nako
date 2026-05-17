use serde::{Deserialize, Serialize};

mod catalog;

pub use catalog::*;

pub const CLIENT_PROTOCOL_VERSION: &str = "v1";
pub const API_VERSION_HEADER: &str = "x-taru-api-version";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    #[must_use]
    pub fn new(code: ClientErrorCode, message: impl Into<String>) -> Self {
        Self {
            code: code.as_str().to_owned(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientErrorCode {
    InvalidInput,
    NotFound,
    Conflict,
    Unsupported,
    Unauthorized,
    Forbidden,
    ProviderError,
    StorageError,
    FfmpegError,
    StagingBudgetExhausted,
    StagingValidationMismatch,
    StorageTimeout,
    StorageUnauthorized,
    StorageRateLimited,
    DatabaseError,
}

impl ClientErrorCode {
    pub const ALL: &'static [Self] = &[
        Self::InvalidInput,
        Self::NotFound,
        Self::Conflict,
        Self::Unsupported,
        Self::Unauthorized,
        Self::Forbidden,
        Self::ProviderError,
        Self::StorageError,
        Self::FfmpegError,
        Self::StagingBudgetExhausted,
        Self::StagingValidationMismatch,
        Self::StorageTimeout,
        Self::StorageUnauthorized,
        Self::StorageRateLimited,
        Self::DatabaseError,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::Unsupported => "unsupported",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::ProviderError => "provider_error",
            Self::StorageError => "storage_error",
            Self::FfmpegError => "ffmpeg_error",
            Self::StagingBudgetExhausted => "staging_budget_exhausted",
            Self::StagingValidationMismatch => "staging_validation_mismatch",
            Self::StorageTimeout => "storage_timeout",
            Self::StorageUnauthorized => "storage_unauthorized",
            Self::StorageRateLimited => "storage_rate_limited",
            Self::DatabaseError => "database_error",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == code)
    }
}

impl From<ClientErrorCode> for String {
    fn from(value: ClientErrorCode) -> Self {
        value.as_str().to_owned()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PageInfo {
    pub limit: u32,
    pub offset: u64,
    pub returned: u32,
}

impl PageInfo {
    #[must_use]
    pub const fn new(limit: u32, offset: u64, returned: u32) -> Self {
        Self {
            limit,
            offset,
            returned,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_system_envelopes_serialize_without_server_types() {
        let health = HealthResponse {
            status: "ok".to_owned(),
            version: CLIENT_PROTOCOL_VERSION.to_owned(),
        };
        let error = ErrorResponse {
            code: "not_found".to_owned(),
            message: "missing".to_owned(),
        };
        let page = PageInfo::new(50, 100, 3);

        let health_json = serde_json::to_value(&health).unwrap();
        let error_json = serde_json::to_value(&error).unwrap();
        let page_json = serde_json::to_value(page).unwrap();

        assert_eq!(health_json["version"], "v1");
        assert_eq!(API_VERSION_HEADER, "x-taru-api-version");
        assert_eq!(error_json["code"], "not_found");
        assert_eq!(page_json["limit"], 50);
        assert_eq!(page_json["offset"], 100);
        assert_eq!(page_json["returned"], 3);
    }

    #[test]
    fn public_error_codes_are_stable_wire_values() {
        let response = ErrorResponse::new(ClientErrorCode::StorageTimeout, "storage timed out");
        let response_json = serde_json::to_value(&response).unwrap();
        let code_json = serde_json::to_value(ClientErrorCode::StorageTimeout).unwrap();

        assert_eq!(response.code, "storage_timeout");
        assert_eq!(response_json["code"], "storage_timeout");
        assert_eq!(code_json, "storage_timeout");
        assert_eq!(
            ClientErrorCode::from_code("storage_timeout"),
            Some(ClientErrorCode::StorageTimeout)
        );
        assert_eq!(
            ClientErrorCode::from_code("unauthorized"),
            Some(ClientErrorCode::Unauthorized)
        );
        assert_eq!(ClientErrorCode::from_code("server_stack_trace"), None);
    }

    #[test]
    fn public_browse_dtos_use_wire_ids_and_client_enums() {
        let item = MediaItemDto {
            id: "item-1".to_owned(),
            kind: ClientMediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadataDto {
                title: "Protocol Demo".to_owned(),
                original_title: None,
                sort_title: None,
                overview: None,
                release_date: None,
                runtime_minutes: None,
                tagline: None,
                genres: vec!["demo".to_owned()],
                tags: Vec::new(),
                ratings: Vec::new(),
                images: Vec::new(),
                credits: Vec::new(),
                collections: Vec::new(),
                studios: Vec::new(),
                external_ids: Vec::new(),
            },
        };

        let value = serde_json::to_value(&item).unwrap();

        assert_eq!(value["id"], "item-1");
        assert_eq!(value["kind"], "movie");
        assert_eq!(value["metadata"]["title"], "Protocol Demo");
        assert!(value.get("input_json").is_none());
    }

    #[test]
    fn public_playback_decision_uses_protocol_owned_types() {
        let response = PlaybackDecisionResponse {
            source: MediaSourceDto {
                id: "source-1".to_owned(),
                library_id: "library-1".to_owned(),
                item_id: "item-1".to_owned(),
                locator: "local:///Demo.mp4".to_owned(),
                file_name: "Demo.mp4".to_owned(),
                size_bytes: Some(42),
                fingerprint: None,
            },
            probe: None,
            decision: ClientPlaybackDecision {
                mode: ClientPlaybackMode::DirectPlay,
                reason: "compatible".to_owned(),
                direct_play: Some(ClientDirectPlayPlan {
                    source_id: "source-1".to_owned(),
                    content_type: "video/mp4".to_owned(),
                    supports_range_requests: true,
                }),
                transcode_plan: None,
            },
        };

        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["decision"]["mode"], "direct_play");
        assert_eq!(value["decision"]["direct_play"]["source_id"], "source-1");
        assert!(value["decision"].get("transcode_plan").is_some());
    }

    #[test]
    fn public_transcode_session_response_hides_server_paths() {
        let response = TranscodeSessionResponse {
            session: TranscodeSessionDto {
                id: "session-1".to_owned(),
                source_id: "source-1".to_owned(),
                kind: ClientTranscodeSessionKind::HlsTranscode,
                request_key: "hls:source-1".to_owned(),
                state: ClientTranscodeSessionState::Running,
                failure_category: None,
                failure_message: None,
                created_at: "2026-05-17T00:00:00Z".to_owned(),
                updated_at: "2026-05-17T00:01:00Z".to_owned(),
                started_at: Some("2026-05-17T00:00:01Z".to_owned()),
                completed_at: None,
            },
        };

        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["session"]["kind"], "hls_transcode");
        assert_eq!(value["session"]["state"], "running");
        assert!(value["session"].get("output_path").is_none());
    }
}
