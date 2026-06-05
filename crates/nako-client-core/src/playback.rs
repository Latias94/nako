#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CorePlaybackCapabilities {
    pub direct_play: Option<bool>,
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

impl CorePlaybackCapabilities {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackDecisionRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub source_id: String,
    pub capabilities: CorePlaybackCapabilities,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackSourceRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub source_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackSessionRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub session_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorePlaybackMode {
    DirectPlay,
    Remux,
    Transcode,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreOutputContainer {
    Hls,
    Mp4,
    Mkv,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreHlsVariantPolicy {
    SingleVariant,
    Adaptive,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreHlsSegmentContainer {
    MpegTs,
    Fmp4,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackDecisionSummary {
    pub source_id: String,
    pub mode: CorePlaybackMode,
    pub transcode_output_container: Option<CoreOutputContainer>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackTargetInput {
    pub base_url: String,
    pub decision: CorePlaybackDecisionSummary,
    pub capabilities: CorePlaybackCapabilities,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreDirectPlaybackTargetInput {
    pub base_url: String,
    pub source_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreRemuxPlaybackTargetInput {
    pub base_url: String,
    pub source_id: String,
    pub capabilities: CorePlaybackCapabilities,
    pub output_container: Option<CoreOutputContainer>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreHlsPlaylistTargetInput {
    pub base_url: String,
    pub source_id: String,
    pub capabilities: CorePlaybackCapabilities,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackTarget {
    pub request: crate::CoreHttpRequest,
    pub session_probe_request: Option<crate::CoreHttpRequest>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackSegmentInput {
    pub base_url: String,
    pub session_id: String,
    pub segment_name: String,
}

#[must_use]
pub fn build_source_probe_request(
    input: &CorePlaybackSourceRequestInput,
) -> crate::CoreHttpRequest {
    crate::build_core_request(
        &crate::CoreHttpRequestSpec::new(
            crate::PLAYBACK_SOURCE_PROBE_REQUEST_ID,
            &input.base_url,
            "GET",
            &format!(
                "/sources/{}/probe",
                crate::encode_path_segment(&input.source_id)
            ),
        )
        .access_token(Some(input.access_token.clone())),
    )
}

#[must_use]
pub fn build_playback_decision_request(
    input: &CorePlaybackDecisionRequestInput,
) -> crate::CoreHttpRequest {
    crate::build_core_request(
        &crate::CoreHttpRequestSpec::new(
            crate::PLAYBACK_DECISION_REQUEST_ID,
            &input.base_url,
            "GET",
            &format!(
                "/sources/{}/playback/decision",
                crate::encode_path_segment(&input.source_id)
            ),
        )
        .query(playback_capability_query(&input.capabilities))
        .access_token(Some(input.access_token.clone())),
    )
}

#[must_use]
pub fn build_recommended_playback_target(
    input: &CorePlaybackTargetInput,
) -> Option<CorePlaybackTarget> {
    match input.decision.mode {
        CorePlaybackMode::DirectPlay => Some(build_direct_playback_target(
            &CoreDirectPlaybackTargetInput {
                base_url: input.base_url.clone(),
                source_id: input.decision.source_id.clone(),
            },
        )),
        CorePlaybackMode::Remux => {
            Some(build_remux_playback_target(&CoreRemuxPlaybackTargetInput {
                base_url: input.base_url.clone(),
                source_id: input.decision.source_id.clone(),
                capabilities: input.capabilities.clone(),
                output_container: remux_output_container(&input.decision),
            }))
        }
        CorePlaybackMode::Transcode => {
            Some(build_hls_playlist_target(&CoreHlsPlaylistTargetInput {
                base_url: input.base_url.clone(),
                source_id: input.decision.source_id.clone(),
                capabilities: input.capabilities.clone(),
            }))
        }
        CorePlaybackMode::Unknown => None,
    }
}

#[must_use]
pub fn build_direct_playback_target(input: &CoreDirectPlaybackTargetInput) -> CorePlaybackTarget {
    CorePlaybackTarget {
        request: streaming_request(
            crate::PLAYBACK_DIRECT_STREAM_REQUEST_ID,
            &input.base_url,
            &input.source_id,
            "/stream",
            Vec::new(),
            None,
        ),
        session_probe_request: None,
    }
}

#[must_use]
pub fn build_head_direct_playback_target(
    input: &CoreDirectPlaybackTargetInput,
) -> CorePlaybackTarget {
    CorePlaybackTarget {
        request: streaming_request(
            crate::PLAYBACK_DIRECT_STREAM_HEAD_REQUEST_ID,
            &input.base_url,
            &input.source_id,
            "/stream",
            Vec::new(),
            Some("HEAD"),
        ),
        session_probe_request: None,
    }
}

#[must_use]
pub fn build_remux_playback_target(input: &CoreRemuxPlaybackTargetInput) -> CorePlaybackTarget {
    let query = remux_query(&input.capabilities, input.output_container);
    CorePlaybackTarget {
        request: streaming_request(
            crate::PLAYBACK_REMUX_STREAM_REQUEST_ID,
            &input.base_url,
            &input.source_id,
            "/stream/remux",
            query.clone(),
            None,
        ),
        session_probe_request: Some(streaming_request(
            crate::PLAYBACK_REMUX_SESSION_PROBE_REQUEST_ID,
            &input.base_url,
            &input.source_id,
            "/stream/remux",
            query,
            Some("HEAD"),
        )),
    }
}

#[must_use]
pub fn build_hls_playlist_target(input: &CoreHlsPlaylistTargetInput) -> CorePlaybackTarget {
    let query = playback_capability_query(&input.capabilities);
    CorePlaybackTarget {
        request: streaming_request(
            crate::PLAYBACK_HLS_PLAYLIST_REQUEST_ID,
            &input.base_url,
            &input.source_id,
            "/stream/hls/playlist.m3u8",
            query.clone(),
            None,
        ),
        session_probe_request: Some(streaming_request(
            crate::PLAYBACK_HLS_PLAYLIST_REQUEST_ID,
            &input.base_url,
            &input.source_id,
            "/stream/hls/playlist.m3u8",
            query,
            None,
        )),
    }
}

#[must_use]
pub fn build_hls_segment_request(input: &CorePlaybackSegmentInput) -> crate::CoreHttpRequest {
    crate::build_core_request(&crate::CoreHttpRequestSpec::new(
        crate::PLAYBACK_HLS_SEGMENT_REQUEST_ID,
        &input.base_url,
        "GET",
        &format!(
            "/playback/sessions/{}/hls/segments/{}",
            crate::encode_path_segment(&input.session_id),
            crate::encode_path_segment(&input.segment_name)
        ),
    ))
}

#[must_use]
pub fn build_get_playback_session_request(
    input: &CorePlaybackSessionRequestInput,
) -> crate::CoreHttpRequest {
    build_playback_session_request(
        crate::PLAYBACK_SESSION_REQUEST_ID,
        &input.base_url,
        &input.access_token,
        &input.session_id,
        "GET",
        "",
    )
}

#[must_use]
pub fn build_cancel_playback_session_request(
    input: &CorePlaybackSessionRequestInput,
) -> crate::CoreHttpRequest {
    build_playback_session_request(
        crate::PLAYBACK_CANCEL_SESSION_REQUEST_ID,
        &input.base_url,
        &input.access_token,
        &input.session_id,
        "POST",
        "/cancel",
    )
}

fn streaming_request(
    request_id: &str,
    base_url: &str,
    source_id: &str,
    suffix: &str,
    query: Vec<crate::CoreQueryParam>,
    method: Option<&str>,
) -> crate::CoreHttpRequest {
    crate::build_core_request(
        &crate::CoreHttpRequestSpec::new(
            request_id,
            base_url,
            method.unwrap_or("GET"),
            &format!(
                "/sources/{}{}",
                crate::encode_path_segment(source_id),
                suffix
            ),
        )
        .query(query),
    )
}

fn build_playback_session_request(
    request_id: &str,
    base_url: &str,
    access_token: &str,
    session_id: &str,
    method: &str,
    suffix: &str,
) -> crate::CoreHttpRequest {
    crate::build_core_request(
        &crate::CoreHttpRequestSpec::new(
            request_id,
            base_url,
            method,
            &format!(
                "/playback/sessions/{}{}",
                crate::encode_path_segment(session_id),
                suffix
            ),
        )
        .access_token(Some(access_token.to_owned())),
    )
}

fn playback_capability_query(
    capabilities: &CorePlaybackCapabilities,
) -> Vec<crate::CoreQueryParam> {
    let mut query = Vec::new();
    if let Some(direct_play) = capabilities.direct_play {
        query.push(crate::CoreQueryParam::new(
            "direct_play",
            if direct_play { "true" } else { "false" },
        ));
    }
    if !capabilities.containers.is_empty() {
        query.push(crate::CoreQueryParam::new(
            "container",
            capabilities.containers.join(","),
        ));
    }
    if !capabilities.video_codecs.is_empty() {
        query.push(crate::CoreQueryParam::new(
            "video_codec",
            capabilities.video_codecs.join(","),
        ));
    }
    if !capabilities.audio_codecs.is_empty() {
        query.push(crate::CoreQueryParam::new(
            "audio_codec",
            capabilities.audio_codecs.join(","),
        ));
    }
    if let Some(max_video_bitrate) = capabilities.max_video_bitrate {
        query.push(crate::CoreQueryParam::new(
            "max_video_bitrate",
            max_video_bitrate.to_string(),
        ));
    }
    if let Some(max_width) = capabilities.max_width {
        query.push(crate::CoreQueryParam::new(
            "max_width",
            max_width.to_string(),
        ));
    }
    if let Some(max_height) = capabilities.max_height {
        query.push(crate::CoreQueryParam::new(
            "max_height",
            max_height.to_string(),
        ));
    }
    if let Some(max_audio_channels) = capabilities.max_audio_channels {
        query.push(crate::CoreQueryParam::new(
            "max_audio_channels",
            max_audio_channels.to_string(),
        ));
    }
    if let Some(supports_hdr) = capabilities.supports_hdr {
        query.push(crate::CoreQueryParam::new(
            "supports_hdr",
            supports_hdr.to_string(),
        ));
    }
    if let Some(supports_subtitles) = capabilities.supports_subtitles {
        query.push(crate::CoreQueryParam::new(
            "supports_subtitles",
            supports_subtitles.to_string(),
        ));
    }
    if let Some(value) = capabilities
        .hls_variant_policy
        .and_then(hls_variant_policy_wire_value)
    {
        query.push(crate::CoreQueryParam::new("hls_variant_policy", value));
    }
    if let Some(value) = capabilities
        .hls_segment_container
        .and_then(hls_segment_container_wire_value)
    {
        query.push(crate::CoreQueryParam::new("hls_segment_container", value));
    }
    query
}

fn remux_query(
    capabilities: &CorePlaybackCapabilities,
    output_container: Option<CoreOutputContainer>,
) -> Vec<crate::CoreQueryParam> {
    let mut query = playback_capability_query(capabilities);
    if let Some(value) = output_container.and_then(output_container_wire_value) {
        query.push(crate::CoreQueryParam::new("output_container", value));
    }
    query
}

fn remux_output_container(decision: &CorePlaybackDecisionSummary) -> Option<CoreOutputContainer> {
    match decision.transcode_output_container {
        Some(CoreOutputContainer::Mkv) => Some(CoreOutputContainer::Mkv),
        Some(CoreOutputContainer::Mp4) | None => Some(CoreOutputContainer::Mp4),
        Some(CoreOutputContainer::Hls | CoreOutputContainer::Unknown) => None,
    }
}

fn output_container_wire_value(value: CoreOutputContainer) -> Option<&'static str> {
    match value {
        CoreOutputContainer::Hls | CoreOutputContainer::Unknown => None,
        CoreOutputContainer::Mp4 => Some("mp4"),
        CoreOutputContainer::Mkv => Some("mkv"),
    }
}

fn hls_variant_policy_wire_value(value: CoreHlsVariantPolicy) -> Option<&'static str> {
    match value {
        CoreHlsVariantPolicy::SingleVariant => Some("single_variant"),
        CoreHlsVariantPolicy::Adaptive => Some("adaptive"),
        CoreHlsVariantPolicy::Unknown => None,
    }
}

fn hls_segment_container_wire_value(value: CoreHlsSegmentContainer) -> Option<&'static str> {
    match value {
        CoreHlsSegmentContainer::MpegTs => Some("mpeg_ts"),
        CoreHlsSegmentContainer::Fmp4 => Some("fmp4"),
        CoreHlsSegmentContainer::Unknown => None,
    }
}
