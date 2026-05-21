#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreHttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CorePlaybackCapabilities {
    pub direct_play: Option<bool>,
    pub containers: Vec<String>,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CorePageQuery {
    pub limit: Option<u32>,
    pub offset: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreBrowsePagedRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub page: Option<CorePageQuery>,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreBrowseEntityRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreBrowseEntityPagedRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub id: String,
    pub page: Option<CorePageQuery>,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreSearchItemsRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub query: Option<String>,
    pub facets: Vec<String>,
    pub page: Option<CorePageQuery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum CorePlaybackMode {
    DirectPlay,
    Remux,
    Transcode,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum CoreOutputContainer {
    Hls,
    Mp4,
    Mkv,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CorePlaybackDecisionSummary {
    pub source_id: String,
    pub mode: CorePlaybackMode,
    pub transcode_output_container: Option<CoreOutputContainer>,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CorePlaybackTarget {
    pub request: CoreHttpRequest,
    pub session_probe_request: Option<CoreHttpRequest>,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreSafeRequestPreview {
    pub method: String,
    pub url: String,
    pub headers: Vec<CoreHttpHeader>,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreHttpRequest {
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<CoreHttpHeader>,
    pub body_utf8: Option<String>,
    pub safe_preview: CoreSafeRequestPreview,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreHttpResponse {
    pub request_id: String,
    pub status_code: i32,
    pub headers: Vec<CoreHttpHeader>,
    pub body_utf8: String,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CorePublicError {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum CoreRuntimeFailureKind {
    MissingAccessToken,
    UnsupportedApiVersion,
    InvalidResponse,
    HttpError,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreRuntimeFailure {
    pub kind: CoreRuntimeFailureKind,
    pub status_code: Option<i32>,
    pub observed_api_version: Option<String>,
    pub public_error: Option<CorePublicError>,
    pub request: Option<CoreSafeRequestPreview>,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreConnectionProbeSuccess {
    pub api_version: String,
    pub health_request: CoreSafeRequestPreview,
    pub auth_probe_request: CoreSafeRequestPreview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum CoreConnectionProbeOutcomeKind {
    NextRequest,
    Success,
    Failure,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreConnectionProbeOutcome {
    pub kind: CoreConnectionProbeOutcomeKind,
    pub next_request: Option<CoreHttpRequest>,
    pub success: Option<CoreConnectionProbeSuccess>,
    pub failure: Option<CoreRuntimeFailure>,
}

#[uniffi::export]
pub fn start_connection_probe(
    base_url: String,
    access_token: String,
) -> CoreConnectionProbeOutcome {
    let input = taru_client_core::CoreConnectionProbeInput::new(base_url, access_token);
    taru_client_core::start_connection_probe(&input).into()
}

#[uniffi::export]
pub fn advance_connection_probe(
    base_url: String,
    access_token: String,
    response: CoreHttpResponse,
) -> CoreConnectionProbeOutcome {
    let input = taru_client_core::CoreConnectionProbeInput::new(base_url, access_token);
    taru_client_core::advance_connection_probe(&input, &response.into()).into()
}

#[uniffi::export]
pub fn build_playback_decision_request(
    base_url: String,
    access_token: String,
    source_id: String,
    capabilities: CorePlaybackCapabilities,
) -> CoreHttpRequest {
    taru_client_core::build_playback_decision_request(
        &taru_client_core::CorePlaybackDecisionRequestInput {
            base_url,
            access_token,
            source_id,
            capabilities: capabilities.into(),
        },
    )
    .into()
}

#[uniffi::export]
pub fn build_recommended_playback_target(
    base_url: String,
    decision: CorePlaybackDecisionSummary,
    capabilities: CorePlaybackCapabilities,
) -> Option<CorePlaybackTarget> {
    taru_client_core::build_recommended_playback_target(
        &taru_client_core::CorePlaybackTargetInput {
            base_url,
            decision: decision.into(),
            capabilities: capabilities.into(),
        },
    )
    .map(Into::into)
}

#[uniffi::export]
pub fn build_direct_playback_target(base_url: String, source_id: String) -> CorePlaybackTarget {
    taru_client_core::build_direct_playback_target(
        &taru_client_core::CoreDirectPlaybackTargetInput {
            base_url,
            source_id,
        },
    )
    .into()
}

#[uniffi::export]
pub fn build_head_direct_playback_target(
    base_url: String,
    source_id: String,
) -> CorePlaybackTarget {
    taru_client_core::build_head_direct_playback_target(
        &taru_client_core::CoreDirectPlaybackTargetInput {
            base_url,
            source_id,
        },
    )
    .into()
}

#[uniffi::export]
pub fn build_remux_playback_target(
    base_url: String,
    source_id: String,
    capabilities: CorePlaybackCapabilities,
    output_container: Option<CoreOutputContainer>,
) -> CorePlaybackTarget {
    taru_client_core::build_remux_playback_target(&taru_client_core::CoreRemuxPlaybackTargetInput {
        base_url,
        source_id,
        capabilities: capabilities.into(),
        output_container: output_container.map(Into::into),
    })
    .into()
}

#[uniffi::export]
pub fn build_hls_playlist_target(
    base_url: String,
    source_id: String,
    capabilities: CorePlaybackCapabilities,
) -> CorePlaybackTarget {
    taru_client_core::build_hls_playlist_target(&taru_client_core::CoreHlsPlaylistTargetInput {
        base_url,
        source_id,
        capabilities: capabilities.into(),
    })
    .into()
}

#[uniffi::export]
pub fn build_hls_segment_request(
    base_url: String,
    session_id: String,
    segment_name: String,
) -> CoreHttpRequest {
    taru_client_core::build_hls_segment_request(&taru_client_core::CorePlaybackSegmentInput {
        base_url,
        session_id,
        segment_name,
    })
    .into()
}

#[uniffi::export]
pub fn build_list_libraries_request(input: CoreBrowsePagedRequestInput) -> CoreHttpRequest {
    taru_client_core::build_list_libraries_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_get_library_request(input: CoreBrowseEntityRequestInput) -> CoreHttpRequest {
    taru_client_core::build_get_library_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_library_sources_request(
    input: CoreBrowseEntityPagedRequestInput,
) -> CoreHttpRequest {
    taru_client_core::build_list_library_sources_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_items_request(input: CoreBrowsePagedRequestInput) -> CoreHttpRequest {
    taru_client_core::build_list_items_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_get_item_request(input: CoreBrowseEntityRequestInput) -> CoreHttpRequest {
    taru_client_core::build_get_item_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_item_images_request(input: CoreBrowseEntityRequestInput) -> CoreHttpRequest {
    taru_client_core::build_list_item_images_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_get_person_request(input: CoreBrowseEntityRequestInput) -> CoreHttpRequest {
    taru_client_core::build_get_person_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_person_items_request(
    input: CoreBrowseEntityPagedRequestInput,
) -> CoreHttpRequest {
    taru_client_core::build_list_person_items_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_genres_request(input: CoreBrowsePagedRequestInput) -> CoreHttpRequest {
    taru_client_core::build_list_genres_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_genre_items_request(input: CoreBrowseEntityPagedRequestInput) -> CoreHttpRequest {
    taru_client_core::build_list_genre_items_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_tags_request(input: CoreBrowsePagedRequestInput) -> CoreHttpRequest {
    taru_client_core::build_list_tags_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_tag_items_request(input: CoreBrowseEntityPagedRequestInput) -> CoreHttpRequest {
    taru_client_core::build_list_tag_items_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_search_items_request(input: CoreSearchItemsRequestInput) -> CoreHttpRequest {
    taru_client_core::build_search_items_request(&input.into()).into()
}

impl From<taru_client_core::CoreHttpHeader> for CoreHttpHeader {
    fn from(value: taru_client_core::CoreHttpHeader) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

impl From<CoreHttpHeader> for taru_client_core::CoreHttpHeader {
    fn from(value: CoreHttpHeader) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

impl From<CorePageQuery> for taru_client_core::CorePageQuery {
    fn from(value: CorePageQuery) -> Self {
        Self {
            limit: value.limit,
            offset: value.offset,
        }
    }
}

impl From<CoreBrowsePagedRequestInput> for taru_client_core::CoreBrowsePagedRequestInput {
    fn from(value: CoreBrowsePagedRequestInput) -> Self {
        Self {
            base_url: value.base_url,
            access_token: value.access_token,
            page: value.page.map(Into::into),
        }
    }
}

impl From<CoreBrowseEntityRequestInput> for taru_client_core::CoreBrowseEntityRequestInput {
    fn from(value: CoreBrowseEntityRequestInput) -> Self {
        Self {
            base_url: value.base_url,
            access_token: value.access_token,
            id: value.id,
        }
    }
}

impl From<CoreBrowseEntityPagedRequestInput>
    for taru_client_core::CoreBrowseEntityPagedRequestInput
{
    fn from(value: CoreBrowseEntityPagedRequestInput) -> Self {
        Self {
            base_url: value.base_url,
            access_token: value.access_token,
            id: value.id,
            page: value.page.map(Into::into),
        }
    }
}

impl From<CoreSearchItemsRequestInput> for taru_client_core::CoreSearchItemsRequestInput {
    fn from(value: CoreSearchItemsRequestInput) -> Self {
        Self {
            base_url: value.base_url,
            access_token: value.access_token,
            query: value.query,
            facets: value.facets,
            page: value.page.map(Into::into),
        }
    }
}

impl From<CorePlaybackCapabilities> for taru_client_core::CorePlaybackCapabilities {
    fn from(value: CorePlaybackCapabilities) -> Self {
        Self {
            direct_play: value.direct_play,
            containers: value.containers,
            video_codecs: value.video_codecs,
            audio_codecs: value.audio_codecs,
        }
    }
}

impl From<taru_client_core::CorePlaybackMode> for CorePlaybackMode {
    fn from(value: taru_client_core::CorePlaybackMode) -> Self {
        match value {
            taru_client_core::CorePlaybackMode::DirectPlay => Self::DirectPlay,
            taru_client_core::CorePlaybackMode::Remux => Self::Remux,
            taru_client_core::CorePlaybackMode::Transcode => Self::Transcode,
            taru_client_core::CorePlaybackMode::Unknown => Self::Unknown,
        }
    }
}

impl From<CorePlaybackMode> for taru_client_core::CorePlaybackMode {
    fn from(value: CorePlaybackMode) -> Self {
        match value {
            CorePlaybackMode::DirectPlay => Self::DirectPlay,
            CorePlaybackMode::Remux => Self::Remux,
            CorePlaybackMode::Transcode => Self::Transcode,
            CorePlaybackMode::Unknown => Self::Unknown,
        }
    }
}

impl From<taru_client_core::CoreOutputContainer> for CoreOutputContainer {
    fn from(value: taru_client_core::CoreOutputContainer) -> Self {
        match value {
            taru_client_core::CoreOutputContainer::Hls => Self::Hls,
            taru_client_core::CoreOutputContainer::Mp4 => Self::Mp4,
            taru_client_core::CoreOutputContainer::Mkv => Self::Mkv,
            taru_client_core::CoreOutputContainer::Unknown => Self::Unknown,
        }
    }
}

impl From<CoreOutputContainer> for taru_client_core::CoreOutputContainer {
    fn from(value: CoreOutputContainer) -> Self {
        match value {
            CoreOutputContainer::Hls => Self::Hls,
            CoreOutputContainer::Mp4 => Self::Mp4,
            CoreOutputContainer::Mkv => Self::Mkv,
            CoreOutputContainer::Unknown => Self::Unknown,
        }
    }
}

impl From<CorePlaybackDecisionSummary> for taru_client_core::CorePlaybackDecisionSummary {
    fn from(value: CorePlaybackDecisionSummary) -> Self {
        Self {
            source_id: value.source_id,
            mode: value.mode.into(),
            transcode_output_container: value.transcode_output_container.map(Into::into),
        }
    }
}

impl From<taru_client_core::CorePlaybackTarget> for CorePlaybackTarget {
    fn from(value: taru_client_core::CorePlaybackTarget) -> Self {
        Self {
            request: value.request.into(),
            session_probe_request: value.session_probe_request.map(Into::into),
        }
    }
}

impl From<taru_client_core::CoreSafeRequestPreview> for CoreSafeRequestPreview {
    fn from(value: taru_client_core::CoreSafeRequestPreview) -> Self {
        Self {
            method: value.method,
            url: value.url,
            headers: value.headers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<CoreSafeRequestPreview> for taru_client_core::CoreSafeRequestPreview {
    fn from(value: CoreSafeRequestPreview) -> Self {
        Self {
            method: value.method,
            url: value.url,
            headers: value.headers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<taru_client_core::CoreHttpRequest> for CoreHttpRequest {
    fn from(value: taru_client_core::CoreHttpRequest) -> Self {
        Self {
            request_id: value.request_id,
            method: value.method,
            url: value.url,
            headers: value.headers.into_iter().map(Into::into).collect(),
            body_utf8: value.body_utf8,
            safe_preview: value.safe_preview.into(),
        }
    }
}

impl From<CoreHttpResponse> for taru_client_core::CoreHttpResponse {
    fn from(value: CoreHttpResponse) -> Self {
        Self {
            request_id: value.request_id,
            status_code: value.status_code,
            headers: value.headers.into_iter().map(Into::into).collect(),
            body_utf8: value.body_utf8,
        }
    }
}

impl From<taru_client_core::CorePublicError> for CorePublicError {
    fn from(value: taru_client_core::CorePublicError) -> Self {
        Self {
            code: value.code,
            message: value.message,
        }
    }
}

impl From<taru_client_core::CoreRuntimeFailureKind> for CoreRuntimeFailureKind {
    fn from(value: taru_client_core::CoreRuntimeFailureKind) -> Self {
        match value {
            taru_client_core::CoreRuntimeFailureKind::MissingAccessToken => {
                Self::MissingAccessToken
            }
            taru_client_core::CoreRuntimeFailureKind::UnsupportedApiVersion => {
                Self::UnsupportedApiVersion
            }
            taru_client_core::CoreRuntimeFailureKind::InvalidResponse => Self::InvalidResponse,
            taru_client_core::CoreRuntimeFailureKind::HttpError => Self::HttpError,
        }
    }
}

impl From<taru_client_core::CoreRuntimeFailure> for CoreRuntimeFailure {
    fn from(value: taru_client_core::CoreRuntimeFailure) -> Self {
        Self {
            kind: value.kind.into(),
            status_code: value.status_code,
            observed_api_version: value.observed_api_version,
            public_error: value.public_error.map(Into::into),
            request: value.request.map(Into::into),
        }
    }
}

impl From<taru_client_core::CoreConnectionProbeSuccess> for CoreConnectionProbeSuccess {
    fn from(value: taru_client_core::CoreConnectionProbeSuccess) -> Self {
        Self {
            api_version: value.api_version,
            health_request: value.health_request.into(),
            auth_probe_request: value.auth_probe_request.into(),
        }
    }
}

impl From<taru_client_core::CoreConnectionProbeOutcomeKind> for CoreConnectionProbeOutcomeKind {
    fn from(value: taru_client_core::CoreConnectionProbeOutcomeKind) -> Self {
        match value {
            taru_client_core::CoreConnectionProbeOutcomeKind::NextRequest => Self::NextRequest,
            taru_client_core::CoreConnectionProbeOutcomeKind::Success => Self::Success,
            taru_client_core::CoreConnectionProbeOutcomeKind::Failure => Self::Failure,
        }
    }
}

impl From<taru_client_core::CoreConnectionProbeOutcome> for CoreConnectionProbeOutcome {
    fn from(value: taru_client_core::CoreConnectionProbeOutcome) -> Self {
        Self {
            kind: value.kind.into(),
            next_request: value.next_request.map(Into::into),
            success: value.success.map(Into::into),
            failure: value.failure.map(Into::into),
        }
    }
}

uniffi::setup_scaffolding!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniffi_surface_exposes_core_connection_start() {
        let outcome = start_connection_probe(
            "https://taru.example/api".to_owned(),
            "secret-token".to_owned(),
        );

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::NextRequest);
        let request = outcome.next_request.unwrap();
        assert_eq!(
            request.request_id,
            taru_client_core::CONNECTION_HEALTH_REQUEST_ID
        );
        assert_eq!(request.url, "https://taru.example/api/health");
    }

    #[test]
    fn uniffi_surface_exposes_playback_target_builder() {
        let target = build_recommended_playback_target(
            "https://taru.example/api".to_owned(),
            CorePlaybackDecisionSummary {
                source_id: "source 1".to_owned(),
                mode: CorePlaybackMode::Remux,
                transcode_output_container: Some(CoreOutputContainer::Mkv),
            },
            CorePlaybackCapabilities {
                direct_play: None,
                containers: Vec::new(),
                video_codecs: Vec::new(),
                audio_codecs: Vec::new(),
            },
        )
        .unwrap();

        assert_eq!(
            target.request.request_id,
            taru_client_core::PLAYBACK_REMUX_STREAM_REQUEST_ID
        );
        assert!(target.session_probe_request.is_some());
        assert_eq!(
            target.request.url,
            "https://taru.example/api/sources/source%201/stream/remux?output_container=mkv"
        );

        let explicit = build_remux_playback_target(
            "https://taru.example/api".to_owned(),
            "source 1".to_owned(),
            CorePlaybackCapabilities {
                direct_play: None,
                containers: Vec::new(),
                video_codecs: Vec::new(),
                audio_codecs: Vec::new(),
            },
            Some(CoreOutputContainer::Hls),
        );
        assert_eq!(
            explicit.request.url,
            "https://taru.example/api/sources/source%201/stream/remux"
        );
    }

    #[test]
    fn uniffi_surface_exposes_browse_request_builders() {
        let libraries = build_list_libraries_request(CoreBrowsePagedRequestInput {
            base_url: "https://taru.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            page: Some(CorePageQuery {
                limit: Some(25),
                offset: Some(50),
            }),
        });
        assert_eq!(
            libraries.url,
            "https://taru.example/api/libraries?limit=25&offset=50"
        );
        assert_eq!(
            libraries.safe_preview.headers,
            vec![CoreHttpHeader {
                name: "Authorization".to_owned(),
                value: "Bearer <redacted>".to_owned(),
            }]
        );

        let search = build_search_items_request(CoreSearchItemsRequestInput {
            base_url: "https://taru.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            query: Some("route demo".to_owned()),
            facets: vec!["genre:test".to_owned(), "tag:favorite".to_owned()],
            page: Some(CorePageQuery {
                limit: Some(12),
                offset: Some(6),
            }),
        });
        assert_eq!(
            search.url,
            "https://taru.example/api/search?q=route%20demo&facet=genre%3Atest%2Ctag%3Afavorite&limit=12&offset=6"
        );

        let tag_items = build_list_tag_items_request(CoreBrowseEntityPagedRequestInput {
            base_url: "https://taru.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            id: "tag:favorite".to_owned(),
            page: None,
        });
        assert_eq!(
            tag_items.url,
            "https://taru.example/api/tags/tag%3Afavorite/items"
        );
    }
}
