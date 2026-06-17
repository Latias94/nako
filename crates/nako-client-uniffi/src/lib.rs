#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreHttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CorePlaybackCapabilities {
    pub direct_play: Option<bool>,
    pub device_family: Option<String>,
    pub profile_version: Option<u32>,
    pub containers: Vec<String>,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
    pub max_video_bitrate: Option<u64>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub max_audio_channels: Option<u32>,
    pub supports_hdr: Option<bool>,
    pub supports_subtitles: Option<bool>,
    pub hls_variant_policy: Option<CoreHlsVariantPolicy>,
    pub hls_segment_container: Option<CoreHlsSegmentContainer>,
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

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreArtworkImageRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub image_id: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreUserPlaybackPagedRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub page: Option<CorePageQuery>,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreUserPlaybackItemRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub item_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreUserPlaybackItemWriteRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub item_id: String,
    pub body_utf8: String,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum CoreHlsVariantPolicy {
    SingleVariant,
    Adaptive,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum CoreHlsSegmentContainer {
    MpegTs,
    Fmp4,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CorePlaybackDecisionSummary {
    pub source_id: String,
    pub mode: CorePlaybackMode,
    pub transcode_output_container: Option<CoreOutputContainer>,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CorePlaybackSourceRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub source_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CorePlaybackSessionRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub session_id: String,
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
    let input = nako_client_core::CoreConnectionProbeInput::new(base_url, access_token);
    nako_client_core::start_connection_probe(&input).into()
}

#[uniffi::export]
pub fn advance_connection_probe(
    base_url: String,
    access_token: String,
    response: CoreHttpResponse,
) -> CoreConnectionProbeOutcome {
    let input = nako_client_core::CoreConnectionProbeInput::new(base_url, access_token);
    nako_client_core::advance_connection_probe(&input, &response.into()).into()
}

#[uniffi::export]
pub fn build_playback_decision_request(
    base_url: String,
    access_token: String,
    source_id: String,
    capabilities: CorePlaybackCapabilities,
) -> CoreHttpRequest {
    nako_client_core::build_playback_decision_request(
        &nako_client_core::CorePlaybackDecisionRequestInput {
            base_url,
            access_token,
            source_id,
            selection: nako_client_core::CorePlaybackSelection::empty(),
            capabilities: capabilities.into(),
        },
    )
    .into()
}

#[uniffi::export]
pub fn build_playback_decision_request_with_profile(
    base_url: String,
    access_token: String,
    source_id: String,
    playback_profile_id: String,
    capabilities: CorePlaybackCapabilities,
) -> CoreHttpRequest {
    nako_client_core::build_playback_decision_request(
        &nako_client_core::CorePlaybackDecisionRequestInput {
            base_url,
            access_token,
            source_id,
            selection: nako_client_core::CorePlaybackSelection::from_playback_profile_id(
                playback_profile_id,
            ),
            capabilities: capabilities.into(),
        },
    )
    .into()
}

#[uniffi::export]
pub fn build_source_probe_request(input: CorePlaybackSourceRequestInput) -> CoreHttpRequest {
    nako_client_core::build_source_probe_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_recommended_playback_target(
    base_url: String,
    decision: CorePlaybackDecisionSummary,
    capabilities: CorePlaybackCapabilities,
) -> Option<CorePlaybackTarget> {
    nako_client_core::build_recommended_playback_target(
        &nako_client_core::CorePlaybackTargetInput {
            base_url,
            decision: decision.into(),
            selection: nako_client_core::CorePlaybackSelection::empty(),
            capabilities: capabilities.into(),
        },
    )
    .map(Into::into)
}

#[uniffi::export]
pub fn build_direct_playback_target(base_url: String, source_id: String) -> CorePlaybackTarget {
    nako_client_core::build_direct_playback_target(
        &nako_client_core::CoreDirectPlaybackTargetInput {
            base_url,
            source_id,
            selection: nako_client_core::CorePlaybackSelection::empty(),
        },
    )
    .into()
}

#[uniffi::export]
pub fn build_direct_playback_target_with_profile(
    base_url: String,
    source_id: String,
    playback_profile_id: String,
) -> CorePlaybackTarget {
    nako_client_core::build_direct_playback_target(
        &nako_client_core::CoreDirectPlaybackTargetInput {
            base_url,
            source_id,
            selection: nako_client_core::CorePlaybackSelection::from_playback_profile_id(
                playback_profile_id,
            ),
        },
    )
    .into()
}

#[uniffi::export]
pub fn build_head_direct_playback_target(
    base_url: String,
    source_id: String,
) -> CorePlaybackTarget {
    nako_client_core::build_head_direct_playback_target(
        &nako_client_core::CoreDirectPlaybackTargetInput {
            base_url,
            source_id,
            selection: nako_client_core::CorePlaybackSelection::empty(),
        },
    )
    .into()
}

#[uniffi::export]
pub fn build_head_direct_playback_target_with_profile(
    base_url: String,
    source_id: String,
    playback_profile_id: String,
) -> CorePlaybackTarget {
    nako_client_core::build_head_direct_playback_target(
        &nako_client_core::CoreDirectPlaybackTargetInput {
            base_url,
            source_id,
            selection: nako_client_core::CorePlaybackSelection::from_playback_profile_id(
                playback_profile_id,
            ),
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
    nako_client_core::build_remux_playback_target(&nako_client_core::CoreRemuxPlaybackTargetInput {
        base_url,
        source_id,
        selection: nako_client_core::CorePlaybackSelection::empty(),
        capabilities: capabilities.into(),
        output_container: output_container.map(Into::into),
    })
    .into()
}

#[uniffi::export]
pub fn build_remux_playback_target_with_profile(
    base_url: String,
    source_id: String,
    playback_profile_id: String,
    capabilities: CorePlaybackCapabilities,
    output_container: Option<CoreOutputContainer>,
) -> CorePlaybackTarget {
    nako_client_core::build_remux_playback_target(&nako_client_core::CoreRemuxPlaybackTargetInput {
        base_url,
        source_id,
        selection: nako_client_core::CorePlaybackSelection::from_playback_profile_id(
            playback_profile_id,
        ),
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
    nako_client_core::build_hls_playlist_target(&nako_client_core::CoreHlsPlaylistTargetInput {
        base_url,
        source_id,
        selection: nako_client_core::CorePlaybackSelection::empty(),
        capabilities: capabilities.into(),
    })
    .into()
}

#[uniffi::export]
pub fn build_hls_playlist_target_with_profile(
    base_url: String,
    source_id: String,
    playback_profile_id: String,
    capabilities: CorePlaybackCapabilities,
) -> CorePlaybackTarget {
    nako_client_core::build_hls_playlist_target(&nako_client_core::CoreHlsPlaylistTargetInput {
        base_url,
        source_id,
        selection: nako_client_core::CorePlaybackSelection::from_playback_profile_id(
            playback_profile_id,
        ),
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
    nako_client_core::build_hls_segment_request(&nako_client_core::CorePlaybackSegmentInput {
        base_url,
        session_id,
        segment_name,
    })
    .into()
}

#[uniffi::export]
pub fn build_get_playback_session_request(
    input: CorePlaybackSessionRequestInput,
) -> CoreHttpRequest {
    nako_client_core::build_get_playback_session_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_cancel_playback_session_request(
    input: CorePlaybackSessionRequestInput,
) -> CoreHttpRequest {
    nako_client_core::build_cancel_playback_session_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_libraries_request(input: CoreBrowsePagedRequestInput) -> CoreHttpRequest {
    nako_client_core::build_list_libraries_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_get_library_request(input: CoreBrowseEntityRequestInput) -> CoreHttpRequest {
    nako_client_core::build_get_library_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_library_sources_request(
    input: CoreBrowseEntityPagedRequestInput,
) -> CoreHttpRequest {
    nako_client_core::build_list_library_sources_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_items_request(input: CoreBrowsePagedRequestInput) -> CoreHttpRequest {
    nako_client_core::build_list_items_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_get_item_request(input: CoreBrowseEntityRequestInput) -> CoreHttpRequest {
    nako_client_core::build_get_item_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_item_images_request(input: CoreBrowseEntityRequestInput) -> CoreHttpRequest {
    nako_client_core::build_list_item_images_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_get_person_request(input: CoreBrowseEntityRequestInput) -> CoreHttpRequest {
    nako_client_core::build_get_person_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_person_items_request(
    input: CoreBrowseEntityPagedRequestInput,
) -> CoreHttpRequest {
    nako_client_core::build_list_person_items_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_genres_request(input: CoreBrowsePagedRequestInput) -> CoreHttpRequest {
    nako_client_core::build_list_genres_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_genre_items_request(input: CoreBrowseEntityPagedRequestInput) -> CoreHttpRequest {
    nako_client_core::build_list_genre_items_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_tags_request(input: CoreBrowsePagedRequestInput) -> CoreHttpRequest {
    nako_client_core::build_list_tags_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_tag_items_request(input: CoreBrowseEntityPagedRequestInput) -> CoreHttpRequest {
    nako_client_core::build_list_tag_items_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_search_items_request(input: CoreSearchItemsRequestInput) -> CoreHttpRequest {
    nako_client_core::build_search_items_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_artwork_image_request(input: CoreArtworkImageRequestInput) -> CoreHttpRequest {
    nako_client_core::build_artwork_image_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_get_user_playback_state_request(
    input: CoreUserPlaybackItemRequestInput,
) -> CoreHttpRequest {
    nako_client_core::build_get_user_playback_state_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_list_continue_watching_request(
    input: CoreUserPlaybackPagedRequestInput,
) -> CoreHttpRequest {
    nako_client_core::build_list_continue_watching_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_update_user_playback_progress_request(
    input: CoreUserPlaybackItemWriteRequestInput,
) -> CoreHttpRequest {
    nako_client_core::build_update_user_playback_progress_request(&input.into()).into()
}

#[uniffi::export]
pub fn build_set_user_watched_state_request(
    input: CoreUserPlaybackItemWriteRequestInput,
) -> CoreHttpRequest {
    nako_client_core::build_set_user_watched_state_request(&input.into()).into()
}

impl From<nako_client_core::CoreHttpHeader> for CoreHttpHeader {
    fn from(value: nako_client_core::CoreHttpHeader) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

impl From<CoreHttpHeader> for nako_client_core::CoreHttpHeader {
    fn from(value: CoreHttpHeader) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

impl From<CorePageQuery> for nako_client_core::CorePageQuery {
    fn from(value: CorePageQuery) -> Self {
        Self {
            limit: value.limit,
            offset: value.offset,
        }
    }
}

impl From<CoreBrowsePagedRequestInput> for nako_client_core::CoreBrowsePagedRequestInput {
    fn from(value: CoreBrowsePagedRequestInput) -> Self {
        Self {
            base_url: value.base_url,
            access_token: value.access_token,
            page: value.page.map(Into::into),
        }
    }
}

impl From<CoreBrowseEntityRequestInput> for nako_client_core::CoreBrowseEntityRequestInput {
    fn from(value: CoreBrowseEntityRequestInput) -> Self {
        Self {
            base_url: value.base_url,
            access_token: value.access_token,
            id: value.id,
        }
    }
}

impl From<CoreBrowseEntityPagedRequestInput>
    for nako_client_core::CoreBrowseEntityPagedRequestInput
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

impl From<CoreSearchItemsRequestInput> for nako_client_core::CoreSearchItemsRequestInput {
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

impl From<CoreArtworkImageRequestInput> for nako_client_core::CoreArtworkImageRequestInput {
    fn from(value: CoreArtworkImageRequestInput) -> Self {
        Self {
            base_url: value.base_url,
            access_token: value.access_token,
            image_id: value.image_id,
            width: value.width,
            height: value.height,
        }
    }
}

impl From<CoreUserPlaybackPagedRequestInput>
    for nako_client_core::CoreUserPlaybackPagedRequestInput
{
    fn from(value: CoreUserPlaybackPagedRequestInput) -> Self {
        Self {
            base_url: value.base_url,
            access_token: value.access_token,
            page: value.page.map(Into::into),
        }
    }
}

impl From<CoreUserPlaybackItemRequestInput> for nako_client_core::CoreUserPlaybackItemRequestInput {
    fn from(value: CoreUserPlaybackItemRequestInput) -> Self {
        Self {
            base_url: value.base_url,
            access_token: value.access_token,
            item_id: value.item_id,
        }
    }
}

impl From<CoreUserPlaybackItemWriteRequestInput>
    for nako_client_core::CoreUserPlaybackItemWriteRequestInput
{
    fn from(value: CoreUserPlaybackItemWriteRequestInput) -> Self {
        Self {
            base_url: value.base_url,
            access_token: value.access_token,
            item_id: value.item_id,
            body_utf8: value.body_utf8,
        }
    }
}

impl From<CorePlaybackCapabilities> for nako_client_core::CorePlaybackCapabilities {
    fn from(value: CorePlaybackCapabilities) -> Self {
        Self {
            direct_play: value.direct_play,
            device_family: value.device_family,
            profile_version: value.profile_version,
            containers: value.containers,
            video_codecs: value.video_codecs,
            audio_codecs: value.audio_codecs,
            max_video_bitrate: value.max_video_bitrate,
            max_width: value.max_width,
            max_height: value.max_height,
            max_audio_channels: value.max_audio_channels,
            supports_hdr: value.supports_hdr,
            supports_subtitles: value.supports_subtitles,
            hls_variant_policy: value.hls_variant_policy.map(Into::into),
            hls_segment_container: value.hls_segment_container.map(Into::into),
        }
    }
}

impl From<nako_client_core::CorePlaybackMode> for CorePlaybackMode {
    fn from(value: nako_client_core::CorePlaybackMode) -> Self {
        match value {
            nako_client_core::CorePlaybackMode::DirectPlay => Self::DirectPlay,
            nako_client_core::CorePlaybackMode::Remux => Self::Remux,
            nako_client_core::CorePlaybackMode::Transcode => Self::Transcode,
            nako_client_core::CorePlaybackMode::Unknown => Self::Unknown,
        }
    }
}

impl From<CorePlaybackMode> for nako_client_core::CorePlaybackMode {
    fn from(value: CorePlaybackMode) -> Self {
        match value {
            CorePlaybackMode::DirectPlay => Self::DirectPlay,
            CorePlaybackMode::Remux => Self::Remux,
            CorePlaybackMode::Transcode => Self::Transcode,
            CorePlaybackMode::Unknown => Self::Unknown,
        }
    }
}

impl From<nako_client_core::CoreOutputContainer> for CoreOutputContainer {
    fn from(value: nako_client_core::CoreOutputContainer) -> Self {
        match value {
            nako_client_core::CoreOutputContainer::Hls => Self::Hls,
            nako_client_core::CoreOutputContainer::Mp4 => Self::Mp4,
            nako_client_core::CoreOutputContainer::Mkv => Self::Mkv,
            nako_client_core::CoreOutputContainer::Unknown => Self::Unknown,
        }
    }
}

impl From<CoreOutputContainer> for nako_client_core::CoreOutputContainer {
    fn from(value: CoreOutputContainer) -> Self {
        match value {
            CoreOutputContainer::Hls => Self::Hls,
            CoreOutputContainer::Mp4 => Self::Mp4,
            CoreOutputContainer::Mkv => Self::Mkv,
            CoreOutputContainer::Unknown => Self::Unknown,
        }
    }
}

impl From<CoreHlsVariantPolicy> for nako_client_core::CoreHlsVariantPolicy {
    fn from(value: CoreHlsVariantPolicy) -> Self {
        match value {
            CoreHlsVariantPolicy::SingleVariant => Self::SingleVariant,
            CoreHlsVariantPolicy::Adaptive => Self::Adaptive,
            CoreHlsVariantPolicy::Unknown => Self::Unknown,
        }
    }
}

impl From<CoreHlsSegmentContainer> for nako_client_core::CoreHlsSegmentContainer {
    fn from(value: CoreHlsSegmentContainer) -> Self {
        match value {
            CoreHlsSegmentContainer::MpegTs => Self::MpegTs,
            CoreHlsSegmentContainer::Fmp4 => Self::Fmp4,
            CoreHlsSegmentContainer::Unknown => Self::Unknown,
        }
    }
}

impl From<CorePlaybackDecisionSummary> for nako_client_core::CorePlaybackDecisionSummary {
    fn from(value: CorePlaybackDecisionSummary) -> Self {
        Self {
            source_id: value.source_id,
            mode: value.mode.into(),
            transcode_output_container: value.transcode_output_container.map(Into::into),
        }
    }
}

impl From<CorePlaybackSourceRequestInput> for nako_client_core::CorePlaybackSourceRequestInput {
    fn from(value: CorePlaybackSourceRequestInput) -> Self {
        Self {
            base_url: value.base_url,
            access_token: value.access_token,
            source_id: value.source_id,
        }
    }
}

impl From<CorePlaybackSessionRequestInput> for nako_client_core::CorePlaybackSessionRequestInput {
    fn from(value: CorePlaybackSessionRequestInput) -> Self {
        Self {
            base_url: value.base_url,
            access_token: value.access_token,
            session_id: value.session_id,
        }
    }
}

impl From<nako_client_core::CorePlaybackTarget> for CorePlaybackTarget {
    fn from(value: nako_client_core::CorePlaybackTarget) -> Self {
        Self {
            request: value.request.into(),
            session_probe_request: value.session_probe_request.map(Into::into),
        }
    }
}

impl From<nako_client_core::CoreSafeRequestPreview> for CoreSafeRequestPreview {
    fn from(value: nako_client_core::CoreSafeRequestPreview) -> Self {
        Self {
            method: value.method,
            url: value.url,
            headers: value.headers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<CoreSafeRequestPreview> for nako_client_core::CoreSafeRequestPreview {
    fn from(value: CoreSafeRequestPreview) -> Self {
        Self {
            method: value.method,
            url: value.url,
            headers: value.headers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<nako_client_core::CoreHttpRequest> for CoreHttpRequest {
    fn from(value: nako_client_core::CoreHttpRequest) -> Self {
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

impl From<CoreHttpResponse> for nako_client_core::CoreHttpResponse {
    fn from(value: CoreHttpResponse) -> Self {
        Self {
            request_id: value.request_id,
            status_code: value.status_code,
            headers: value.headers.into_iter().map(Into::into).collect(),
            body_utf8: value.body_utf8,
        }
    }
}

impl From<nako_client_core::CorePublicError> for CorePublicError {
    fn from(value: nako_client_core::CorePublicError) -> Self {
        Self {
            code: value.code,
            message: value.message,
        }
    }
}

impl From<nako_client_core::CoreRuntimeFailureKind> for CoreRuntimeFailureKind {
    fn from(value: nako_client_core::CoreRuntimeFailureKind) -> Self {
        match value {
            nako_client_core::CoreRuntimeFailureKind::MissingAccessToken => {
                Self::MissingAccessToken
            }
            nako_client_core::CoreRuntimeFailureKind::UnsupportedApiVersion => {
                Self::UnsupportedApiVersion
            }
            nako_client_core::CoreRuntimeFailureKind::InvalidResponse => Self::InvalidResponse,
            nako_client_core::CoreRuntimeFailureKind::HttpError => Self::HttpError,
        }
    }
}

impl From<nako_client_core::CoreRuntimeFailure> for CoreRuntimeFailure {
    fn from(value: nako_client_core::CoreRuntimeFailure) -> Self {
        Self {
            kind: value.kind.into(),
            status_code: value.status_code,
            observed_api_version: value.observed_api_version,
            public_error: value.public_error.map(Into::into),
            request: value.request.map(Into::into),
        }
    }
}

impl From<nako_client_core::CoreConnectionProbeSuccess> for CoreConnectionProbeSuccess {
    fn from(value: nako_client_core::CoreConnectionProbeSuccess) -> Self {
        Self {
            api_version: value.api_version,
            health_request: value.health_request.into(),
            auth_probe_request: value.auth_probe_request.into(),
        }
    }
}

impl From<nako_client_core::CoreConnectionProbeOutcomeKind> for CoreConnectionProbeOutcomeKind {
    fn from(value: nako_client_core::CoreConnectionProbeOutcomeKind) -> Self {
        match value {
            nako_client_core::CoreConnectionProbeOutcomeKind::NextRequest => Self::NextRequest,
            nako_client_core::CoreConnectionProbeOutcomeKind::Success => Self::Success,
            nako_client_core::CoreConnectionProbeOutcomeKind::Failure => Self::Failure,
        }
    }
}

impl From<nako_client_core::CoreConnectionProbeOutcome> for CoreConnectionProbeOutcome {
    fn from(value: nako_client_core::CoreConnectionProbeOutcome) -> Self {
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
            "https://nako.example/api".to_owned(),
            "secret-token".to_owned(),
        );

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::NextRequest);
        let request = outcome.next_request.unwrap();
        assert_eq!(
            request.request_id,
            nako_client_core::CONNECTION_HEALTH_REQUEST_ID
        );
        assert_eq!(request.url, "https://nako.example/api/health");
    }

    #[test]
    fn uniffi_surface_exposes_playback_target_builder() {
        let target = build_recommended_playback_target(
            "https://nako.example/api".to_owned(),
            CorePlaybackDecisionSummary {
                source_id: "source 1".to_owned(),
                mode: CorePlaybackMode::Remux,
                transcode_output_container: Some(CoreOutputContainer::Mkv),
            },
            CorePlaybackCapabilities {
                direct_play: None,
                device_family: None,
                profile_version: None,
                containers: Vec::new(),
                video_codecs: Vec::new(),
                audio_codecs: Vec::new(),
                max_video_bitrate: None,
                max_width: None,
                max_height: None,
                max_audio_channels: None,
                supports_hdr: None,
                supports_subtitles: None,
                hls_variant_policy: None,
                hls_segment_container: None,
            },
        )
        .unwrap();

        assert_eq!(
            target.request.request_id,
            nako_client_core::PLAYBACK_REMUX_STREAM_REQUEST_ID
        );
        assert!(target.session_probe_request.is_some());
        assert_eq!(
            target.request.url,
            "https://nako.example/api/sources/source%201/stream/remux?output_container=mkv"
        );

        let explicit = build_remux_playback_target(
            "https://nako.example/api".to_owned(),
            "source 1".to_owned(),
            CorePlaybackCapabilities {
                direct_play: None,
                device_family: None,
                profile_version: None,
                containers: Vec::new(),
                video_codecs: Vec::new(),
                audio_codecs: Vec::new(),
                max_video_bitrate: None,
                max_width: None,
                max_height: None,
                max_audio_channels: None,
                supports_hdr: None,
                supports_subtitles: None,
                hls_variant_policy: None,
                hls_segment_container: None,
            },
            Some(CoreOutputContainer::Hls),
        );
        assert_eq!(
            explicit.request.url,
            "https://nako.example/api/sources/source%201/stream/remux"
        );
    }

    #[test]
    fn uniffi_surface_preserves_full_playback_capability_query_fields() {
        let request = build_playback_decision_request(
            "https://nako.example/api".to_owned(),
            "secret-token".to_owned(),
            "source 1".to_owned(),
            CorePlaybackCapabilities {
                direct_play: Some(false),
                device_family: Some("browser_chromium".to_owned()),
                profile_version: Some(1),
                containers: vec!["mp4".to_owned(), "webm".to_owned()],
                video_codecs: vec!["h264".to_owned()],
                audio_codecs: vec!["aac".to_owned()],
                max_video_bitrate: Some(8_000_000),
                max_width: Some(1920),
                max_height: Some(1080),
                max_audio_channels: Some(2),
                supports_hdr: Some(false),
                supports_subtitles: Some(true),
                hls_variant_policy: Some(CoreHlsVariantPolicy::Adaptive),
                hls_segment_container: Some(CoreHlsSegmentContainer::Fmp4),
            },
        );

        assert_eq!(
            request.url,
            "https://nako.example/api/sources/source%201/playback/decision?direct_play=false&device_family=browser_chromium&profile_version=1&container=mp4%2Cwebm&video_codec=h264&audio_codec=aac&max_video_bitrate=8000000&max_width=1920&max_height=1080&max_audio_channels=2&supports_hdr=false&supports_subtitles=true&hls_variant_policy=adaptive&hls_segment_container=fmp4"
        );
    }

    #[test]
    fn uniffi_surface_exposes_residual_playback_request_builders() {
        let source_probe = build_source_probe_request(CorePlaybackSourceRequestInput {
            base_url: "https://nako.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            source_id: "source 1".to_owned(),
        });
        assert_eq!(
            source_probe.request_id,
            nako_client_core::PLAYBACK_SOURCE_PROBE_REQUEST_ID
        );
        assert_eq!(
            source_probe.url,
            "https://nako.example/api/sources/source%201/probe"
        );
        assert_eq!(
            source_probe.safe_preview.headers,
            vec![CoreHttpHeader {
                name: "Authorization".to_owned(),
                value: "Bearer <redacted>".to_owned(),
            }]
        );

        let session = build_get_playback_session_request(CorePlaybackSessionRequestInput {
            base_url: "https://nako.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            session_id: "session 1".to_owned(),
        });
        assert_eq!(
            session.url,
            "https://nako.example/api/playback/sessions/session%201"
        );
        assert_eq!(session.method, "GET");

        let cancel = build_cancel_playback_session_request(CorePlaybackSessionRequestInput {
            base_url: "https://nako.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            session_id: "session/1".to_owned(),
        });
        assert_eq!(cancel.method, "POST");
        assert_eq!(
            cancel.url,
            "https://nako.example/api/playback/sessions/session%2F1/cancel"
        );
    }

    #[test]
    fn uniffi_surface_exposes_browse_request_builders() {
        let libraries = build_list_libraries_request(CoreBrowsePagedRequestInput {
            base_url: "https://nako.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            page: Some(CorePageQuery {
                limit: Some(25),
                offset: Some(50),
            }),
        });
        assert_eq!(
            libraries.url,
            "https://nako.example/api/libraries?limit=25&offset=50"
        );
        assert_eq!(
            libraries.safe_preview.headers,
            vec![CoreHttpHeader {
                name: "Authorization".to_owned(),
                value: "Bearer <redacted>".to_owned(),
            }]
        );

        let search = build_search_items_request(CoreSearchItemsRequestInput {
            base_url: "https://nako.example/api".to_owned(),
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
            "https://nako.example/api/search?q=route%20demo&facet=genre%3Atest%2Ctag%3Afavorite&limit=12&offset=6"
        );

        let tag_items = build_list_tag_items_request(CoreBrowseEntityPagedRequestInput {
            base_url: "https://nako.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            id: "tag:favorite".to_owned(),
            page: None,
        });
        assert_eq!(
            tag_items.url,
            "https://nako.example/api/tags/tag%3Afavorite/items"
        );
    }

    #[test]
    fn uniffi_surface_exposes_artwork_image_request_builder() {
        let request = build_artwork_image_request(CoreArtworkImageRequestInput {
            base_url: "https://nako.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            image_id: "poster 1".to_owned(),
            width: Some(320),
            height: Some(180),
        });

        assert_eq!(
            request.request_id,
            nako_client_core::ARTWORK_IMAGE_REQUEST_ID
        );
        assert_eq!(
            request.url,
            "https://nako.example/api/images/poster%201?width=320&height=180"
        );
        assert_eq!(
            request.safe_preview.headers,
            vec![CoreHttpHeader {
                name: "Authorization".to_owned(),
                value: "Bearer <redacted>".to_owned(),
            }]
        );
    }

    #[test]
    fn uniffi_surface_exposes_user_playback_request_builders() {
        let continue_watching =
            build_list_continue_watching_request(CoreUserPlaybackPagedRequestInput {
                base_url: "https://nako.example/api".to_owned(),
                access_token: "secret-token".to_owned(),
                page: Some(CorePageQuery {
                    limit: Some(12),
                    offset: Some(24),
                }),
            });
        assert_eq!(
            continue_watching.url,
            "https://nako.example/api/users/me/playback-state/continue-watching?limit=12&offset=24"
        );
        assert_eq!(
            continue_watching.safe_preview.headers,
            vec![CoreHttpHeader {
                name: "Authorization".to_owned(),
                value: "Bearer <redacted>".to_owned(),
            }]
        );

        let progress =
            build_update_user_playback_progress_request(CoreUserPlaybackItemWriteRequestInput {
                base_url: "https://nako.example/api".to_owned(),
                access_token: "secret-token".to_owned(),
                item_id: "item 1".to_owned(),
                body_utf8: r#"{"position_ms":123000}"#.to_owned(),
            });
        assert_eq!(progress.request_id, "user_playback.progress");
        assert_eq!(progress.method, "PUT");
        assert_eq!(
            progress.url,
            "https://nako.example/api/users/me/playback-state/items/item%201/progress"
        );
        assert_eq!(
            progress.headers,
            vec![
                CoreHttpHeader {
                    name: "Authorization".to_owned(),
                    value: "Bearer secret-token".to_owned(),
                },
                CoreHttpHeader {
                    name: "Content-Type".to_owned(),
                    value: "application/json".to_owned(),
                },
            ]
        );
        assert_eq!(
            progress.body_utf8.as_deref(),
            Some(r#"{"position_ms":123000}"#)
        );

        let watched = build_set_user_watched_state_request(CoreUserPlaybackItemWriteRequestInput {
            base_url: "https://nako.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            item_id: "item/1".to_owned(),
            body_utf8: r#"{"watched":true}"#.to_owned(),
        });
        assert_eq!(
            watched.url,
            "https://nako.example/api/users/me/playback-state/items/item%2F1/watched"
        );
    }
}
