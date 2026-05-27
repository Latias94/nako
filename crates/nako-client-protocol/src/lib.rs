use serde::{Deserialize, Serialize};

mod catalog;

pub use catalog::*;

pub const CLIENT_PROTOCOL_VERSION: &str = "v1";
pub const API_VERSION_HEADER: &str = "x-nako-api-version";
pub const PLAYBACK_SESSION_ID_HEADER: &str = "x-nako-playback-session-id";

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
    Put,
}

impl PublicClientHttpMethod {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Post => "POST",
            Self::Put => "PUT",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicClientRouteKind {
    System,
    Account,
    Library,
    Catalog,
    Management,
    Playback,
    Renderer,
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
        path: "/auth/login",
        methods: &[PublicClientHttpMethod::Post],
        kind: PublicClientRouteKind::Account,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/auth/invitations/redeem",
        methods: &[PublicClientHttpMethod::Post],
        kind: PublicClientRouteKind::Account,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/auth/logout",
        methods: &[PublicClientHttpMethod::Post],
        kind: PublicClientRouteKind::Account,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/users/me",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Account,
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
        path: "/images/{image_id}",
        methods: &[PublicClientHttpMethod::Get, PublicClientHttpMethod::Head],
        kind: PublicClientRouteKind::Catalog,
        rust_sdk_exposure: PublicClientRustSdkExposure::StreamingBuilder,
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
        path: "/management/context-links",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Management,
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
        path: "/sources/{source_id}/playback/browser-ticket",
        methods: &[PublicClientHttpMethod::Post],
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
        methods: &[PublicClientHttpMethod::Get, PublicClientHttpMethod::Head],
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
        path: "/playback/sessions/{session_id}/heartbeat",
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
    PublicClientRoute {
        path: "/renderers",
        methods: &[PublicClientHttpMethod::Get, PublicClientHttpMethod::Post],
        kind: PublicClientRouteKind::Renderer,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/renderers/{renderer_session_id}/heartbeat",
        methods: &[PublicClientHttpMethod::Post],
        kind: PublicClientRouteKind::Renderer,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/renderers/{renderer_session_id}/commands/next",
        methods: &[PublicClientHttpMethod::Post],
        kind: PublicClientRouteKind::Renderer,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/renderers/{renderer_session_id}/commands/play",
        methods: &[PublicClientHttpMethod::Post],
        kind: PublicClientRouteKind::Renderer,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/renderers/{renderer_session_id}/commands/{command_id}/complete",
        methods: &[PublicClientHttpMethod::Post],
        kind: PublicClientRouteKind::Renderer,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/users/me/playback-state/items/{item_id}",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Playback,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/users/me/playback-state/continue-watching",
        methods: &[PublicClientHttpMethod::Get],
        kind: PublicClientRouteKind::Playback,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/users/me/playback-state/items/{item_id}/progress",
        methods: &[PublicClientHttpMethod::Put],
        kind: PublicClientRouteKind::Playback,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
    },
    PublicClientRoute {
        path: "/users/me/playback-state/items/{item_id}/watched",
        methods: &[PublicClientHttpMethod::Put],
        kind: PublicClientRouteKind::Playback,
        rust_sdk_exposure: PublicClientRustSdkExposure::JsonMethod,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LoginResponse {
    pub session: UserSessionDto,
    pub account: CurrentUserResponse,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RedeemInvitationRequest {
    pub token: String,
    pub username: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CurrentUserResponse {
    pub user: CurrentUserDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CurrentUserDto {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub roles: Vec<String>,
    pub bootstrap: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserSessionDto {
    pub token: String,
    pub expires_at_ms: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LogoutResponse {
    pub revoked: bool,
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
        assert_eq!(API_VERSION_HEADER, "x-nako-api-version");
        assert_eq!(PLAYBACK_SESSION_ID_HEADER, "x-nako-playback-session-id");
        assert_eq!(error_json["code"], "not_found");
        assert_eq!(page_json["limit"], 50);
        assert_eq!(page_json["offset"], 100);
        assert_eq!(page_json["returned"], 3);
    }

    #[test]
    fn public_route_inventory_is_protocol_owned_and_complete() {
        let paths = public_client_paths().collect::<Vec<_>>();

        assert_eq!(paths.len(), 41);
        assert!(paths.contains(&"/health"));
        assert!(paths.contains(&"/auth/login"));
        assert!(paths.contains(&"/auth/invitations/redeem"));
        assert!(paths.contains(&"/auth/logout"));
        assert!(paths.contains(&"/users/me"));
        assert!(paths.contains(&"/management/context-links"));
        assert!(paths.contains(&"/images/{image_id}"));
        assert!(paths.contains(&"/sources/{source_id}/stream"));
        assert!(paths.contains(&"/sources/{source_id}/playback/browser-ticket"));
        assert!(paths.contains(&"/playback/sessions/{session_id}/heartbeat"));
        assert!(paths.contains(&"/playback/sessions/{session_id}/hls/segments/{segment_name}"));
        assert!(paths.contains(&"/renderers"));
        assert!(paths.contains(&"/renderers/{renderer_session_id}/heartbeat"));
        assert!(paths.contains(&"/renderers/{renderer_session_id}/commands/next"));
        assert!(paths.contains(&"/renderers/{renderer_session_id}/commands/play"));
        assert!(paths.contains(&"/renderers/{renderer_session_id}/commands/{command_id}/complete"));
        assert!(paths.contains(&"/users/me/playback-state/items/{item_id}"));
        assert!(paths.contains(&"/users/me/playback-state/continue-watching"));
        assert!(paths.contains(&"/users/me/playback-state/items/{item_id}/progress"));
        assert!(paths.contains(&"/users/me/playback-state/items/{item_id}/watched"));

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

        let browser_ticket = PUBLIC_CLIENT_ROUTES
            .iter()
            .find(|route| route.path == "/sources/{source_id}/playback/browser-ticket")
            .expect("browser playback ticket route exists");
        assert_eq!(browser_ticket.kind, PublicClientRouteKind::Playback);
        assert_eq!(
            browser_ticket.rust_sdk_exposure,
            PublicClientRustSdkExposure::JsonMethod
        );
        assert_eq!(
            browser_ticket
                .methods
                .iter()
                .map(|method| method.as_str())
                .collect::<Vec<_>>(),
            vec!["POST"]
        );

        let json_count = public_client_json_routes().count();
        let streaming_count = public_client_streaming_routes().count();
        assert_eq!(json_count, 36);
        assert_eq!(streaming_count, 5);
        assert_eq!(json_count + streaming_count, PUBLIC_CLIENT_ROUTES.len());
        let remux_stream = PUBLIC_CLIENT_ROUTES
            .iter()
            .find(|route| route.path == "/sources/{source_id}/stream/remux")
            .expect("remux stream route exists");
        assert_eq!(
            remux_stream
                .methods
                .iter()
                .map(|method| method.as_str())
                .collect::<Vec<_>>(),
            vec!["GET", "HEAD"]
        );
    }

    #[test]
    fn public_route_inventory_has_renderer_session_surface_without_external_cast_routes() {
        let paths = public_client_paths().collect::<Vec<_>>();

        assert!(paths.contains(&"/playback/sessions/{session_id}"));
        assert!(paths.contains(&"/playback/sessions/{session_id}/heartbeat"));
        assert!(paths.contains(&"/playback/sessions/{session_id}/cancel"));
        assert!(paths.contains(&"/renderers"));
        assert!(paths.contains(&"/renderers/{renderer_session_id}/heartbeat"));
        assert!(paths.contains(&"/renderers/{renderer_session_id}/commands/next"));
        assert!(paths.contains(&"/renderers/{renderer_session_id}/commands/play"));
        assert!(paths.iter().all(|path| !path.contains("cast")));
        assert!(
            PUBLIC_CLIENT_ROUTES
                .iter()
                .filter(|route| route.kind == PublicClientRouteKind::Playback)
                .all(|route| route.path.starts_with("/sources/")
                    || route.path.starts_with("/playback/")
                    || route.path.starts_with("/users/me/playback-state/"))
        );
        assert!(
            PUBLIC_CLIENT_ROUTES
                .iter()
                .filter(|route| route.kind == PublicClientRouteKind::Renderer)
                .all(|route| route.path.starts_with("/renderers"))
        );
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
            "nako_core",
            "nako-api",
            "nako-server",
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
            target: ClientPlaybackTargetDto {
                kind: ClientPlaybackTargetKind::Browser,
                network_scope: ClientPlaybackTargetNetworkScope::Local,
                transport_auth: ClientPlaybackTargetTransportAuth::BrowserTicket,
                media_capabilities: ClientPlaybackCapabilitiesDto {
                    direct_play: true,
                    containers: vec!["mp4".to_owned()],
                    video_codecs: vec!["h264".to_owned()],
                    audio_codecs: vec!["aac".to_owned()],
                },
                control_capabilities: ClientRendererControlCapabilitiesDto {
                    commands: Vec::new(),
                },
            },
            decision: ClientPlaybackDecision {
                mode: ClientPlaybackMode::DirectPlay,
                reason: ClientPlaybackDecisionReason::Compatible,
                denial: None,
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
        assert_eq!(value["decision"]["reason"], "compatible");
        assert_eq!(value["target"]["kind"], "browser");
        assert_eq!(value["target"]["transport_auth"], "browser_ticket");
        assert_eq!(value["decision"]["direct_play"]["source_id"], "source-1");
        assert!(value["source"].get("locator").is_none());
        assert!(value["decision"].get("transcode_plan").is_some());
        assert!(value["decision"].get("denial").is_some());
    }

    #[test]
    fn public_browser_playback_ticket_uses_protocol_owned_safe_urls() {
        let empty_request = serde_json::to_value(BrowserPlaybackTicketRequest {
            mode: BrowserPlaybackMode::Direct,
            capabilities: None,
        })
        .unwrap();

        assert_eq!(empty_request["mode"], "direct");
        assert!(empty_request.get("capabilities").is_none());

        let request = BrowserPlaybackTicketRequest {
            mode: BrowserPlaybackMode::Hls,
            capabilities: Some(BrowserPlaybackCapabilitiesDto {
                direct_play: Some(true),
                container: Some(vec!["mp4".to_owned(), "webm".to_owned()]),
                video_codec: Some(vec!["h264".to_owned()]),
                audio_codec: Some(vec!["aac".to_owned()]),
                output_container: Some(BrowserPlaybackOutputContainer::Mp4),
            }),
        };

        let request_value = serde_json::to_value(request).unwrap();
        assert_eq!(request_value["mode"], "hls");
        assert_eq!(request_value["capabilities"]["direct_play"], true);
        assert_eq!(request_value["capabilities"]["container"][0], "mp4");
        assert_eq!(request_value["capabilities"]["output_container"], "mp4");

        let response = BrowserPlaybackTicketResponse {
            source_id: "source-1".to_owned(),
            item_id: Some("item-1".to_owned()),
            mode: BrowserPlaybackMode::Hls,
            expires_at: "2026-05-26T12:00:00Z".to_owned(),
            urls: vec![BrowserPlaybackUrlDto {
                kind: BrowserPlaybackUrlKind::Playlist,
                url: "/sources/source-1/stream/hls/playlist.m3u8?ticket=opaque".to_owned(),
                content_type: "application/vnd.apple.mpegurl".to_owned(),
                supports_range_requests: false,
            }],
        };

        let value = serde_json::to_value(response).unwrap();
        assert_eq!(value["mode"], "hls");
        assert_eq!(value["urls"][0]["kind"], "playlist");
        assert_eq!(
            value["urls"][0]["content_type"],
            "application/vnd.apple.mpegurl"
        );
        assert!(value.get("locator").is_none());
        assert!(value.get("bearer_token").is_none());
    }

    #[test]
    fn public_renderer_command_transport_uses_typed_safe_envelope() {
        let command = RendererCommandDto {
            id: "command-1".to_owned(),
            renderer_session_id: "renderer-1".to_owned(),
            command: ClientRendererControlCommand::Play,
            state: ClientRendererCommandState::Queued,
            item_id: Some("item-1".to_owned()),
            source_id: Some("source-1".to_owned()),
            playback_session_id: Some("playback-1".to_owned()),
            position_ms: Some(1_000),
            volume_percent: None,
            transport: Some(RendererCommandTransportDto {
                mode: RendererTransportMode::Hls,
                expires_at: "2026-05-27T12:00:00Z".to_owned(),
                urls: vec![RendererCommandTransportUrlDto {
                    kind: RendererTransportUrlKind::Playlist,
                    url: "/sources/source-1/stream/hls/playlist.m3u8?ticket=opaque".to_owned(),
                    content_type: "application/vnd.apple.mpegurl".to_owned(),
                    supports_range_requests: false,
                }],
            }),
            created_at: "2026-05-27T11:00:00Z".to_owned(),
            updated_at: "2026-05-27T11:00:00Z".to_owned(),
        };

        let value = serde_json::to_value(command).unwrap();
        assert_eq!(value["transport"]["mode"], "hls");
        assert_eq!(value["transport"]["urls"][0]["kind"], "playlist");
        assert!(value.get("payload_json").is_none());
        assert!(value.get("bearer_token").is_none());
        assert!(value.get("source_locator").is_none());
        assert!(value.get("transcode_session_id").is_none());
    }

    #[test]
    fn public_wire_values_preserve_unknown_additive_strings() {
        let response = serde_json::from_value::<PlaybackDecisionResponse>(serde_json::json!({
            "source": {
                "id": "source-1",
                "library_id": "library-1",
                "item_id": "item-1",
                "file_name": "Demo.mp4",
                "size_bytes": 42,
                "fingerprint": null
            },
            "probe": null,
            "target": {
                "kind": "server_future_target",
                "network_scope": "server_future_network",
                "transport_auth": "server_future_transport",
                "media_capabilities": {
                    "direct_play": true,
                    "containers": ["mp4"],
                    "video_codecs": ["h264"],
                    "audio_codecs": ["aac"]
                },
                "control_capabilities": {
                    "commands": ["server_future_command"]
                }
            },
            "decision": {
                "mode": "server_future_mode",
                "reason": "server_future_reason",
                "denial": {
                    "permission": "server_future_permission",
                    "reason": "server_future_denial"
                },
                "direct_play": null,
                "transcode_plan": {
                    "output_container": "future_container",
                    "video_codec": null,
                    "audio_codec": null
                }
            }
        }))
        .unwrap();

        assert_eq!(
            response.decision.mode,
            ClientPlaybackMode::Other("server_future_mode".to_owned())
        );
        assert!(!response.decision.mode.is_known());
        assert_eq!(
            response.decision.reason,
            ClientPlaybackDecisionReason::Other("server_future_reason".to_owned())
        );
        assert!(!response.decision.reason.is_known());
        assert_eq!(
            response.target.kind,
            ClientPlaybackTargetKind::Other("server_future_target".to_owned())
        );
        assert_eq!(
            response.target.control_capabilities.commands[0],
            ClientRendererControlCommand::Other("server_future_command".to_owned())
        );
        assert_eq!(
            response.decision.denial.as_ref().unwrap().permission,
            ClientPlaybackPermission::Other("server_future_permission".to_owned())
        );
        let plan = response.decision.transcode_plan.unwrap();
        assert_eq!(
            plan.output_container,
            ClientOutputContainer::Other("future_container".to_owned())
        );

        let encoded = serde_json::to_value(PlaybackDecisionResponse {
            source: response.source,
            probe: None,
            target: response.target,
            decision: ClientPlaybackDecision {
                mode: ClientPlaybackMode::Other("server_future_mode".to_owned()),
                reason: ClientPlaybackDecisionReason::Other("server_future_reason".to_owned()),
                denial: Some(ClientPlaybackDenialDto {
                    permission: ClientPlaybackPermission::Other(
                        "server_future_permission".to_owned(),
                    ),
                    reason: ClientPlaybackPermissionDecisionReason::Other(
                        "server_future_denial".to_owned(),
                    ),
                }),
                direct_play: None,
                transcode_plan: Some(plan),
            },
        })
        .unwrap();

        assert_eq!(encoded["decision"]["mode"], "server_future_mode");
        assert_eq!(encoded["decision"]["reason"], "server_future_reason");
        assert_eq!(
            encoded["decision"]["transcode_plan"]["output_container"],
            "future_container"
        );
        assert!(
            encoded["decision"]["transcode_plan"]
                .get("hardware_acceleration")
                .is_none()
        );

        let browser_ticket =
            serde_json::from_value::<BrowserPlaybackTicketResponse>(serde_json::json!({
                "source_id": "source-1",
                "item_id": null,
                "mode": "future_browser_mode",
                "expires_at": "2026-05-26T12:00:00Z",
                "urls": [{
                    "kind": "future_url_kind",
                    "url": "/playback/future",
                    "content_type": "video/example",
                    "supports_range_requests": true
                }]
            }))
            .unwrap();

        assert_eq!(
            browser_ticket.mode,
            BrowserPlaybackMode::Other("future_browser_mode".to_owned())
        );
        assert_eq!(
            browser_ticket.urls[0].kind,
            BrowserPlaybackUrlKind::Other("future_url_kind".to_owned())
        );
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

    #[test]
    fn public_user_playback_state_hides_principal_identity() {
        let response = UserPlaybackStateResponse {
            state: UserPlaybackStateDto {
                item_id: "item-1".to_owned(),
                source_id: Some("source-1".to_owned()),
                resume_position_ms: Some(120_000),
                duration_ms: Some(600_000),
                progress_percent: Some(0.2),
                watched: false,
                watched_at: None,
                last_played_at: Some("1970-01-01T00:00:10Z".to_owned()),
                updated_at: Some("1970-01-01T00:00:10Z".to_owned()),
                version: 1,
            },
        };

        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["state"]["item_id"], "item-1");
        let progress = value["state"]["progress_percent"].as_f64().unwrap();
        assert!((progress - 0.2).abs() < 0.000_001);
        assert!(value["state"].get("principal_id").is_none());
        assert!(value["state"].get("user_id").is_none());
    }
}
