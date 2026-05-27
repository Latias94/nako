pub use nako_core::{
    EffectivePlaybackPolicy, EffectivePlaybackPolicyReason, PlaybackPermission,
    PlaybackPermissionDecision, PlaybackPermissionDecisionReason, PlaybackPermissionPolicy,
    PlaybackTargetId, PlaybackTargetKind, PlaybackTargetNetworkScope, PlaybackTargetTransportAuth,
    RendererControlCapabilities, RendererControlCommand,
};
use nako_core::{LibraryId, MediaProbeResult, MediaSource, MediaSourceId, MediaStreamKind, Result};
use nako_transcode::{
    HlsTranscodeProfile, OutputContainer, RemuxContainer, RemuxTranscodeProfile,
    TranscodeAccelerationPlan, TranscodeExecutionPolicy, TranscodeOutputConstraints, TranscodePlan,
    TranscodeProfile, TranscodeTrackSelection, validate_playback_transcode_plan,
    validate_transcode_profile,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackDecision {
    pub mode: PlaybackMode,
    pub reason: PlaybackDecisionReason,
    pub selected_source: PlaybackSelectedSource,
    pub execution: PlaybackExecutionPlan,
    pub direct_play: Option<DirectPlayPlan>,
    pub transcode_plan: Option<TranscodePlan>,
    pub denial: Option<PlaybackDenial>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackMode {
    DirectPlay,
    Remux,
    Transcode,
    Denied,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackDecisionReason {
    Compatible,
    RequestedTranscodeOutput,
    ClientDisabledDirectPlay,
    SourceContainerUnknown,
    ClientContainerUnsupported,
    SourceCodecsUnsupported,
    PolicyDenied,
}

impl PlaybackDecisionReason {
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Compatible => {
                "source container and codecs are compatible with client capabilities"
            }
            Self::RequestedTranscodeOutput => "playback request requires transcode output",
            Self::ClientDisabledDirectPlay => "client disabled direct play",
            Self::SourceContainerUnknown => "source container could not be inferred from file name",
            Self::ClientContainerUnsupported => {
                "client does not advertise support for the source container"
            }
            Self::SourceCodecsUnsupported => {
                "source codecs are not compatible with client capabilities"
            }
            Self::PolicyDenied => "effective playback policy denied the selected playback mode",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackDenial {
    pub permission: PlaybackPermission,
    pub reason: PlaybackPermissionDecisionReason,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DirectPlayPlan {
    pub source_id: MediaSourceId,
    pub content_type: String,
    pub supports_range_requests: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaybackPlanningRequest<'a> {
    pub source: &'a MediaSource,
    pub probe: Option<&'a MediaProbeResult>,
    pub target: &'a PlaybackTarget,
    pub effective_policy: &'a EffectivePlaybackPolicy,
    pub context: PlaybackSelectionContext,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackSelectionContext {
    pub storage: PlaybackStorageContext,
    pub preferences: PlaybackPreferenceContext,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackStorageContext {
    pub remote: bool,
    pub range_readable: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackPreferenceContext {
    pub requested_audio_stream: Option<u32>,
    pub requested_subtitle_stream: Option<u32>,
    pub max_video_bitrate: Option<u64>,
    pub prefer_hdr: Option<bool>,
    pub remux_output_container: Option<RemuxContainer>,
    pub transcode_output_container: Option<OutputContainer>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackProfile {
    pub direct_play: bool,
    pub containers: Vec<String>,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
    pub storage: PlaybackStorageContext,
    pub preferences: PlaybackPreferenceContext,
}

impl PlaybackProfile {
    #[must_use]
    pub fn from_context(
        client: &ClientPlaybackCapabilities,
        context: PlaybackSelectionContext,
    ) -> Self {
        Self {
            direct_play: client.direct_play,
            containers: normalized_values(&client.containers),
            video_codecs: normalized_values(&client.video_codecs),
            audio_codecs: normalized_values(&client.audio_codecs),
            storage: context.storage,
            preferences: context.preferences,
        }
    }

    #[must_use]
    pub fn identity(&self) -> PlaybackProfileIdentity {
        PlaybackProfileIdentity {
            request_key: format!(
                "playback-profile:v1;direct={};containers={};vcodecs={};acodecs={};remote={};range={};audio={};subtitle={};max_video_bitrate={};prefer_hdr={};remux={};transcode={}",
                self.direct_play,
                list_key(&self.containers),
                list_key(&self.video_codecs),
                list_key(&self.audio_codecs),
                self.storage.remote,
                optional_bool(self.storage.range_readable),
                optional_u32(self.preferences.requested_audio_stream),
                optional_u32(self.preferences.requested_subtitle_stream),
                optional_u64(self.preferences.max_video_bitrate),
                optional_bool(self.preferences.prefer_hdr),
                self.preferences
                    .remux_output_container
                    .map_or("auto", RemuxContainer::file_extension),
                self.preferences
                    .transcode_output_container
                    .map_or("auto", OutputContainer::as_str),
            ),
        }
    }

    #[must_use]
    pub fn identity_key(&self) -> String {
        self.identity().persisted_request_key().to_owned()
    }

    #[must_use]
    pub fn track_selection(&self) -> TranscodeTrackSelection {
        TranscodeTrackSelection {
            audio_stream: self.preferences.requested_audio_stream,
            subtitle_stream: self.preferences.requested_subtitle_stream,
        }
    }

    #[must_use]
    pub fn remux_transcode_profile(&self, output_container: RemuxContainer) -> TranscodeProfile {
        self.try_remux_transcode_profile(output_container)
            .expect("playback remux profile must be valid")
    }

    pub fn try_remux_transcode_profile(
        &self,
        output_container: RemuxContainer,
    ) -> Result<TranscodeProfile> {
        let profile = TranscodeProfile::remux(RemuxTranscodeProfile {
            output_container,
            track_selection: self.track_selection(),
            remote_input: self.storage.remote,
            playback_profile_key: self.identity().persisted_request_key().to_owned(),
        });
        validate_transcode_profile(&profile)?;
        Ok(profile)
    }

    #[must_use]
    pub fn hls_transcode_profile(
        &self,
        plan: &TranscodePlan,
        acceleration: TranscodeAccelerationPlan,
    ) -> TranscodeProfile {
        self.try_hls_transcode_profile(plan, acceleration)
            .expect("playback hls profile must be valid")
    }

    pub fn try_hls_transcode_profile(
        &self,
        plan: &TranscodePlan,
        acceleration: TranscodeAccelerationPlan,
    ) -> Result<TranscodeProfile> {
        validate_playback_transcode_plan(plan)?;
        let track_selection = self.track_selection();
        let output_constraints = TranscodeOutputConstraints {
            max_video_bitrate: self.preferences.max_video_bitrate,
            prefer_hdr: self.preferences.prefer_hdr,
        };
        let profile = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: plan.video_codec.clone(),
            audio_codec: plan.audio_codec.clone(),
            execution_policy: TranscodeExecutionPolicy::hls_single_variant(
                acceleration,
                track_selection,
                output_constraints,
            ),
            track_selection,
            remote_input: self.storage.remote,
            playback_profile_key: self.identity().persisted_request_key().to_owned(),
        });
        validate_transcode_profile(&profile)?;
        Ok(profile)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PlaybackProfileIdentity {
    request_key: String,
}

impl PlaybackProfileIdentity {
    #[must_use]
    pub fn persisted_request_key(&self) -> &str {
        &self.request_key
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackSelectedSource {
    pub source_id: MediaSourceId,
    pub library_id: LibraryId,
    pub locator: String,
    pub file_name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum PlaybackExecutionPlan {
    DirectPlay(DirectPlayPlan),
    Remux(RemuxPlaybackPlan),
    Transcode(TranscodePlan),
    Denied(PlaybackDenial),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemuxPlaybackPlan {
    pub source_id: MediaSourceId,
    pub input_locator: String,
    pub output_container: RemuxContainer,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientPlaybackCapabilities {
    pub direct_play: bool,
    pub containers: Vec<String>,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
}

impl Default for ClientPlaybackCapabilities {
    fn default() -> Self {
        Self {
            direct_play: true,
            containers: vec!["mp4".to_owned(), "m4v".to_owned(), "webm".to_owned()],
            video_codecs: vec!["h264".to_owned(), "hevc".to_owned(), "vp9".to_owned()],
            audio_codecs: vec!["aac".to_owned(), "mp3".to_owned(), "opus".to_owned()],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackTarget {
    pub id: PlaybackTargetId,
    pub kind: PlaybackTargetKind,
    pub display_name: String,
    pub network_scope: PlaybackTargetNetworkScope,
    pub transport_auth: PlaybackTargetTransportAuth,
    pub media_capabilities: ClientPlaybackCapabilities,
    pub control_capabilities: RendererControlCapabilities,
}

impl PlaybackTarget {
    #[must_use]
    pub fn browser_with_capabilities(
        display_name: impl Into<String>,
        media_capabilities: ClientPlaybackCapabilities,
    ) -> Self {
        Self {
            id: PlaybackTargetId::new(),
            kind: PlaybackTargetKind::Browser,
            display_name: display_name.into(),
            network_scope: PlaybackTargetNetworkScope::Local,
            transport_auth: PlaybackTargetTransportAuth::BrowserTicket,
            media_capabilities,
            control_capabilities: RendererControlCapabilities::none(),
        }
    }

    #[must_use]
    pub fn browser_default(display_name: impl Into<String>) -> Self {
        Self::browser_with_capabilities(display_name, ClientPlaybackCapabilities::default())
    }

    #[must_use]
    pub fn nako_remote_client(
        display_name: impl Into<String>,
        media_capabilities: ClientPlaybackCapabilities,
    ) -> Self {
        Self {
            id: PlaybackTargetId::new(),
            kind: PlaybackTargetKind::NakoRemoteClient,
            display_name: display_name.into(),
            network_scope: PlaybackTargetNetworkScope::Local,
            transport_auth: PlaybackTargetTransportAuth::Bearer,
            media_capabilities,
            control_capabilities: RendererControlCapabilities::full_remote_player(),
        }
    }

    #[must_use]
    pub fn requires_ticket_transport(&self) -> bool {
        self.transport_auth.uses_ticket()
    }

    #[must_use]
    pub const fn requires_cast_permission(&self) -> bool {
        matches!(
            self.kind,
            PlaybackTargetKind::NakoRemoteClient
                | PlaybackTargetKind::Chromecast
                | PlaybackTargetKind::DlnaRenderer
                | PlaybackTargetKind::Airplay
        )
    }

    #[must_use]
    pub const fn is_remote_network(&self) -> bool {
        matches!(self.network_scope, PlaybackTargetNetworkScope::Remote)
    }
}

#[derive(Clone, Debug, Default)]
pub struct PlaybackPlanner;

impl PlaybackPlanner {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn plan(&self, request: PlaybackPlanningRequest<'_>) -> PlaybackDecision {
        plan_playback(request)
    }
}

#[must_use]
pub fn plan_playback(request: PlaybackPlanningRequest<'_>) -> PlaybackDecision {
    let PlaybackPlanningRequest {
        source,
        probe,
        target,
        effective_policy,
        context,
    } = request;
    let client = &target.media_capabilities;
    let content_type = content_type_for_file_name(&source.file_name).to_owned();
    let container = container_for_file_name(&source.file_name);
    let selected_source = PlaybackSelectedSource::from(source);

    if target.is_remote_network() || context.storage.remote {
        if let Some(denial) = policy_denial(effective_policy, PlaybackPermission::RemotePlayback) {
            return denied_decision(selected_source, denial);
        }
    }

    if target.requires_cast_permission() {
        if let Some(denial) = policy_denial(effective_policy, PlaybackPermission::Cast) {
            return denied_decision(selected_source, denial);
        }
    }

    if let Some(output_container) = context.preferences.transcode_output_container {
        if let Some(denial) = transcode_policy_denial(effective_policy) {
            return denied_decision(selected_source, denial);
        }
        return transcode_decision(
            selected_source,
            source.locator.clone(),
            output_container,
            PlaybackDecisionReason::RequestedTranscodeOutput,
        );
    }

    if !client.direct_play {
        if let Some(denial) = transcode_policy_denial(effective_policy) {
            return denied_decision(selected_source, denial);
        }
        return transcode_decision(
            selected_source,
            source.locator.clone(),
            OutputContainer::Hls,
            PlaybackDecisionReason::ClientDisabledDirectPlay,
        );
    }

    let Some(container) = container else {
        if let Some(denial) = transcode_policy_denial(effective_policy) {
            return denied_decision(selected_source, denial);
        }
        return transcode_decision(
            selected_source,
            source.locator.clone(),
            OutputContainer::Hls,
            PlaybackDecisionReason::SourceContainerUnknown,
        );
    };

    let container_allowed = client.containers.is_empty()
        || client
            .containers
            .iter()
            .any(|value| value.eq_ignore_ascii_case(container));

    if !container_allowed {
        let codecs_allowed = probe.is_some_and(|probe| codecs_are_supported(probe, client));

        return if codecs_allowed {
            if let Some(denial) = policy_denial(effective_policy, PlaybackPermission::Remux) {
                return denied_decision(selected_source, denial);
            }
            remux_decision(
                selected_source,
                source.locator.clone(),
                context
                    .preferences
                    .remux_output_container
                    .unwrap_or(RemuxContainer::Mp4),
                PlaybackDecisionReason::ClientContainerUnsupported,
            )
        } else {
            if let Some(denial) = transcode_policy_denial(effective_policy) {
                return denied_decision(selected_source, denial);
            }
            transcode_decision(
                selected_source,
                source.locator.clone(),
                OutputContainer::Hls,
                PlaybackDecisionReason::ClientContainerUnsupported,
            )
        };
    }

    if probe.is_some_and(|probe| !codecs_are_supported(probe, client)) {
        if let Some(denial) = transcode_policy_denial(effective_policy) {
            return denied_decision(selected_source, denial);
        }
        return transcode_decision(
            selected_source,
            source.locator.clone(),
            OutputContainer::Hls,
            PlaybackDecisionReason::SourceCodecsUnsupported,
        );
    }

    let direct_play = DirectPlayPlan {
        source_id: source.id,
        content_type,
        supports_range_requests: context.storage.range_readable.unwrap_or(true),
    };
    if let Some(denial) = policy_denial(effective_policy, PlaybackPermission::DirectPlay) {
        return denied_decision(selected_source, denial);
    }
    direct_play_decision(
        selected_source,
        direct_play,
        PlaybackDecisionReason::Compatible,
    )
}

impl From<&MediaSource> for PlaybackSelectedSource {
    fn from(source: &MediaSource) -> Self {
        Self {
            source_id: source.id,
            library_id: source.library_id,
            locator: source.locator.clone(),
            file_name: source.file_name.clone(),
        }
    }
}

fn direct_play_decision(
    selected_source: PlaybackSelectedSource,
    direct_play: DirectPlayPlan,
    reason: PlaybackDecisionReason,
) -> PlaybackDecision {
    PlaybackDecision {
        mode: PlaybackMode::DirectPlay,
        reason,
        selected_source,
        execution: PlaybackExecutionPlan::DirectPlay(direct_play.clone()),
        direct_play: Some(direct_play),
        transcode_plan: None,
        denial: None,
    }
}

fn remux_decision(
    selected_source: PlaybackSelectedSource,
    input_locator: String,
    output_container: RemuxContainer,
    reason: PlaybackDecisionReason,
) -> PlaybackDecision {
    PlaybackDecision {
        mode: PlaybackMode::Remux,
        reason,
        execution: PlaybackExecutionPlan::Remux(RemuxPlaybackPlan {
            source_id: selected_source.source_id,
            input_locator,
            output_container,
        }),
        selected_source,
        direct_play: None,
        transcode_plan: None,
        denial: None,
    }
}

fn transcode_decision(
    selected_source: PlaybackSelectedSource,
    input_locator: String,
    output_container: OutputContainer,
    reason: PlaybackDecisionReason,
) -> PlaybackDecision {
    let transcode_plan = TranscodePlan {
        input_locator,
        output_container,
        video_codec: Some("h264".to_owned()),
        audio_codec: Some("aac".to_owned()),
    };

    PlaybackDecision {
        mode: PlaybackMode::Transcode,
        reason,
        execution: PlaybackExecutionPlan::Transcode(transcode_plan.clone()),
        selected_source,
        direct_play: None,
        transcode_plan: Some(transcode_plan),
        denial: None,
    }
}

fn denied_decision(
    selected_source: PlaybackSelectedSource,
    denial: PlaybackDenial,
) -> PlaybackDecision {
    PlaybackDecision {
        mode: PlaybackMode::Denied,
        reason: PlaybackDecisionReason::PolicyDenied,
        selected_source,
        execution: PlaybackExecutionPlan::Denied(denial),
        direct_play: None,
        transcode_plan: None,
        denial: Some(denial),
    }
}

fn policy_denial(
    policy: &EffectivePlaybackPolicy,
    permission: PlaybackPermission,
) -> Option<PlaybackDenial> {
    let decision = policy.check(permission);
    (!decision.allowed).then_some(PlaybackDenial {
        permission,
        reason: decision.reason,
    })
}

fn transcode_policy_denial(policy: &EffectivePlaybackPolicy) -> Option<PlaybackDenial> {
    policy_denial(policy, PlaybackPermission::VideoTranscode)
        .or_else(|| policy_denial(policy, PlaybackPermission::AudioTranscode))
}

fn codecs_are_supported(probe: &MediaProbeResult, client: &ClientPlaybackCapabilities) -> bool {
    probe.streams.iter().all(|stream| match stream.kind {
        MediaStreamKind::Video => codec_allowed(stream.codec.as_deref(), &client.video_codecs),
        MediaStreamKind::Audio => codec_allowed(stream.codec.as_deref(), &client.audio_codecs),
        MediaStreamKind::Subtitle | MediaStreamKind::Data | MediaStreamKind::Attachment => true,
        MediaStreamKind::Other(_) => true,
    })
}

fn codec_allowed(codec: Option<&str>, allowed: &[String]) -> bool {
    allowed.is_empty()
        || codec.is_none_or(|codec| {
            allowed
                .iter()
                .any(|value| value.eq_ignore_ascii_case(codec))
        })
}

fn container_for_file_name(file_name: &str) -> Option<&str> {
    match extension(file_name)?.as_str() {
        "mp4" | "m4v" => Some("mp4"),
        "webm" => Some("webm"),
        "mkv" => Some("mkv"),
        "mov" => Some("mov"),
        "avi" => Some("avi"),
        "ts" | "m2ts" | "mts" => Some("mpegts"),
        _ => None,
    }
}

#[must_use]
pub fn content_type_for_file_name(file_name: &str) -> &'static str {
    match extension(file_name).as_deref().unwrap_or_default() {
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "ts" | "m2ts" | "mts" => "video/mp2t",
        _ => "application/octet-stream",
    }
}

fn extension(file_name: &str) -> Option<String> {
    file_name
        .rsplit_once('.')
        .map(|(_stem, extension)| extension)
        .filter(|extension| !extension.is_empty())
        .map(str::to_ascii_lowercase)
}

fn normalized_values(values: &[String]) -> Vec<String> {
    let mut values = values
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn list_key(values: &[String]) -> String {
    if values.is_empty() {
        "any".to_owned()
    } else {
        values.join("|")
    }
}

fn optional_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "default".to_owned(), |value| value.to_string())
}

fn optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "auto".to_owned(), |value| value.to_string())
}

fn optional_bool(value: Option<bool>) -> String {
    value.map_or_else(|| "auto".to_owned(), |value| value.to_string())
}

#[cfg(test)]
mod tests {
    use nako_core::{
        MediaProbeResult, MediaSource, MediaSourceId, MediaStreamInfo, MediaStreamKind,
    };

    use super::*;

    #[test]
    fn planner_allows_direct_play_for_compatible_mp4() {
        let source = media_source("movie.mp4");
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![
                stream(MediaStreamKind::Video, Some("h264")),
                stream(MediaStreamKind::Audio, Some("aac")),
            ],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext::default(),
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert_eq!(decision.mode, PlaybackMode::DirectPlay);
        assert_eq!(decision.reason, PlaybackDecisionReason::Compatible);
        assert_eq!(decision.selected_source.source_id, source.id);
        assert!(matches!(
            decision.execution,
            PlaybackExecutionPlan::DirectPlay(_)
        ));
        assert_eq!(
            decision.direct_play.unwrap().content_type,
            "video/mp4".to_owned()
        );
    }

    #[test]
    fn unsupported_container_with_supported_codecs_requests_remux() {
        let source = media_source("movie.mkv");
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("matroska,webm".to_owned()),
            bit_rate: None,
            streams: vec![
                stream(MediaStreamKind::Video, Some("h264")),
                stream(MediaStreamKind::Audio, Some("aac")),
            ],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext::default(),
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert_eq!(decision.mode, PlaybackMode::Remux);
        assert_eq!(
            decision.reason,
            PlaybackDecisionReason::ClientContainerUnsupported
        );
        assert!(matches!(
            decision.execution,
            PlaybackExecutionPlan::Remux(RemuxPlaybackPlan {
                output_container: nako_transcode::RemuxContainer::Mp4,
                ..
            })
        ));
    }

    #[test]
    fn planning_request_can_choose_requested_remux_output_container() {
        let source = media_source("movie.mkv");
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("matroska,webm".to_owned()),
            bit_rate: None,
            streams: vec![
                stream(MediaStreamKind::Video, Some("h264")),
                stream(MediaStreamKind::Audio, Some("aac")),
            ],
        };
        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    remux_output_container: Some(nako_transcode::RemuxContainer::Mkv),
                    ..Default::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert!(matches!(
            decision.execution,
            PlaybackExecutionPlan::Remux(RemuxPlaybackPlan {
                output_container: nako_transcode::RemuxContainer::Mkv,
                ..
            })
        ));
    }

    #[test]
    fn planning_request_carries_storage_and_preference_context() {
        let source = media_source("movie.mp4");
        let client = ClientPlaybackCapabilities::default();

        let decision = plan_with_policy(
            &source,
            None,
            client,
            PlaybackSelectionContext {
                storage: PlaybackStorageContext {
                    remote: true,
                    range_readable: Some(false),
                },
                preferences: PlaybackPreferenceContext {
                    requested_audio_stream: Some(1),
                    requested_subtitle_stream: Some(2),
                    max_video_bitrate: Some(4_000_000),
                    prefer_hdr: Some(false),
                    remux_output_container: Some(nako_transcode::RemuxContainer::Mkv),
                    transcode_output_container: None,
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert_eq!(decision.mode, PlaybackMode::DirectPlay);
        assert_eq!(decision.selected_source.library_id, source.library_id);
        assert_eq!(decision.direct_play.unwrap().supports_range_requests, false);
    }

    #[test]
    fn planner_characterizes_remote_context_as_not_a_permission_gate_yet() {
        let source = media_source("movie.mp4");
        let client = ClientPlaybackCapabilities::default();
        let context = PlaybackSelectionContext {
            storage: PlaybackStorageContext {
                remote: true,
                range_readable: Some(true),
            },
            preferences: PlaybackPreferenceContext::default(),
        };

        let target = PlaybackTarget::browser_with_capabilities("Test", client.clone());
        let policy = EffectivePlaybackPolicy::from_library_access(
            source.library_id,
            nako_core::LibraryAccessLevel::Play,
        );
        let profile = PlaybackProfile::from_context(&client, context.clone());
        let decision = PlaybackPlanner::new().plan(PlaybackPlanningRequest {
            source: &source,
            probe: None,
            target: &target,
            effective_policy: &policy,
            context,
        });

        assert_eq!(decision.mode, PlaybackMode::DirectPlay);
        assert_eq!(decision.reason, PlaybackDecisionReason::Compatible);
        assert!(profile.identity_key().contains("remote=true"));
        assert!(!profile.identity_key().contains("allow_remote"));
        assert!(!profile.identity_key().contains("policy"));
    }

    #[test]
    fn playback_target_records_keep_transport_separate_from_media_capabilities() {
        let browser = PlaybackTarget::browser_default("Web");
        let remote = PlaybackTarget::nako_remote_client(
            "Living Room",
            ClientPlaybackCapabilities {
                direct_play: true,
                containers: vec!["mp4".to_owned()],
                video_codecs: vec!["h264".to_owned()],
                audio_codecs: vec!["aac".to_owned()],
            },
        );

        assert_eq!(browser.kind, PlaybackTargetKind::Browser);
        assert_eq!(
            browser.transport_auth,
            PlaybackTargetTransportAuth::BrowserTicket
        );
        assert!(browser.requires_ticket_transport());
        assert!(browser.media_capabilities.direct_play);
        assert_eq!(remote.kind, PlaybackTargetKind::NakoRemoteClient);
        assert_eq!(remote.transport_auth, PlaybackTargetTransportAuth::Bearer);
        assert!(!remote.requires_ticket_transport());
        assert!(
            remote
                .control_capabilities
                .supports(RendererControlCommand::Play)
        );
    }

    #[test]
    fn playback_policy_records_are_available_to_planner_crate_without_enforcement_yet() {
        let policy = EffectivePlaybackPolicy::from_library_access(
            nako_core::LibraryId::new(),
            nako_core::LibraryAccessLevel::Play,
        );

        assert!(policy.check(PlaybackPermission::DirectPlay).allowed);
        assert!(policy.check(PlaybackPermission::VideoTranscode).allowed);
        assert_eq!(
            policy.check(PlaybackPermission::Cast).reason,
            PlaybackPermissionDecisionReason::CastDisabled
        );
    }

    #[test]
    fn planning_request_can_require_hls_transcode_output() {
        let source = media_source("movie.mp4");
        let client = ClientPlaybackCapabilities::default();

        let decision = plan_with_policy(
            &source,
            None,
            client,
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    transcode_output_container: Some(nako_transcode::OutputContainer::Hls),
                    ..Default::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert_eq!(decision.mode, PlaybackMode::Transcode);
        assert_eq!(
            decision.reason,
            PlaybackDecisionReason::RequestedTranscodeOutput
        );
        assert!(matches!(
            decision.execution,
            PlaybackExecutionPlan::Transcode(nako_transcode::TranscodePlan {
                output_container: nako_transcode::OutputContainer::Hls,
                ..
            })
        ));
    }

    #[test]
    fn playback_profile_identity_normalizes_capability_order_and_case() {
        let left = PlaybackProfile::from_context(
            &ClientPlaybackCapabilities {
                direct_play: true,
                containers: vec!["MP4".to_owned(), "webm".to_owned(), "mp4".to_owned()],
                video_codecs: vec!["H264".to_owned(), "hevc".to_owned()],
                audio_codecs: vec!["AAC".to_owned(), "opus".to_owned()],
            },
            PlaybackSelectionContext {
                storage: PlaybackStorageContext {
                    remote: true,
                    range_readable: Some(false),
                },
                preferences: PlaybackPreferenceContext {
                    requested_audio_stream: Some(2),
                    requested_subtitle_stream: None,
                    max_video_bitrate: Some(8_000_000),
                    prefer_hdr: Some(true),
                    remux_output_container: Some(nako_transcode::RemuxContainer::Mp4),
                    transcode_output_container: Some(nako_transcode::OutputContainer::Hls),
                },
            },
        );
        let right = PlaybackProfile::from_context(
            &ClientPlaybackCapabilities {
                direct_play: true,
                containers: vec!["webm".to_owned(), "mp4".to_owned()],
                video_codecs: vec!["hevc".to_owned(), "h264".to_owned()],
                audio_codecs: vec!["opus".to_owned(), "aac".to_owned()],
            },
            PlaybackSelectionContext {
                storage: PlaybackStorageContext {
                    remote: true,
                    range_readable: Some(false),
                },
                preferences: PlaybackPreferenceContext {
                    requested_audio_stream: Some(2),
                    requested_subtitle_stream: None,
                    max_video_bitrate: Some(8_000_000),
                    prefer_hdr: Some(true),
                    remux_output_container: Some(nako_transcode::RemuxContainer::Mp4),
                    transcode_output_container: Some(nako_transcode::OutputContainer::Hls),
                },
            },
        );

        assert_eq!(left.identity_key(), right.identity_key());
        assert!(left.identity_key().contains("containers=mp4|webm"));
        assert!(left.identity_key().contains("audio=2"));
        assert!(left.identity_key().contains("transcode=hls"));
    }

    #[test]
    fn playback_profile_builds_hls_execution_policy_from_runtime_acceleration() {
        let profile = PlaybackProfile::from_context(
            &ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    requested_subtitle_stream: Some(2),
                    max_video_bitrate: Some(8_000_000),
                    prefer_hdr: Some(true),
                    ..Default::default()
                },
            },
        );
        let plan = nako_transcode::TranscodePlan {
            input_locator: "local:///demo.mkv".to_owned(),
            output_container: nako_transcode::OutputContainer::Hls,
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
        };

        let hls_profile = profile
            .try_hls_transcode_profile(
                &plan,
                nako_transcode::TranscodeAccelerationPlan::for_selected_hardware(
                    nako_transcode::HardwareAcceleration::Nvenc,
                ),
            )
            .unwrap();

        assert_eq!(
            hls_profile.execution_policy.acceleration.encode.accelerator,
            nako_transcode::HardwareAcceleration::Nvenc
        );
        assert_eq!(
            hls_profile
                .execution_policy
                .output_constraints
                .max_video_bitrate,
            Some(8_000_000)
        );
        assert_eq!(
            hls_profile.execution_policy.subtitle_strategy,
            nako_transcode::TranscodeSubtitleStrategy::OmitSelected
        );
    }

    #[test]
    fn planner_denies_direct_play_when_effective_policy_disallows_direct() {
        let source = media_source("movie.mp4");
        let mut permissions = PlaybackPermissionPolicy::current_playback_defaults();
        permissions.allow_direct_play = false;
        let policy = EffectivePlaybackPolicy {
            library_id: source.library_id,
            library_access: nako_core::LibraryAccessLevel::Play,
            permissions,
            reason: EffectivePlaybackPolicyReason::UserPolicy,
        };

        let decision = plan_with_policy(
            &source,
            None,
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext::default(),
            policy,
        );

        assert_policy_denied(
            decision,
            PlaybackPermission::DirectPlay,
            PlaybackPermissionDecisionReason::DirectPlayDisabled,
        );
    }

    #[test]
    fn planner_denies_remux_when_effective_policy_disallows_remux() {
        let source = media_source("movie.mkv");
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("matroska,webm".to_owned()),
            bit_rate: None,
            streams: vec![
                stream(MediaStreamKind::Video, Some("h264")),
                stream(MediaStreamKind::Audio, Some("aac")),
            ],
        };
        let mut permissions = PlaybackPermissionPolicy::current_playback_defaults();
        permissions.allow_remux = false;
        let policy = EffectivePlaybackPolicy {
            library_id: source.library_id,
            library_access: nako_core::LibraryAccessLevel::Play,
            permissions,
            reason: EffectivePlaybackPolicyReason::UserPolicy,
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext::default(),
            policy,
        );

        assert_policy_denied(
            decision,
            PlaybackPermission::Remux,
            PlaybackPermissionDecisionReason::RemuxDisabled,
        );
    }

    #[test]
    fn planner_denies_transcode_when_effective_policy_disallows_video_transcode() {
        let source = media_source("movie.mkv");
        let mut permissions = PlaybackPermissionPolicy::current_playback_defaults();
        permissions.allow_video_transcode = false;
        let policy = EffectivePlaybackPolicy {
            library_id: source.library_id,
            library_access: nako_core::LibraryAccessLevel::Play,
            permissions,
            reason: EffectivePlaybackPolicyReason::UserPolicy,
        };

        let decision = plan_with_policy(
            &source,
            None,
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext::default(),
            policy,
        );

        assert_policy_denied(
            decision,
            PlaybackPermission::VideoTranscode,
            PlaybackPermissionDecisionReason::VideoTranscodeDisabled,
        );
    }

    #[test]
    fn planner_denies_remote_target_when_effective_policy_disallows_remote_playback() {
        let source = media_source("movie.mp4");
        let mut permissions = PlaybackPermissionPolicy::current_playback_defaults();
        permissions.allow_remote_playback = false;
        let policy = EffectivePlaybackPolicy {
            library_id: source.library_id,
            library_access: nako_core::LibraryAccessLevel::Play,
            permissions,
            reason: EffectivePlaybackPolicyReason::UserPolicy,
        };
        let target = PlaybackTarget {
            network_scope: PlaybackTargetNetworkScope::Remote,
            ..PlaybackTarget::browser_default("Remote Browser")
        };

        let decision = PlaybackPlanner::new().plan(PlaybackPlanningRequest {
            source: &source,
            probe: None,
            target: &target,
            effective_policy: &policy,
            context: PlaybackSelectionContext::default(),
        });

        assert_policy_denied(
            decision,
            PlaybackPermission::RemotePlayback,
            PlaybackPermissionDecisionReason::RemotePlaybackDisabled,
        );
    }

    #[test]
    fn planner_denies_cast_target_when_effective_policy_disallows_cast() {
        let source = media_source("movie.mp4");
        let target = PlaybackTarget::nako_remote_client(
            "Living Room",
            ClientPlaybackCapabilities::default(),
        );
        let policy = EffectivePlaybackPolicy::from_library_access(
            source.library_id,
            nako_core::LibraryAccessLevel::Play,
        );

        let decision = PlaybackPlanner::new().plan(PlaybackPlanningRequest {
            source: &source,
            probe: None,
            target: &target,
            effective_policy: &policy,
            context: PlaybackSelectionContext::default(),
        });

        assert_policy_denied(
            decision,
            PlaybackPermission::Cast,
            PlaybackPermissionDecisionReason::CastDisabled,
        );
    }

    fn plan_with_policy(
        source: &MediaSource,
        probe: Option<&MediaProbeResult>,
        client: ClientPlaybackCapabilities,
        context: PlaybackSelectionContext,
        effective_policy: EffectivePlaybackPolicy,
    ) -> PlaybackDecision {
        let target = PlaybackTarget::browser_with_capabilities("Test", client);
        PlaybackPlanner::new().plan(PlaybackPlanningRequest {
            source,
            probe,
            target: &target,
            effective_policy: &effective_policy,
            context,
        })
    }

    fn assert_policy_denied(
        decision: PlaybackDecision,
        permission: PlaybackPermission,
        reason: PlaybackPermissionDecisionReason,
    ) {
        assert_eq!(decision.mode, PlaybackMode::Denied);
        assert_eq!(decision.reason, PlaybackDecisionReason::PolicyDenied);
        assert!(matches!(
            decision.execution,
            PlaybackExecutionPlan::Denied(PlaybackDenial {
                permission: actual_permission,
                reason: actual_reason,
            }) if actual_permission == permission && actual_reason == reason
        ));
        assert_eq!(decision.denial, Some(PlaybackDenial { permission, reason }));
        assert!(decision.direct_play.is_none());
        assert!(decision.transcode_plan.is_none());
    }

    fn media_source(file_name: &str) -> MediaSource {
        MediaSource {
            id: MediaSourceId::new(),
            library_id: nako_core::LibraryId::new(),
            item_id: nako_core::MediaItemId::new(),
            locator: format!("local:///{file_name}"),
            file_name: file_name.to_owned(),
            size_bytes: Some(1_000),
            fingerprint: None,
        }
    }

    fn stream(kind: MediaStreamKind, codec: Option<&str>) -> MediaStreamInfo {
        MediaStreamInfo {
            index: 0,
            kind,
            codec: codec.map(ToOwned::to_owned),
            language: None,
            duration_ms: None,
            bit_rate: None,
            width: None,
            height: None,
            channels: None,
            sample_rate: None,
        }
    }
}
