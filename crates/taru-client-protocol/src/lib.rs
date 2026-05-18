use serde::{Deserialize, Serialize};

mod catalog;

pub use catalog::*;

pub const CLIENT_PROTOCOL_VERSION: &str = "v1";
pub const API_VERSION_HEADER: &str = "x-taru-api-version";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublicClientRoute {
    pub path: &'static str,
    pub methods: &'static [PublicClientHttpMethod],
    pub kind: PublicClientRouteKind,
    pub rust_sdk_exposure: PublicClientRustSdkExposure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicClientHttpMethod {
    Get,
    Head,
    Post,
}

impl PublicClientHttpMethod {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Post => "POST",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicClientRouteKind {
    System,
    Library,
    Catalog,
    Playback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicClientRustSdkExposure {
    JsonMethod,
    StreamingBuilder,
}

pub const PUBLIC_CLIENT_ROUTES: &[PublicClientRoute] = &[
    PublicClientRoute {
        path: "/health",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::System,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/libraries",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Library,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/libraries/{library_id}",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Library,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/libraries/{library_id}/sources",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Library,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/items",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/items/{item_id}",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/items/{item_id}/credits",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/items/{item_id}/images",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/people",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/people/{person_id}",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/people/{person_id}/items",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/tags",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/tags/{tag_id}/items",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/genres",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/genres/{genre_id}/items",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/search",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/sources/{source_id}/probe",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Playback,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/sources/{source_id}/playback/decision",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Playback,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/sources/{source_id}/stream",
        methods: &[PublicClientHttpMethod::Get, PublicClientHttpMethod::Head],
        kind: PublicClientRouteKind::Playback,
        rust_sdk_exposure: PublicClientRustSdkExposure::StreamingBuilder,
    },
    PublicClientRoute {
        path: "/sources/{source_id}/stream/remux",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Playback,
        rust_sdk_exposure: PublicClientRustSdkExposure::StreamingBuilder,
    },
    PublicClientRoute {
        path: "/sources/{source_id}/stream/hls/playlist.m3u8",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Playback,
        rust_sdk_exposure: PublicClientRustSdkExposure::StreamingBuilder,
    },
    PublicClientRoute {
        path: "/playback/sessions/{session_id}",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Playback,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/playback/sessions/{session_id}/cancel",
        methods: &[PublicClientHttpMethod::Post],
        kind: PublicClientRouteKind::Playback,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/playback/sessions/{session_id}/hls/segments/{segment_name}",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Playback,
        rust_sdk_exposure: PublicClientRustSdkExposure::StreamingBuilder,
    },
];

#[must_use]
pub fn public_client_paths() -> impl ExactSizeIterator<Item = &'static str> {
    PUBLIC_CLIENT_ROUTES.iter().map(|route| route.path)
}

#[must_use]
pub fn public_client_json_routes() -> impl Iterator<Item = PublicClientRoute> {
    PUBLIC_CLIENT_ROUTES
        .iter()
        .copied()
        .filter(|route| route.rust_sdk_exposure == PublicClientRustSdkExposure::JsonMethod)
}

#[must_use]
pub fn public_client_streaming_routes() -> impl Iterator<Item = PublicClientRoute> {
    PUBLIC_CLIENT_ROUTES
        .iter()
        .copied()
        .filter(|route| route.rust_sdk_exposure == PublicClientRustSdkExposure::StreamingBuilder)
}

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
    fn public_route_inventory_is_protocol_owned_and_complete() {
        let paths = public_client_paths().collect::<Vec<_>>();

        assert_eq!(paths.len(), 24);
        assert!(paths.contains(&"/health"));
        assert!(paths.contains(&"/sources/{source_id}/stream"));
        assert!(paths.contains(&"/playback/sessions/{session_id}/hls/segments/{segment_name}"));

        let direct_stream = PUBLIC_CLIENT_ROUTES
            .iter()
            .find(|route| route.path == "/sources/{source_id}/stream")
            .expect("direct stream route exists");
        assert_eq!(direct_stream.kind, PublicClientRouteKind::Playback);
        assert_eq!(
            direct_stream.rust_sdk_exposure,
            PublicClientRustSdkExposure::StreamingBuilder
        );
        assert_eq!(
            direct_stream
                .methods
                .iter()
                .map(|method| method.as_str())
                .collect::<Vec<_>>(),
            vec!["GET", "HEAD"]
        );

        let json_count = public_client_json_routes().count();
        let streaming_count = public_client_streaming_routes().count();
        assert_eq!(json_count, 20);
        assert_eq!(streaming_count, 4);
        assert_eq!(json_count + streaming_count, PUBLIC_CLIENT_ROUTES.len());
    }

    #[test]
    fn public_route_inventory_rejects_internal_and_secret_surfaces() {
        let serialized = PUBLIC_CLIENT_ROUTES
            .iter()
            .map(|route| route.path)
            .collect::<Vec<_>>()
            .join("\n")
            .to_ascii_lowercase();

        for forbidden in [
            "/addons",
            "/webhooks",
            "/automation",
            "/storage/backends",
            "/jobs",
            "secret_env",
            "output_path",
            "providerrawresponse",
            "taru_core",
            "taru-api",
            "taru-server",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "public route inventory leaked forbidden term: {forbidden}"
            );
        }
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
        assert!(value["source"].get("locator").is_none());
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
