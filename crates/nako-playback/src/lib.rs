pub use nako_core::{
    EffectivePlaybackPolicy, EffectivePlaybackPolicyReason, PlaybackPermission,
    PlaybackPermissionDecision, PlaybackPermissionDecisionReason, PlaybackPermissionPolicy,
    PlaybackTargetId, PlaybackTargetKind, PlaybackTargetNetworkScope, PlaybackTargetTransportAuth,
    RendererControlCapabilities, RendererControlCommand,
};
use nako_core::{
    LibraryId, MediaProbeResult, MediaSource, MediaSourceId, MediaStreamInfo, MediaStreamKind,
};
use serde::{Deserialize, Serialize};

mod capability;
mod values;

pub use capability::{
    DirectPlayCapabilityProfile, PlaybackCapabilityEvaluation, PlaybackCompatibilityCondition,
    PlaybackDecisionReport, PlaybackTargetProfile, RemuxCapabilityProfile,
    TranscodeCapabilityProfile,
};
use capability::{evaluate_direct_play, evaluate_remux, evaluate_transcode};
pub use values::{
    PlaybackAudioCompatibilityReason, PlaybackAudioDownmixRequirement,
    PlaybackAudioNormalizationRequirement, PlaybackAudioOutputRequirement,
    PlaybackColorCompatibilityReason, PlaybackColorPipelineRequirement,
    PlaybackColorPipelineSource, PlaybackColorPipelineTarget, PlaybackHdrToneMappingRequirement,
    PlaybackHlsOutputRequirement, PlaybackHlsSegmentContainer, PlaybackHlsVariantPolicy,
    PlaybackOutputConstraints, PlaybackRemuxContainer, PlaybackSubtitleStrategy,
    PlaybackTrackSelection, PlaybackTranscodeContainer, PlaybackTranscodePlan,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackDecision {
    pub mode: PlaybackMode,
    pub reason: PlaybackDecisionReason,
    pub selected_source: PlaybackSelectedSource,
    pub rendition: PlaybackRenditionPlan,
    pub report: PlaybackDecisionReport,
    pub denial: Option<PlaybackDenial>,
}

impl PlaybackDecision {
    #[must_use]
    pub fn direct_play_plan(&self) -> Option<&DirectPlayPlan> {
        match &self.rendition {
            PlaybackRenditionPlan::DirectPlay(plan) => Some(plan),
            PlaybackRenditionPlan::Remux(_)
            | PlaybackRenditionPlan::Transcode(_)
            | PlaybackRenditionPlan::Denied(_) => None,
        }
    }

    #[must_use]
    pub fn remux_plan(&self) -> Option<&RemuxPlaybackPlan> {
        match &self.rendition {
            PlaybackRenditionPlan::Remux(plan) => Some(plan),
            PlaybackRenditionPlan::DirectPlay(_)
            | PlaybackRenditionPlan::Transcode(_)
            | PlaybackRenditionPlan::Denied(_) => None,
        }
    }

    #[must_use]
    pub fn transcode_plan(&self) -> Option<&PlaybackTranscodePlan> {
        match &self.rendition {
            PlaybackRenditionPlan::Transcode(plan) => Some(&plan.plan),
            PlaybackRenditionPlan::DirectPlay(_)
            | PlaybackRenditionPlan::Remux(_)
            | PlaybackRenditionPlan::Denied(_) => None,
        }
    }

    #[must_use]
    pub fn transcode_requirement(&self) -> Option<&TranscodeRequirement> {
        match &self.rendition {
            PlaybackRenditionPlan::Transcode(plan) => Some(&plan.requirement),
            PlaybackRenditionPlan::DirectPlay(_)
            | PlaybackRenditionPlan::Remux(_)
            | PlaybackRenditionPlan::Denied(_) => None,
        }
    }
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
    #[serde(default)]
    pub preferred_audio_languages: Vec<String>,
    pub requested_subtitle_stream: Option<u32>,
    #[serde(default)]
    pub preferred_subtitle_languages: Vec<String>,
    pub max_video_bitrate: Option<u64>,
    pub prefer_hdr: Option<bool>,
    pub remux_output_container: Option<PlaybackRemuxContainer>,
    pub transcode_output_container: Option<PlaybackTranscodeContainer>,
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
pub enum PlaybackRenditionPlan {
    DirectPlay(DirectPlayPlan),
    Remux(RemuxPlaybackPlan),
    Transcode(TranscodeRenditionPlan),
    Denied(PlaybackDenial),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeRenditionPlan {
    pub plan: PlaybackTranscodePlan,
    pub requirement: TranscodeRequirement,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemuxPlaybackPlan {
    pub source_id: MediaSourceId,
    pub input_locator: String,
    pub output_container: PlaybackRemuxContainer,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeRequirement {
    pub source_id: MediaSourceId,
    pub input_locator: String,
    pub output_container: PlaybackTranscodeContainer,
    pub output_video_codec: Option<String>,
    pub output_audio_codec: Option<String>,
    pub track_selection: PlaybackTrackSelection,
    pub output_constraints: PlaybackOutputConstraints,
    pub color_pipeline: PlaybackColorPipelineRequirement,
    pub audio_output: PlaybackAudioOutputRequirement,
    pub hls_output: Option<PlaybackHlsOutputRequirement>,
    pub subtitle_strategy: PlaybackSubtitleStrategy,
    pub selected_streams: TranscodeRequirementStreams,
    pub reasons: Vec<PlaybackCompatibilityCondition>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeRequirementStreams {
    pub video: Option<TranscodeRequirementStream>,
    pub audio: Option<TranscodeRequirementStream>,
    pub subtitle: Option<TranscodeRequirementStream>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeRequirementStream {
    pub index: u32,
    pub kind: MediaStreamKind,
    pub codec: Option<String>,
    pub language: Option<String>,
    pub duration_ms: Option<u64>,
    pub bit_rate: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub channels: Option<u32>,
    pub sample_rate: Option<u32>,
    pub codec_profile: Option<String>,
    pub codec_level: Option<u32>,
    pub pixel_format: Option<String>,
    pub bits_per_raw_sample: Option<u32>,
    pub bits_per_sample: Option<u32>,
    pub dynamic_range: Option<String>,
    pub color_space: Option<String>,
    pub color_transfer: Option<String>,
    pub color_primaries: Option<String>,
    pub mastering_display: bool,
    pub content_light_level: bool,
    pub dolby_vision: bool,
    pub hdr10_plus: bool,
    pub channel_layout: Option<String>,
    pub forced: bool,
    pub default: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientPlaybackCapabilities {
    pub direct_play: bool,
    pub containers: Vec<String>,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
    pub max_video_bitrate: Option<u64>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub max_audio_channels: Option<u32>,
    #[serde(default = "default_true")]
    pub supports_hdr: bool,
    #[serde(default = "default_true")]
    pub supports_subtitles: bool,
    #[serde(default)]
    pub hls_variant_policy: PlaybackHlsVariantPolicy,
    #[serde(default)]
    pub hls_segment_container: PlaybackHlsSegmentContainer,
}

impl Default for ClientPlaybackCapabilities {
    fn default() -> Self {
        Self {
            direct_play: true,
            containers: vec!["mp4".to_owned(), "m4v".to_owned(), "webm".to_owned()],
            video_codecs: vec!["h264".to_owned(), "hevc".to_owned(), "vp9".to_owned()],
            audio_codecs: vec!["aac".to_owned(), "mp3".to_owned(), "opus".to_owned()],
            max_video_bitrate: None,
            max_width: None,
            max_height: None,
            max_audio_channels: None,
            supports_hdr: true,
            supports_subtitles: true,
            hls_variant_policy: PlaybackHlsVariantPolicy::SingleVariant,
            hls_segment_container: PlaybackHlsSegmentContainer::MpegTs,
        }
    }
}

const fn default_true() -> bool {
    true
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
    let target_profile = PlaybackTargetProfile::from_target(target, context);
    let content_type = content_type_for_file_name(&source.file_name).to_owned();
    let container = container_for_file_name(&source.file_name);
    let selected_source = PlaybackSelectedSource::from(source);
    let mut report = PlaybackDecisionReport::new(source.id, target_profile.identity_key());

    if target.is_remote_network() || target_profile.storage.remote {
        if let Some(denial) = policy_denial(effective_policy, PlaybackPermission::RemotePlayback) {
            return denied_decision(selected_source, denial, report);
        }
    }

    if target.requires_cast_permission() {
        if let Some(denial) = policy_denial(effective_policy, PlaybackPermission::Cast) {
            return denied_decision(selected_source, denial, report);
        }
    }

    report.direct_play = evaluate_direct_play(probe, &target_profile, container);
    report.remux = evaluate_remux(probe, &target_profile);
    report.transcode = evaluate_transcode(&target_profile);

    if let Some(output_container) = target_profile.preferences.transcode_output_container {
        if let Some(denial) = transcode_policy_denial(effective_policy) {
            return denied_decision(selected_source, denial, report);
        }
        report.transcode = PlaybackCapabilityEvaluation::unsupported(vec![
            PlaybackCompatibilityCondition::RequestedTranscodeOutput,
        ]);
        return transcode_decision(
            selected_source,
            source.locator.clone(),
            output_container,
            PlaybackDecisionReason::RequestedTranscodeOutput,
            report,
            &target_profile,
            probe,
        );
    }

    if !target_profile.direct_play {
        if let Some(denial) = transcode_policy_denial(effective_policy) {
            return denied_decision(selected_source, denial, report);
        }
        return transcode_decision(
            selected_source,
            source.locator.clone(),
            PlaybackTranscodeContainer::Hls,
            PlaybackDecisionReason::ClientDisabledDirectPlay,
            report,
            &target_profile,
            probe,
        );
    }

    let Some(_container) = container else {
        if let Some(denial) = transcode_policy_denial(effective_policy) {
            return denied_decision(selected_source, denial, report);
        }
        return transcode_decision(
            selected_source,
            source.locator.clone(),
            PlaybackTranscodeContainer::Hls,
            PlaybackDecisionReason::SourceContainerUnknown,
            report,
            &target_profile,
            probe,
        );
    };

    if report
        .direct_play
        .has(PlaybackCompatibilityCondition::ContainerUnsupported)
    {
        return if report.remux.supported {
            if let Some(denial) = policy_denial(effective_policy, PlaybackPermission::Remux) {
                return denied_decision(selected_source, denial, report);
            }
            remux_decision(
                selected_source,
                source.locator.clone(),
                target_profile
                    .preferences
                    .remux_output_container
                    .unwrap_or(PlaybackRemuxContainer::Mp4),
                PlaybackDecisionReason::ClientContainerUnsupported,
                report,
            )
        } else {
            if let Some(denial) = transcode_policy_denial(effective_policy) {
                return denied_decision(selected_source, denial, report);
            }
            transcode_decision(
                selected_source,
                source.locator.clone(),
                PlaybackTranscodeContainer::Hls,
                PlaybackDecisionReason::ClientContainerUnsupported,
                report,
                &target_profile,
                probe,
            )
        };
    }

    if !report.direct_play.supported {
        if let Some(denial) = transcode_policy_denial(effective_policy) {
            return denied_decision(selected_source, denial, report);
        }
        return transcode_decision(
            selected_source,
            source.locator.clone(),
            PlaybackTranscodeContainer::Hls,
            PlaybackDecisionReason::SourceCodecsUnsupported,
            report,
            &target_profile,
            probe,
        );
    }

    let direct_play = DirectPlayPlan {
        source_id: source.id,
        content_type,
        supports_range_requests: target_profile.storage.range_readable.unwrap_or(true),
    };
    if let Some(denial) = policy_denial(effective_policy, PlaybackPermission::DirectPlay) {
        return denied_decision(selected_source, denial, report);
    }
    direct_play_decision(
        selected_source,
        direct_play,
        PlaybackDecisionReason::Compatible,
        report,
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
    report: PlaybackDecisionReport,
) -> PlaybackDecision {
    let selection_reasons = report.direct_play.reasons.clone();
    PlaybackDecision {
        mode: PlaybackMode::DirectPlay,
        reason,
        selected_source,
        rendition: PlaybackRenditionPlan::DirectPlay(direct_play),
        report: report
            .with_selected_mode(PlaybackMode::DirectPlay)
            .with_selection_reasons(selection_reasons),
        denial: None,
    }
}

fn remux_decision(
    selected_source: PlaybackSelectedSource,
    input_locator: String,
    output_container: PlaybackRemuxContainer,
    reason: PlaybackDecisionReason,
    report: PlaybackDecisionReport,
) -> PlaybackDecision {
    let selection_reasons = selection_reasons_for_remux(reason, &report);
    PlaybackDecision {
        mode: PlaybackMode::Remux,
        reason,
        rendition: PlaybackRenditionPlan::Remux(RemuxPlaybackPlan {
            source_id: selected_source.source_id,
            input_locator,
            output_container,
        }),
        report: report
            .with_selected_mode(PlaybackMode::Remux)
            .with_selection_reasons(selection_reasons),
        selected_source,
        denial: None,
    }
}

fn transcode_decision(
    selected_source: PlaybackSelectedSource,
    input_locator: String,
    output_container: PlaybackTranscodeContainer,
    reason: PlaybackDecisionReason,
    report: PlaybackDecisionReport,
    target_profile: &PlaybackTargetProfile,
    probe: Option<&MediaProbeResult>,
) -> PlaybackDecision {
    let selection_reasons = selection_reasons_for_transcode(reason, &report);
    let output_profile = target_profile
        .transcode_profiles
        .iter()
        .find(|profile| profile.output_container == output_container);
    let video_codec = output_profile
        .and_then(|profile| profile.video_codec.clone())
        .or_else(|| {
            (output_container == PlaybackTranscodeContainer::Hls).then(|| "h264".to_owned())
        });
    let audio_codec = output_profile
        .and_then(|profile| profile.audio_codec.clone())
        .or_else(|| {
            (output_container == PlaybackTranscodeContainer::Hls).then(|| "aac".to_owned())
        });
    let transcode_plan = PlaybackTranscodePlan {
        input_locator: input_locator.clone(),
        output_container,
        video_codec: video_codec.clone(),
        audio_codec: audio_codec.clone(),
    };
    let transcode_requirement = build_transcode_requirement(
        selected_source.source_id,
        input_locator,
        output_container,
        video_codec,
        audio_codec,
        target_profile,
        probe,
        selection_reasons.clone(),
    );

    PlaybackDecision {
        mode: PlaybackMode::Transcode,
        reason,
        rendition: PlaybackRenditionPlan::Transcode(TranscodeRenditionPlan {
            plan: transcode_plan,
            requirement: transcode_requirement,
        }),
        report: report
            .with_selected_mode(PlaybackMode::Transcode)
            .with_selection_reasons(selection_reasons),
        selected_source,
        denial: None,
    }
}

fn denied_decision(
    selected_source: PlaybackSelectedSource,
    denial: PlaybackDenial,
    report: PlaybackDecisionReport,
) -> PlaybackDecision {
    PlaybackDecision {
        mode: PlaybackMode::Denied,
        reason: PlaybackDecisionReason::PolicyDenied,
        selected_source,
        rendition: PlaybackRenditionPlan::Denied(denial),
        report: report
            .with_denial(denial)
            .with_selected_mode(PlaybackMode::Denied),
        denial: Some(denial),
    }
}

fn build_transcode_requirement(
    source_id: MediaSourceId,
    input_locator: String,
    output_container: PlaybackTranscodeContainer,
    output_video_codec: Option<String>,
    output_audio_codec: Option<String>,
    target_profile: &PlaybackTargetProfile,
    probe: Option<&MediaProbeResult>,
    reasons: Vec<PlaybackCompatibilityCondition>,
) -> TranscodeRequirement {
    let track_selection = target_profile.track_selection_for_probe(probe);
    let selected_streams = selected_transcode_streams(probe, track_selection);
    let audio_output = target_profile.audio_output_requirement(
        selected_streams
            .audio
            .as_ref()
            .and_then(|stream| stream.channels),
    );
    let color_pipeline = target_profile.color_pipeline_requirement(
        selected_streams
            .video
            .as_ref()
            .map(PlaybackColorPipelineSource::from),
    );
    TranscodeRequirement {
        source_id,
        input_locator,
        output_container,
        output_video_codec,
        output_audio_codec,
        track_selection,
        output_constraints: target_profile.output_constraints(),
        color_pipeline,
        audio_output,
        hls_output: (output_container == PlaybackTranscodeContainer::Hls)
            .then_some(target_profile.hls_output_requirement()),
        subtitle_strategy: target_profile.subtitle_strategy_for_track_selection(
            output_container,
            track_selection,
            selected_streams
                .subtitle
                .as_ref()
                .and_then(|stream| stream.codec.as_deref()),
        ),
        selected_streams,
        reasons,
    }
}

fn selected_transcode_streams(
    probe: Option<&MediaProbeResult>,
    track_selection: PlaybackTrackSelection,
) -> TranscodeRequirementStreams {
    TranscodeRequirementStreams {
        video: selected_stream(probe, None, |stream| {
            matches!(stream.kind, MediaStreamKind::Video)
        }),
        audio: selected_stream(probe, track_selection.audio_stream, |stream| {
            matches!(stream.kind, MediaStreamKind::Audio)
        }),
        subtitle: selected_stream(probe, track_selection.subtitle_stream, |stream| {
            matches!(stream.kind, MediaStreamKind::Subtitle)
        }),
    }
}

fn selected_stream(
    probe: Option<&MediaProbeResult>,
    requested_stream: Option<u32>,
    matches_kind: impl Fn(&MediaStreamInfo) -> bool,
) -> Option<TranscodeRequirementStream> {
    let probe = probe?;
    requested_stream
        .and_then(|index| {
            probe
                .streams
                .iter()
                .find(|stream| stream.index == index && matches_kind(stream))
        })
        .or_else(|| probe.streams.iter().find(|stream| matches_kind(stream)))
        .map(TranscodeRequirementStream::from)
}

impl From<&MediaStreamInfo> for TranscodeRequirementStream {
    fn from(stream: &MediaStreamInfo) -> Self {
        Self {
            index: stream.index,
            kind: stream.kind.clone(),
            codec: stream.codec.clone(),
            language: stream.language.clone(),
            duration_ms: stream.duration_ms,
            bit_rate: stream.bit_rate,
            width: stream.width,
            height: stream.height,
            channels: stream.channels,
            sample_rate: stream.sample_rate,
            codec_profile: stream.technical.codec_profile.clone(),
            codec_level: stream.technical.codec_level,
            pixel_format: stream.technical.pixel_format.clone(),
            bits_per_raw_sample: stream.technical.bits_per_raw_sample,
            bits_per_sample: stream.technical.bits_per_sample,
            dynamic_range: stream.technical.hdr.dynamic_range.clone(),
            color_space: stream.technical.color.space.clone(),
            color_transfer: stream.technical.color.transfer.clone(),
            color_primaries: stream.technical.color.primaries.clone(),
            mastering_display: stream.technical.hdr.mastering_display,
            content_light_level: stream.technical.hdr.content_light_level,
            dolby_vision: stream.technical.hdr.dolby_vision,
            hdr10_plus: stream.technical.hdr.hdr10_plus,
            channel_layout: stream.technical.channel_layout.clone(),
            forced: stream.technical.disposition.forced,
            default: stream.technical.disposition.default,
        }
    }
}

impl From<&TranscodeRequirementStream> for PlaybackColorPipelineSource {
    fn from(stream: &TranscodeRequirementStream) -> Self {
        Self {
            dynamic_range: stream.dynamic_range.clone(),
            color_space: stream.color_space.clone(),
            color_transfer: stream.color_transfer.clone(),
            color_primaries: stream.color_primaries.clone(),
            mastering_display: stream.mastering_display,
            content_light_level: stream.content_light_level,
            dolby_vision: stream.dolby_vision,
            hdr10_plus: stream.hdr10_plus,
        }
    }
}

fn selection_reasons_for_transcode(
    reason: PlaybackDecisionReason,
    report: &PlaybackDecisionReport,
) -> Vec<PlaybackCompatibilityCondition> {
    let mut reasons = Vec::new();
    if let Some(condition) = decision_reason_condition(reason) {
        push_unique_condition(&mut reasons, condition);
    }

    for evaluation in [&report.direct_play, &report.remux, &report.transcode] {
        for condition in &evaluation.reasons {
            if *condition != PlaybackCompatibilityCondition::Compatible {
                push_unique_condition(&mut reasons, *condition);
            }
        }
    }

    reasons
}

fn selection_reasons_for_remux(
    reason: PlaybackDecisionReason,
    report: &PlaybackDecisionReport,
) -> Vec<PlaybackCompatibilityCondition> {
    let mut reasons = Vec::new();
    if let Some(condition) = decision_reason_condition(reason) {
        push_unique_condition(&mut reasons, condition);
    }

    for condition in &report.direct_play.reasons {
        if *condition != PlaybackCompatibilityCondition::Compatible {
            push_unique_condition(&mut reasons, *condition);
        }
    }

    reasons
}

fn push_unique_condition(
    reasons: &mut Vec<PlaybackCompatibilityCondition>,
    reason: PlaybackCompatibilityCondition,
) {
    if !reasons.contains(&reason) {
        reasons.push(reason);
    }
}

fn decision_reason_condition(
    reason: PlaybackDecisionReason,
) -> Option<PlaybackCompatibilityCondition> {
    match reason {
        PlaybackDecisionReason::Compatible => None,
        PlaybackDecisionReason::RequestedTranscodeOutput => {
            Some(PlaybackCompatibilityCondition::RequestedTranscodeOutput)
        }
        PlaybackDecisionReason::ClientDisabledDirectPlay => {
            Some(PlaybackCompatibilityCondition::DirectPlayDisabled)
        }
        PlaybackDecisionReason::SourceContainerUnknown => {
            Some(PlaybackCompatibilityCondition::ContainerUnknown)
        }
        PlaybackDecisionReason::ClientContainerUnsupported => {
            Some(PlaybackCompatibilityCondition::ContainerUnsupported)
        }
        PlaybackDecisionReason::SourceCodecsUnsupported => None,
        PlaybackDecisionReason::PolicyDenied => Some(PlaybackCompatibilityCondition::PolicyDenied),
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

#[cfg(test)]
mod tests {
    use nako_core::{
        MediaColorInfo, MediaHdrMetadata, MediaProbeResult, MediaSource, MediaSourceId,
        MediaStreamDisposition, MediaStreamInfo, MediaStreamKind, MediaStreamTechnicalFacts,
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
        assert_eq!(
            decision.report.selection_reasons,
            vec![PlaybackCompatibilityCondition::Compatible]
        );
        assert_eq!(decision.selected_source.source_id, source.id);
        assert!(matches!(
            decision.rendition,
            PlaybackRenditionPlan::DirectPlay(_)
        ));
        assert_eq!(
            decision.direct_play_plan().unwrap().content_type.as_str(),
            "video/mp4"
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
        assert_eq!(decision.report.selected_mode, PlaybackMode::Remux);
        assert_eq!(
            decision.report.selection_reasons,
            vec![PlaybackCompatibilityCondition::ContainerUnsupported]
        );
        assert!(
            decision
                .report
                .direct_play
                .has(PlaybackCompatibilityCondition::ContainerUnsupported)
        );
        assert!(decision.report.remux.supported);
        assert!(matches!(
            decision.rendition,
            PlaybackRenditionPlan::Remux(RemuxPlaybackPlan {
                output_container: PlaybackRemuxContainer::Mp4,
                ..
            })
        ));
    }

    #[test]
    fn unsupported_container_with_burn_in_subtitle_requirement_does_not_remux() {
        let source = media_source("movie.mkv");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        let mut subtitle = stream(MediaStreamKind::Subtitle, Some("ass"));
        subtitle.index = 2;
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("matroska,webm".to_owned()),
            bit_rate: None,
            streams: vec![video, audio, subtitle],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    requested_subtitle_stream: Some(2),
                    ..PlaybackPreferenceContext::default()
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
            PlaybackDecisionReason::ClientContainerUnsupported
        );
        assert!(
            decision
                .report
                .direct_play
                .has(PlaybackCompatibilityCondition::ContainerUnsupported)
        );
        assert!(
            decision
                .report
                .remux
                .has(PlaybackCompatibilityCondition::SubtitleDeliveryUnsupported)
        );
        assert!(
            decision
                .report
                .selection_reasons
                .contains(&PlaybackCompatibilityCondition::ContainerUnsupported)
        );
        assert!(
            decision
                .report
                .selection_reasons
                .contains(&PlaybackCompatibilityCondition::SubtitleDeliveryUnsupported)
        );

        let requirement = decision
            .transcode_requirement()
            .expect("burn-in-only subtitle delivery should force HLS transcode");
        assert_eq!(
            requirement.subtitle_strategy,
            PlaybackSubtitleStrategy::BurnInSelected
        );
        assert!(
            requirement
                .reasons
                .contains(&PlaybackCompatibilityCondition::ContainerUnsupported)
        );
        assert!(
            requirement
                .reasons
                .contains(&PlaybackCompatibilityCondition::SubtitleDeliveryUnsupported)
        );
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
                    remux_output_container: Some(PlaybackRemuxContainer::Mkv),
                    ..Default::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert!(matches!(
            decision.rendition,
            PlaybackRenditionPlan::Remux(RemuxPlaybackPlan {
                output_container: PlaybackRemuxContainer::Mkv,
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
                    preferred_audio_languages: Vec::new(),
                    requested_subtitle_stream: Some(2),
                    preferred_subtitle_languages: Vec::new(),
                    max_video_bitrate: Some(4_000_000),
                    prefer_hdr: Some(false),
                    remux_output_container: Some(PlaybackRemuxContainer::Mkv),
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
        assert!(!decision.direct_play_plan().unwrap().supports_range_requests);
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
        let profile = PlaybackTargetProfile::from_capabilities(&client, context.clone());
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
                ..ClientPlaybackCapabilities::default()
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
                    transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
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
        assert!(
            decision
                .report
                .transcode
                .has(PlaybackCompatibilityCondition::RequestedTranscodeOutput)
        );
        assert!(matches!(
            decision.rendition,
            PlaybackRenditionPlan::Transcode(TranscodeRenditionPlan {
                plan: PlaybackTranscodePlan {
                    output_container: PlaybackTranscodeContainer::Hls,
                    ..
                },
                ..
            })
        ));
    }

    #[test]
    fn transcode_decision_carries_source_aware_requirement() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("mpeg2video"));
        video.index = 0;
        video.width = Some(3840);
        video.height = Some(2160);
        video.bit_rate = Some(18_000_000);
        video.technical = MediaStreamTechnicalFacts {
            codec_profile: Some("Main 10".to_owned()),
            codec_level: Some(51),
            pixel_format: Some("yuv420p10le".to_owned()),
            bits_per_raw_sample: Some(10),
            color: MediaColorInfo {
                transfer: Some("smpte2084".to_owned()),
                primaries: Some("bt2020".to_owned()),
                ..MediaColorInfo::default()
            },
            hdr: MediaHdrMetadata {
                dynamic_range: Some("hdr10".to_owned()),
                mastering_display: true,
                content_light_level: true,
                ..MediaHdrMetadata::default()
            },
            ..MediaStreamTechnicalFacts::default()
        };
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        audio.channels = Some(6);
        audio.technical = MediaStreamTechnicalFacts {
            channel_layout: Some("5.1".to_owned()),
            bits_per_sample: Some(24),
            ..MediaStreamTechnicalFacts::default()
        };
        let mut subtitle = stream(MediaStreamKind::Subtitle, Some("subrip"));
        subtitle.index = 2;
        subtitle.language = Some("jpn".to_owned());
        subtitle.technical = MediaStreamTechnicalFacts {
            disposition: MediaStreamDisposition {
                forced: true,
                ..MediaStreamDisposition::default()
            },
            ..MediaStreamTechnicalFacts::default()
        };
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: Some(18_500_000),
            streams: vec![video, audio, subtitle],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    requested_audio_stream: Some(1),
                    requested_subtitle_stream: Some(2),
                    max_video_bitrate: Some(8_000_000),
                    prefer_hdr: Some(false),
                    ..PlaybackPreferenceContext::default()
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
            PlaybackDecisionReason::SourceCodecsUnsupported
        );
        let requirement = decision
            .transcode_requirement()
            .expect("transcode decision carries source-aware requirement");
        assert_eq!(
            requirement.output_container,
            PlaybackTranscodeContainer::Hls
        );
        assert_eq!(requirement.output_video_codec.as_deref(), Some("h264"));
        assert_eq!(requirement.output_audio_codec.as_deref(), Some("aac"));
        assert_eq!(requirement.track_selection.audio_stream, Some(1));
        assert_eq!(requirement.track_selection.subtitle_stream, Some(2));
        assert_eq!(
            requirement.output_constraints.max_video_bitrate,
            Some(8_000_000)
        );
        assert_eq!(requirement.output_constraints.prefer_hdr, Some(false));
        assert_eq!(
            requirement.subtitle_strategy,
            PlaybackSubtitleStrategy::SidecarSelected
        );
        assert!(
            requirement
                .reasons
                .contains(&PlaybackCompatibilityCondition::VideoCodecUnsupported)
        );

        let selected_video = requirement.selected_streams.video.as_ref().unwrap();
        assert_eq!(selected_video.codec.as_deref(), Some("mpeg2video"));
        assert_eq!(selected_video.codec_profile.as_deref(), Some("Main 10"));
        assert_eq!(selected_video.pixel_format.as_deref(), Some("yuv420p10le"));
        assert_eq!(selected_video.bits_per_raw_sample, Some(10));
        assert_eq!(selected_video.dynamic_range.as_deref(), Some("hdr10"));
        assert_eq!(
            requirement
                .selected_streams
                .audio
                .as_ref()
                .and_then(|stream| stream.channel_layout.as_deref()),
            Some("5.1")
        );
        assert!(
            requirement
                .selected_streams
                .subtitle
                .as_ref()
                .is_some_and(|stream| stream.forced)
        );
    }

    #[test]
    fn audio_language_preference_selects_first_matching_transcode_stream() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        let mut english = stream(MediaStreamKind::Audio, Some("aac"));
        english.index = 1;
        english.language = Some("eng".to_owned());
        let mut japanese = stream(MediaStreamKind::Audio, Some("aac"));
        japanese.index = 2;
        japanese.language = Some("JPN".to_owned());
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![video, english, japanese],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    preferred_audio_languages: vec![" jpn ".to_owned(), "eng".to_owned()],
                    transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
                    ..PlaybackPreferenceContext::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        let requirement = decision
            .transcode_requirement()
            .expect("preferred audio language should be reflected in transcode requirement");
        assert_eq!(requirement.track_selection.audio_stream, Some(2));
        assert_eq!(
            requirement
                .selected_streams
                .audio
                .as_ref()
                .map(|stream| stream.index),
            Some(2)
        );
    }

    #[test]
    fn requested_audio_stream_overrides_audio_language_preference() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        let mut english = stream(MediaStreamKind::Audio, Some("aac"));
        english.index = 1;
        english.language = Some("eng".to_owned());
        let mut japanese = stream(MediaStreamKind::Audio, Some("aac"));
        japanese.index = 2;
        japanese.language = Some("jpn".to_owned());
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![video, english, japanese],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    requested_audio_stream: Some(1),
                    preferred_audio_languages: vec!["jpn".to_owned()],
                    transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
                    ..PlaybackPreferenceContext::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        let requirement = decision
            .transcode_requirement()
            .expect("explicit audio stream should be reflected in transcode requirement");
        assert_eq!(requirement.track_selection.audio_stream, Some(1));
        assert_eq!(
            requirement
                .selected_streams
                .audio
                .as_ref()
                .map(|stream| stream.index),
            Some(1)
        );
    }

    #[test]
    fn audio_language_preference_falls_back_to_first_audio_without_match() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        let mut english = stream(MediaStreamKind::Audio, Some("aac"));
        english.index = 1;
        english.language = Some("eng".to_owned());
        let mut japanese = stream(MediaStreamKind::Audio, Some("aac"));
        japanese.index = 2;
        japanese.language = Some("jpn".to_owned());
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![video, english, japanese],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    preferred_audio_languages: vec!["fra".to_owned()],
                    transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
                    ..PlaybackPreferenceContext::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        let requirement = decision
            .transcode_requirement()
            .expect("fallback audio should be reflected in transcode requirement");
        assert_eq!(requirement.track_selection.audio_stream, None);
        assert_eq!(
            requirement
                .selected_streams
                .audio
                .as_ref()
                .map(|stream| stream.index),
            Some(1)
        );
    }

    #[test]
    fn subtitle_language_preference_selects_first_matching_transcode_stream() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        let mut english = stream(MediaStreamKind::Subtitle, Some("webvtt"));
        english.index = 2;
        english.language = Some("eng".to_owned());
        let mut japanese = stream(MediaStreamKind::Subtitle, Some("webvtt"));
        japanese.index = 3;
        japanese.language = Some("JPN".to_owned());
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![video, audio, english, japanese],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    preferred_subtitle_languages: vec![" jpn ".to_owned(), "eng".to_owned()],
                    transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
                    ..PlaybackPreferenceContext::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        let requirement = decision
            .transcode_requirement()
            .expect("preferred subtitle language should be reflected in transcode requirement");
        assert_eq!(requirement.track_selection.subtitle_stream, Some(3));
        assert_eq!(
            requirement
                .selected_streams
                .subtitle
                .as_ref()
                .map(|stream| stream.index),
            Some(3)
        );
    }

    #[test]
    fn requested_subtitle_stream_overrides_subtitle_language_preference() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        let mut english = stream(MediaStreamKind::Subtitle, Some("webvtt"));
        english.index = 2;
        english.language = Some("eng".to_owned());
        let mut japanese = stream(MediaStreamKind::Subtitle, Some("webvtt"));
        japanese.index = 3;
        japanese.language = Some("jpn".to_owned());
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![video, audio, english, japanese],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    requested_subtitle_stream: Some(2),
                    preferred_subtitle_languages: vec!["jpn".to_owned()],
                    transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
                    ..PlaybackPreferenceContext::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        let requirement = decision
            .transcode_requirement()
            .expect("explicit subtitle stream should be reflected in transcode requirement");
        assert_eq!(requirement.track_selection.subtitle_stream, Some(2));
        assert_eq!(
            requirement
                .selected_streams
                .subtitle
                .as_ref()
                .map(|stream| stream.index),
            Some(2)
        );
    }

    #[test]
    fn subtitle_language_preference_falls_back_to_first_subtitle_without_match() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        let mut english = stream(MediaStreamKind::Subtitle, Some("webvtt"));
        english.index = 2;
        english.language = Some("eng".to_owned());
        let mut japanese = stream(MediaStreamKind::Subtitle, Some("webvtt"));
        japanese.index = 3;
        japanese.language = Some("jpn".to_owned());
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![video, audio, english, japanese],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    preferred_subtitle_languages: vec!["fra".to_owned()],
                    transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
                    ..PlaybackPreferenceContext::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        let requirement = decision
            .transcode_requirement()
            .expect("fallback subtitle should be reflected in transcode requirement");
        assert_eq!(requirement.track_selection.subtitle_stream, None);
        assert_eq!(
            requirement
                .selected_streams
                .subtitle
                .as_ref()
                .map(|stream| stream.index),
            Some(2)
        );
    }

    #[test]
    fn client_capability_limits_drive_hls_requirement_and_transcode_reasons() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        video.width = Some(3840);
        video.height = Some(2160);
        video.bit_rate = Some(12_000_000);
        video.technical = MediaStreamTechnicalFacts {
            hdr: MediaHdrMetadata {
                dynamic_range: Some("hdr10".to_owned()),
                mastering_display: true,
                content_light_level: true,
                ..MediaHdrMetadata::default()
            },
            ..MediaStreamTechnicalFacts::default()
        };
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        audio.channels = Some(6);
        let mut subtitle = stream(MediaStreamKind::Subtitle, Some("subrip"));
        subtitle.index = 2;
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: Some(13_000_000),
            streams: vec![video, audio, subtitle],
        };
        let client = ClientPlaybackCapabilities {
            max_video_bitrate: Some(8_000_000),
            max_width: Some(1920),
            max_height: Some(1080),
            max_audio_channels: Some(2),
            supports_hdr: false,
            supports_subtitles: false,
            hls_variant_policy: PlaybackHlsVariantPolicy::Adaptive,
            hls_segment_container: PlaybackHlsSegmentContainer::Fmp4,
            ..ClientPlaybackCapabilities::default()
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            client,
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    requested_audio_stream: Some(1),
                    requested_subtitle_stream: Some(2),
                    ..PlaybackPreferenceContext::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert_eq!(decision.mode, PlaybackMode::Transcode);
        for reason in [
            PlaybackCompatibilityCondition::VideoBitrateUnsupported,
            PlaybackCompatibilityCondition::VideoResolutionUnsupported,
            PlaybackCompatibilityCondition::VideoHdrUnsupported,
            PlaybackCompatibilityCondition::AudioChannelsUnsupported,
            PlaybackCompatibilityCondition::SubtitleDeliveryUnsupported,
        ] {
            assert!(
                decision.report.direct_play.has(reason),
                "missing direct-play reason {reason:?}"
            );
            assert!(
                decision.report.selection_reasons.contains(&reason),
                "missing selected decision reason {reason:?}"
            );
        }

        let requirement = decision
            .transcode_requirement()
            .expect("capability-limited playback should transcode");
        assert_eq!(
            requirement.output_constraints.max_video_bitrate,
            Some(8_000_000)
        );
        assert_eq!(requirement.output_constraints.prefer_hdr, Some(false));
        assert_eq!(
            requirement.hls_output,
            Some(PlaybackHlsOutputRequirement {
                variant_policy: PlaybackHlsVariantPolicy::Adaptive,
                segment_container: PlaybackHlsSegmentContainer::Fmp4,
            })
        );
        assert!(
            requirement
                .reasons
                .contains(&PlaybackCompatibilityCondition::VideoHdrUnsupported)
        );
        assert!(
            requirement
                .reasons
                .contains(&PlaybackCompatibilityCondition::SubtitleDeliveryUnsupported)
        );
        assert_eq!(
            requirement.subtitle_strategy,
            PlaybackSubtitleStrategy::BurnInSelected
        );
    }

    #[test]
    fn sidecar_capable_hls_subtitle_format_remains_sidecar_selected() {
        for codec in ["subrip", "vtt", "webvtt"] {
            let source = media_source("movie.mp4");
            let mut video = stream(MediaStreamKind::Video, Some("h264"));
            video.index = 0;
            let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
            audio.index = 1;
            let mut subtitle = stream(MediaStreamKind::Subtitle, Some(codec));
            subtitle.index = 2;
            let probe = MediaProbeResult {
                duration_ms: Some(1_000),
                container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
                bit_rate: None,
                streams: vec![video, audio, subtitle],
            };

            let decision = plan_with_policy(
                &source,
                Some(&probe),
                ClientPlaybackCapabilities::default(),
                PlaybackSelectionContext {
                    storage: PlaybackStorageContext::default(),
                    preferences: PlaybackPreferenceContext {
                        requested_subtitle_stream: Some(2),
                        transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
                        ..PlaybackPreferenceContext::default()
                    },
                },
                EffectivePlaybackPolicy::from_library_access(
                    source.library_id,
                    nako_core::LibraryAccessLevel::Play,
                ),
            );

            let requirement = decision
                .transcode_requirement()
                .expect("requested HLS transcode should carry subtitle strategy");

            assert_eq!(
                requirement.subtitle_strategy,
                PlaybackSubtitleStrategy::SidecarSelected,
                "codec {codec} should remain sidecar selected"
            );
            assert_eq!(requirement.track_selection.subtitle_stream, Some(2));
        }
    }

    #[test]
    fn unknown_hls_subtitle_codec_facts_preserve_legacy_sidecar_strategy() {
        for codec in [None, Some("   ")] {
            let source = media_source("movie.mp4");
            let mut video = stream(MediaStreamKind::Video, Some("h264"));
            video.index = 0;
            let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
            audio.index = 1;
            let mut subtitle = stream(MediaStreamKind::Subtitle, codec);
            subtitle.index = 2;
            let probe = MediaProbeResult {
                duration_ms: Some(1_000),
                container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
                bit_rate: None,
                streams: vec![video, audio, subtitle],
            };

            let decision = plan_with_policy(
                &source,
                Some(&probe),
                ClientPlaybackCapabilities::default(),
                PlaybackSelectionContext {
                    storage: PlaybackStorageContext::default(),
                    preferences: PlaybackPreferenceContext {
                        requested_subtitle_stream: Some(2),
                        transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
                        ..PlaybackPreferenceContext::default()
                    },
                },
                EffectivePlaybackPolicy::from_library_access(
                    source.library_id,
                    nako_core::LibraryAccessLevel::Play,
                ),
            );

            assert!(
                !decision
                    .report
                    .direct_play
                    .has(PlaybackCompatibilityCondition::SubtitleDeliveryUnsupported),
                "unknown codec {codec:?} should not be treated as burn-in-only"
            );
            let requirement = decision
                .transcode_requirement()
                .expect("requested HLS transcode should carry subtitle strategy");
            assert_eq!(
                requirement.subtitle_strategy,
                PlaybackSubtitleStrategy::SidecarSelected,
                "unknown codec {codec:?} should preserve legacy sidecar behavior"
            );
        }
    }

    #[test]
    fn unsupported_hls_subtitle_format_requires_burn_in_strategy() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        let mut subtitle = stream(MediaStreamKind::Subtitle, Some("ass"));
        subtitle.index = 2;
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![video, audio, subtitle],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    requested_subtitle_stream: Some(2),
                    ..PlaybackPreferenceContext::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert_eq!(decision.mode, PlaybackMode::Transcode);
        assert!(
            decision
                .report
                .direct_play
                .has(PlaybackCompatibilityCondition::SubtitleDeliveryUnsupported)
        );
        let requirement = decision
            .transcode_requirement()
            .expect("unsupported subtitle delivery should require transcode planning");

        assert_eq!(
            requirement.subtitle_strategy,
            PlaybackSubtitleStrategy::BurnInSelected
        );
        assert!(
            requirement
                .reasons
                .contains(&PlaybackCompatibilityCondition::SubtitleDeliveryUnsupported)
        );
        assert_eq!(
            requirement
                .selected_streams
                .subtitle
                .as_ref()
                .and_then(|stream| stream.codec.as_deref()),
            Some("ass")
        );
    }

    #[test]
    fn hdr_limited_client_gets_color_pipeline_tone_mapping_requirement() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        video.technical = MediaStreamTechnicalFacts {
            color: MediaColorInfo {
                space: Some("bt2020nc".to_owned()),
                transfer: Some("smpte2084".to_owned()),
                primaries: Some("bt2020".to_owned()),
                ..MediaColorInfo::default()
            },
            hdr: MediaHdrMetadata {
                dynamic_range: Some("hdr10".to_owned()),
                mastering_display: true,
                content_light_level: true,
                ..MediaHdrMetadata::default()
            },
            ..MediaStreamTechnicalFacts::default()
        };
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![video, audio],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities {
                supports_hdr: false,
                ..ClientPlaybackCapabilities::default()
            },
            PlaybackSelectionContext::default(),
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert_eq!(decision.mode, PlaybackMode::Transcode);
        assert!(
            decision
                .report
                .direct_play
                .has(PlaybackCompatibilityCondition::VideoHdrUnsupported)
        );
        assert_eq!(
            decision
                .transcode_requirement()
                .expect("HDR-limited client should transcode")
                .color_pipeline,
            PlaybackColorPipelineRequirement {
                source: Some(PlaybackColorPipelineSource {
                    dynamic_range: Some("hdr10".to_owned()),
                    color_space: Some("bt2020nc".to_owned()),
                    color_transfer: Some("smpte2084".to_owned()),
                    color_primaries: Some("bt2020".to_owned()),
                    mastering_display: true,
                    content_light_level: true,
                    dolby_vision: false,
                    hdr10_plus: false,
                }),
                target: PlaybackColorPipelineTarget::Sdr,
                tone_mapping: PlaybackHdrToneMappingRequirement::Required,
                reasons: vec![
                    PlaybackColorCompatibilityReason::SourceHdrDetected,
                    PlaybackColorCompatibilityReason::ClientHdrUnsupported,
                    PlaybackColorCompatibilityReason::ToneMappingRequired,
                ],
            }
        );
    }

    #[test]
    fn hdr_limited_client_does_not_remux_when_tone_mapping_is_required() {
        let source = media_source("movie.mkv");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        video.technical = MediaStreamTechnicalFacts {
            color: MediaColorInfo {
                space: Some("bt2020nc".to_owned()),
                transfer: Some("smpte2084".to_owned()),
                primaries: Some("bt2020".to_owned()),
                ..MediaColorInfo::default()
            },
            hdr: MediaHdrMetadata {
                dynamic_range: Some("hdr10".to_owned()),
                mastering_display: true,
                content_light_level: true,
                ..MediaHdrMetadata::default()
            },
            ..MediaStreamTechnicalFacts::default()
        };
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("matroska,webm".to_owned()),
            bit_rate: None,
            streams: vec![video, audio],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities {
                supports_hdr: false,
                ..ClientPlaybackCapabilities::default()
            },
            PlaybackSelectionContext::default(),
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert_eq!(decision.mode, PlaybackMode::Transcode);
        assert!(
            decision
                .report
                .remux
                .has(PlaybackCompatibilityCondition::VideoHdrUnsupported)
        );
        assert_eq!(
            decision
                .transcode_requirement()
                .expect("HDR remux cannot satisfy SDR client tone mapping")
                .color_pipeline
                .tone_mapping,
            PlaybackHdrToneMappingRequirement::Required
        );
    }

    #[test]
    fn hdr_capable_client_preserves_source_color_when_transcode_is_requested() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        video.technical = MediaStreamTechnicalFacts {
            color: MediaColorInfo {
                transfer: Some("arib-std-b67".to_owned()),
                primaries: Some("bt2020".to_owned()),
                ..MediaColorInfo::default()
            },
            hdr: MediaHdrMetadata {
                dynamic_range: Some("hlg".to_owned()),
                ..MediaHdrMetadata::default()
            },
            ..MediaStreamTechnicalFacts::default()
        };
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![video, audio],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
                    ..PlaybackPreferenceContext::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert_eq!(decision.mode, PlaybackMode::Transcode);
        assert_eq!(
            decision
                .transcode_requirement()
                .expect("requested transcode should carry color requirement")
                .color_pipeline,
            PlaybackColorPipelineRequirement {
                source: Some(PlaybackColorPipelineSource {
                    dynamic_range: Some("hlg".to_owned()),
                    color_space: None,
                    color_transfer: Some("arib-std-b67".to_owned()),
                    color_primaries: Some("bt2020".to_owned()),
                    mastering_display: false,
                    content_light_level: false,
                    dolby_vision: false,
                    hdr10_plus: false,
                }),
                target: PlaybackColorPipelineTarget::PreserveSource,
                tone_mapping: PlaybackHdrToneMappingRequirement::None,
                reasons: vec![
                    PlaybackColorCompatibilityReason::SourceHdrDetected,
                    PlaybackColorCompatibilityReason::HdrPassthroughSupported,
                ],
            }
        );
    }

    #[test]
    fn dynamic_hdr_for_sdr_client_marks_color_pipeline_deferred() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        video.technical = MediaStreamTechnicalFacts {
            hdr: MediaHdrMetadata {
                dolby_vision: true,
                ..MediaHdrMetadata::default()
            },
            ..MediaStreamTechnicalFacts::default()
        };
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![video, audio],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities {
                supports_hdr: false,
                ..ClientPlaybackCapabilities::default()
            },
            PlaybackSelectionContext::default(),
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert_eq!(decision.mode, PlaybackMode::Transcode);
        assert_eq!(
            decision
                .transcode_requirement()
                .expect("dynamic HDR should transcode for an SDR client")
                .color_pipeline,
            PlaybackColorPipelineRequirement {
                source: Some(PlaybackColorPipelineSource {
                    dynamic_range: None,
                    color_space: None,
                    color_transfer: None,
                    color_primaries: None,
                    mastering_display: false,
                    content_light_level: false,
                    dolby_vision: true,
                    hdr10_plus: false,
                }),
                target: PlaybackColorPipelineTarget::Sdr,
                tone_mapping: PlaybackHdrToneMappingRequirement::DeferredUnsupported,
                reasons: vec![
                    PlaybackColorCompatibilityReason::SourceHdrDetected,
                    PlaybackColorCompatibilityReason::ClientHdrUnsupported,
                    PlaybackColorCompatibilityReason::UnsupportedHdrFormatDeferred,
                ],
            }
        );
    }

    #[test]
    fn audio_channel_limit_drives_downmix_output_requirement() {
        let source = media_source("movie.mp4");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        audio.channels = Some(6);
        audio.technical = MediaStreamTechnicalFacts {
            channel_layout: Some("5.1".to_owned()),
            ..MediaStreamTechnicalFacts::default()
        };
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![video, audio],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities {
                max_audio_channels: Some(2),
                ..ClientPlaybackCapabilities::default()
            },
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    requested_audio_stream: Some(1),
                    ..PlaybackPreferenceContext::default()
                },
            },
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert_eq!(decision.mode, PlaybackMode::Transcode);
        assert!(
            decision
                .report
                .direct_play
                .has(PlaybackCompatibilityCondition::AudioChannelsUnsupported)
        );

        let requirement = decision
            .transcode_requirement()
            .expect("channel-limited playback should transcode");
        assert_eq!(
            requirement.audio_output,
            PlaybackAudioOutputRequirement {
                source_channels: Some(6),
                max_supported_channels: Some(2),
                target_channels: Some(2),
                downmix: PlaybackAudioDownmixRequirement::Required,
                normalization: PlaybackAudioNormalizationRequirement::None,
                reasons: vec![
                    PlaybackAudioCompatibilityReason::ChannelLimitExceeded,
                    PlaybackAudioCompatibilityReason::DownmixRequired,
                ],
            }
        );
    }

    #[test]
    fn remux_is_not_selected_when_audio_downmix_is_required() {
        let source = media_source("movie.mkv");
        let mut video = stream(MediaStreamKind::Video, Some("h264"));
        video.index = 0;
        let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
        audio.index = 1;
        audio.channels = Some(8);
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("matroska,webm".to_owned()),
            bit_rate: None,
            streams: vec![video, audio],
        };

        let decision = plan_with_policy(
            &source,
            Some(&probe),
            ClientPlaybackCapabilities {
                max_audio_channels: Some(2),
                ..ClientPlaybackCapabilities::default()
            },
            PlaybackSelectionContext::default(),
            EffectivePlaybackPolicy::from_library_access(
                source.library_id,
                nako_core::LibraryAccessLevel::Play,
            ),
        );

        assert_eq!(decision.mode, PlaybackMode::Transcode);
        assert!(!decision.report.remux.supported);
        assert!(
            decision
                .report
                .remux
                .has(PlaybackCompatibilityCondition::AudioChannelsUnsupported)
        );
        assert_eq!(
            decision
                .transcode_requirement()
                .expect("downmix should force transcode")
                .audio_output
                .target_channels,
            Some(2)
        );
    }

    #[test]
    fn playback_compatibility_matrix_covers_direct_play_remux_hls_transcode_hdr_and_audio() {
        for case in playback_compatibility_matrix_cases() {
            let source = media_source(case.file_name);
            let probe = matrix_probe(
                case.probe_container,
                case.video_codec,
                case.hdr,
                case.audio_codec,
                case.audio_channels,
            );

            let decision = plan_with_policy(
                &source,
                Some(&probe),
                case.client,
                case.context,
                EffectivePlaybackPolicy::from_library_access(
                    source.library_id,
                    nako_core::LibraryAccessLevel::Play,
                ),
            );

            assert_eq!(decision.mode, case.expected_mode, "{}", case.name);
            assert_eq!(decision.reason, case.expected_reason, "{}", case.name);
            assert_eq!(
                decision.report.selected_mode, case.expected_mode,
                "{}",
                case.name
            );
            assert_eq!(
                decision.report.direct_play.supported, case.expected_direct_supported,
                "{}",
                case.name
            );
            assert_eq!(
                decision.report.remux.supported, case.expected_remux_supported,
                "{}",
                case.name
            );

            for condition in case.expected_direct_conditions {
                assert!(
                    decision.report.direct_play.has(condition),
                    "{} missing direct-play condition {condition:?}",
                    case.name
                );
            }
            for condition in case.expected_remux_conditions {
                assert!(
                    decision.report.remux.has(condition),
                    "{} missing remux condition {condition:?}",
                    case.name
                );
            }
            for condition in case.expected_selection_conditions {
                assert!(
                    decision.report.selection_reasons.contains(&condition),
                    "{} missing selected decision condition {condition:?}",
                    case.name
                );
            }

            match case.expected_transcode {
                Some(expected) => {
                    let requirement = decision
                        .transcode_requirement()
                        .expect("matrix transcode case should carry requirement");
                    assert_eq!(
                        requirement.output_container, expected.output_container,
                        "{}",
                        case.name
                    );
                    assert_eq!(requirement.hls_output, expected.hls_output, "{}", case.name);
                    assert_eq!(
                        requirement.audio_output.target_channels, expected.audio_target_channels,
                        "{}",
                        case.name
                    );
                    assert_eq!(
                        requirement.audio_output.downmix, expected.audio_downmix,
                        "{}",
                        case.name
                    );
                    assert_eq!(
                        requirement.audio_output.normalization, expected.audio_normalization,
                        "{}",
                        case.name
                    );
                    for reason in expected.audio_reasons {
                        assert!(
                            requirement.audio_output.reasons.contains(&reason),
                            "{} missing audio output reason {reason:?}",
                            case.name
                        );
                    }
                    assert_eq!(
                        requirement.color_pipeline.target, expected.color_target,
                        "{}",
                        case.name
                    );
                    assert_eq!(
                        requirement.color_pipeline.tone_mapping, expected.tone_mapping,
                        "{}",
                        case.name
                    );
                }
                None => {
                    assert!(
                        decision.transcode_requirement().is_none(),
                        "{} should not carry a transcode requirement",
                        case.name
                    );
                }
            }
        }
    }

    #[test]
    fn playback_compatibility_matrix_audio_output_requirements_cover_downmix_and_normalization() {
        for case in audio_output_requirement_matrix_cases() {
            let requirement = PlaybackAudioOutputRequirement::from_channel_support(
                case.source_channels,
                case.max_supported_channels,
            )
            .with_normalization(case.normalization);

            assert_eq!(
                requirement.target_channels, case.expected_target_channels,
                "{}",
                case.name
            );
            assert_eq!(requirement.downmix, case.expected_downmix, "{}", case.name);
            assert_eq!(
                requirement.normalization, case.normalization,
                "{}",
                case.name
            );
            assert_eq!(requirement.reasons, case.expected_reasons, "{}", case.name);
        }
    }

    #[test]
    fn remux_compatibility_matrix_preserves_video_output_constraints() {
        for case in remux_video_output_constraint_cases() {
            let source = media_source("movie.mkv");
            let mut video = stream(MediaStreamKind::Video, Some("h264"));
            video.index = 0;
            video.bit_rate = case.video_bitrate;
            video.width = case.video_width;
            video.height = case.video_height;
            let mut audio = stream(MediaStreamKind::Audio, Some("aac"));
            audio.index = 1;
            let probe = MediaProbeResult {
                duration_ms: Some(1_000),
                container: Some("matroska,webm".to_owned()),
                bit_rate: None,
                streams: vec![video, audio],
            };

            let decision = plan_with_policy(
                &source,
                Some(&probe),
                case.client,
                case.context,
                EffectivePlaybackPolicy::from_library_access(
                    source.library_id,
                    nako_core::LibraryAccessLevel::Play,
                ),
            );

            assert_eq!(decision.mode, PlaybackMode::Transcode, "{}", case.name);
            assert_eq!(
                decision.reason,
                PlaybackDecisionReason::ClientContainerUnsupported,
                "{}",
                case.name
            );
            assert!(
                decision
                    .report
                    .direct_play
                    .has(PlaybackCompatibilityCondition::ContainerUnsupported),
                "{} missing direct-play container reason",
                case.name
            );
            assert!(
                decision.report.remux.has(case.expected_remux_condition),
                "{} missing remux condition {:?}",
                case.name,
                case.expected_remux_condition
            );

            let requirement = decision
                .transcode_requirement()
                .expect("remux constraint failure should fall back to transcode");
            assert!(
                requirement
                    .reasons
                    .contains(&PlaybackCompatibilityCondition::ContainerUnsupported),
                "{} missing source container reason in requirement",
                case.name
            );
            assert!(
                requirement.reasons.contains(&case.expected_remux_condition),
                "{} missing remux blocker in requirement {:?}",
                case.name,
                case.expected_remux_condition
            );
        }
    }

    #[test]
    fn playback_target_profile_identity_normalizes_capability_order_and_case() {
        let left = PlaybackTargetProfile::from_capabilities(
            &ClientPlaybackCapabilities {
                direct_play: true,
                containers: vec!["MP4".to_owned(), "webm".to_owned(), "mp4".to_owned()],
                video_codecs: vec!["H264".to_owned(), "hevc".to_owned()],
                audio_codecs: vec!["AAC".to_owned(), "opus".to_owned()],
                ..ClientPlaybackCapabilities::default()
            },
            PlaybackSelectionContext {
                storage: PlaybackStorageContext {
                    remote: true,
                    range_readable: Some(false),
                },
                preferences: PlaybackPreferenceContext {
                    requested_audio_stream: Some(2),
                    preferred_audio_languages: Vec::new(),
                    requested_subtitle_stream: None,
                    preferred_subtitle_languages: Vec::new(),
                    max_video_bitrate: Some(8_000_000),
                    prefer_hdr: Some(true),
                    remux_output_container: Some(PlaybackRemuxContainer::Mp4),
                    transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
                },
            },
        );
        let right = PlaybackTargetProfile::from_capabilities(
            &ClientPlaybackCapabilities {
                direct_play: true,
                containers: vec!["webm".to_owned(), "mp4".to_owned()],
                video_codecs: vec!["hevc".to_owned(), "h264".to_owned()],
                audio_codecs: vec!["opus".to_owned(), "aac".to_owned()],
                ..ClientPlaybackCapabilities::default()
            },
            PlaybackSelectionContext {
                storage: PlaybackStorageContext {
                    remote: true,
                    range_readable: Some(false),
                },
                preferences: PlaybackPreferenceContext {
                    requested_audio_stream: Some(2),
                    preferred_audio_languages: Vec::new(),
                    requested_subtitle_stream: None,
                    preferred_subtitle_languages: Vec::new(),
                    max_video_bitrate: Some(8_000_000),
                    prefer_hdr: Some(true),
                    remux_output_container: Some(PlaybackRemuxContainer::Mp4),
                    transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
                },
            },
        );

        assert_eq!(left.identity_key(), right.identity_key());
        assert!(left.identity_key().contains("containers=mp4|webm"));
        assert!(left.identity_key().contains("audio=2"));
        assert!(left.identity_key().contains("hls_variant=single_variant"));
        assert!(left.identity_key().contains("hls_segment=mpeg_ts"));
        assert!(
            left.identity_key()
                .contains("transcode=container=hls,vcodec=h264,acodec=aac")
        );
    }

    #[test]
    fn playback_target_profile_identity_normalizes_audio_language_preferences() {
        let left = PlaybackTargetProfile::from_capabilities(
            &ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    preferred_audio_languages: vec![
                        " JPN ".to_owned(),
                        "eng".to_owned(),
                        "jpn".to_owned(),
                    ],
                    ..PlaybackPreferenceContext::default()
                },
            },
        );
        let right = PlaybackTargetProfile::from_capabilities(
            &ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    preferred_audio_languages: vec!["jpn".to_owned(), "ENG".to_owned()],
                    ..PlaybackPreferenceContext::default()
                },
            },
        );

        assert_eq!(left.identity_key(), right.identity_key());
        assert!(left.identity_key().contains("audio_languages=jpn|eng"));
    }

    #[test]
    fn playback_target_profile_identity_normalizes_subtitle_language_preferences() {
        let left = PlaybackTargetProfile::from_capabilities(
            &ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    preferred_subtitle_languages: vec![
                        " JPN ".to_owned(),
                        "eng".to_owned(),
                        "jpn".to_owned(),
                    ],
                    ..PlaybackPreferenceContext::default()
                },
            },
        );
        let right = PlaybackTargetProfile::from_capabilities(
            &ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    preferred_subtitle_languages: vec!["jpn".to_owned(), "ENG".to_owned()],
                    ..PlaybackPreferenceContext::default()
                },
            },
        );

        assert_eq!(left.identity_key(), right.identity_key());
        assert!(left.identity_key().contains("subtitle_languages=jpn|eng"));
    }

    #[test]
    fn playback_target_profile_identity_includes_capability_profiles() {
        let profile = PlaybackTargetProfile::from_capabilities(
            &ClientPlaybackCapabilities {
                direct_play: true,
                containers: vec!["MP4".to_owned(), "mkv".to_owned()],
                video_codecs: vec!["H264".to_owned()],
                audio_codecs: vec!["AAC".to_owned()],
                ..ClientPlaybackCapabilities::default()
            },
            PlaybackSelectionContext {
                storage: PlaybackStorageContext {
                    remote: false,
                    range_readable: Some(true),
                },
                preferences: PlaybackPreferenceContext {
                    max_video_bitrate: Some(5_000_000),
                    ..Default::default()
                },
            },
        );

        let identity = profile.identity_key();

        assert!(identity.contains("playback-target-profile:v1"));
        assert!(identity.contains("containers=mkv|mp4"));
        assert!(identity.contains("vcodecs=h264"));
        assert!(identity.contains("maxv=5000000"));
        assert!(identity.contains("transcode=container=hls,vcodec=h264,acodec=aac"));
    }

    #[test]
    fn planner_reports_video_codec_mismatch_before_transcode() {
        let source = media_source("movie.mp4");
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![
                stream(MediaStreamKind::Video, Some("mpeg2video")),
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

        assert_eq!(decision.mode, PlaybackMode::Transcode);
        assert_eq!(decision.report.selected_mode, PlaybackMode::Transcode);
        assert!(
            decision
                .report
                .direct_play
                .has(PlaybackCompatibilityCondition::VideoCodecUnsupported)
        );
        assert!(
            decision
                .report
                .selection_reasons
                .contains(&PlaybackCompatibilityCondition::VideoCodecUnsupported)
        );
        assert!(decision.report.transcode.supported);
    }

    #[test]
    fn playback_target_profile_keeps_transcode_planning_facts() {
        let profile = PlaybackTargetProfile::from_capabilities(
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

        assert_eq!(
            profile.track_selection(),
            PlaybackTrackSelection {
                audio_stream: None,
                subtitle_stream: Some(2),
            }
        );
        assert_eq!(
            profile.output_constraints().max_video_bitrate,
            Some(8_000_000)
        );
        assert_eq!(profile.output_constraints().prefer_hdr, Some(true));
        assert_eq!(
            profile.hls_output_requirement(),
            PlaybackHlsOutputRequirement::default()
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

    struct PlaybackCompatibilityMatrixCase {
        name: &'static str,
        file_name: &'static str,
        probe_container: &'static str,
        video_codec: &'static str,
        hdr: MatrixHdrInput,
        audio_codec: &'static str,
        audio_channels: Option<u32>,
        client: ClientPlaybackCapabilities,
        context: PlaybackSelectionContext,
        expected_mode: PlaybackMode,
        expected_reason: PlaybackDecisionReason,
        expected_direct_supported: bool,
        expected_direct_conditions: Vec<PlaybackCompatibilityCondition>,
        expected_remux_supported: bool,
        expected_remux_conditions: Vec<PlaybackCompatibilityCondition>,
        expected_selection_conditions: Vec<PlaybackCompatibilityCondition>,
        expected_transcode: Option<ExpectedTranscodeMatrixRequirement>,
    }

    struct ExpectedTranscodeMatrixRequirement {
        output_container: PlaybackTranscodeContainer,
        hls_output: Option<PlaybackHlsOutputRequirement>,
        audio_target_channels: Option<u32>,
        audio_downmix: PlaybackAudioDownmixRequirement,
        audio_normalization: PlaybackAudioNormalizationRequirement,
        audio_reasons: Vec<PlaybackAudioCompatibilityReason>,
        color_target: PlaybackColorPipelineTarget,
        tone_mapping: PlaybackHdrToneMappingRequirement,
    }

    #[derive(Clone, Copy)]
    enum MatrixHdrInput {
        Sdr,
        Hdr10,
    }

    fn playback_compatibility_matrix_cases() -> Vec<PlaybackCompatibilityMatrixCase> {
        vec![
            PlaybackCompatibilityMatrixCase {
                name: "direct play for mp4 h264 aac source",
                file_name: "movie.mp4",
                probe_container: "mov,mp4,m4a,3gp,3g2,mj2",
                video_codec: "h264",
                hdr: MatrixHdrInput::Sdr,
                audio_codec: "aac",
                audio_channels: Some(2),
                client: ClientPlaybackCapabilities::default(),
                context: PlaybackSelectionContext::default(),
                expected_mode: PlaybackMode::DirectPlay,
                expected_reason: PlaybackDecisionReason::Compatible,
                expected_direct_supported: true,
                expected_direct_conditions: vec![PlaybackCompatibilityCondition::Compatible],
                expected_remux_supported: true,
                expected_remux_conditions: vec![PlaybackCompatibilityCondition::Compatible],
                expected_selection_conditions: vec![PlaybackCompatibilityCondition::Compatible],
                expected_transcode: None,
            },
            PlaybackCompatibilityMatrixCase {
                name: "remux for unsupported source container with compatible streams",
                file_name: "movie.mkv",
                probe_container: "matroska,webm",
                video_codec: "h264",
                hdr: MatrixHdrInput::Sdr,
                audio_codec: "aac",
                audio_channels: Some(2),
                client: ClientPlaybackCapabilities::default(),
                context: PlaybackSelectionContext::default(),
                expected_mode: PlaybackMode::Remux,
                expected_reason: PlaybackDecisionReason::ClientContainerUnsupported,
                expected_direct_supported: false,
                expected_direct_conditions: vec![
                    PlaybackCompatibilityCondition::ContainerUnsupported,
                ],
                expected_remux_supported: true,
                expected_remux_conditions: vec![PlaybackCompatibilityCondition::Compatible],
                expected_selection_conditions: vec![
                    PlaybackCompatibilityCondition::ContainerUnsupported,
                ],
                expected_transcode: None,
            },
            PlaybackCompatibilityMatrixCase {
                name: "hls transcode for unsupported video codec",
                file_name: "movie.mp4",
                probe_container: "mov,mp4,m4a,3gp,3g2,mj2",
                video_codec: "mpeg2video",
                hdr: MatrixHdrInput::Sdr,
                audio_codec: "aac",
                audio_channels: Some(2),
                client: ClientPlaybackCapabilities::default(),
                context: PlaybackSelectionContext::default(),
                expected_mode: PlaybackMode::Transcode,
                expected_reason: PlaybackDecisionReason::SourceCodecsUnsupported,
                expected_direct_supported: false,
                expected_direct_conditions: vec![
                    PlaybackCompatibilityCondition::VideoCodecUnsupported,
                ],
                expected_remux_supported: false,
                expected_remux_conditions: vec![
                    PlaybackCompatibilityCondition::VideoCodecUnsupported,
                ],
                expected_selection_conditions: vec![
                    PlaybackCompatibilityCondition::VideoCodecUnsupported,
                ],
                expected_transcode: Some(default_hls_requirement_expectation()),
            },
            PlaybackCompatibilityMatrixCase {
                name: "hls transcode for unsupported audio codec",
                file_name: "movie.mp4",
                probe_container: "mov,mp4,m4a,3gp,3g2,mj2",
                video_codec: "h264",
                hdr: MatrixHdrInput::Sdr,
                audio_codec: "flac",
                audio_channels: Some(2),
                client: ClientPlaybackCapabilities::default(),
                context: PlaybackSelectionContext::default(),
                expected_mode: PlaybackMode::Transcode,
                expected_reason: PlaybackDecisionReason::SourceCodecsUnsupported,
                expected_direct_supported: false,
                expected_direct_conditions: vec![
                    PlaybackCompatibilityCondition::AudioCodecUnsupported,
                ],
                expected_remux_supported: false,
                expected_remux_conditions: vec![
                    PlaybackCompatibilityCondition::AudioCodecUnsupported,
                ],
                expected_selection_conditions: vec![
                    PlaybackCompatibilityCondition::AudioCodecUnsupported,
                ],
                expected_transcode: Some(default_hls_requirement_expectation()),
            },
            PlaybackCompatibilityMatrixCase {
                name: "hls transcode instead of remux for hdr source on sdr client",
                file_name: "movie.mkv",
                probe_container: "matroska,webm",
                video_codec: "h264",
                hdr: MatrixHdrInput::Hdr10,
                audio_codec: "aac",
                audio_channels: Some(2),
                client: ClientPlaybackCapabilities {
                    supports_hdr: false,
                    ..ClientPlaybackCapabilities::default()
                },
                context: PlaybackSelectionContext::default(),
                expected_mode: PlaybackMode::Transcode,
                expected_reason: PlaybackDecisionReason::ClientContainerUnsupported,
                expected_direct_supported: false,
                expected_direct_conditions: vec![
                    PlaybackCompatibilityCondition::ContainerUnsupported,
                ],
                expected_remux_supported: false,
                expected_remux_conditions: vec![
                    PlaybackCompatibilityCondition::VideoHdrUnsupported,
                ],
                expected_selection_conditions: vec![
                    PlaybackCompatibilityCondition::ContainerUnsupported,
                    PlaybackCompatibilityCondition::VideoHdrUnsupported,
                ],
                expected_transcode: Some(ExpectedTranscodeMatrixRequirement {
                    color_target: PlaybackColorPipelineTarget::Sdr,
                    tone_mapping: PlaybackHdrToneMappingRequirement::Required,
                    ..default_hls_requirement_expectation()
                }),
            },
            PlaybackCompatibilityMatrixCase {
                name: "hls transcode instead of remux when audio downmix is required",
                file_name: "movie.mkv",
                probe_container: "matroska,webm",
                video_codec: "h264",
                hdr: MatrixHdrInput::Sdr,
                audio_codec: "aac",
                audio_channels: Some(6),
                client: ClientPlaybackCapabilities {
                    max_audio_channels: Some(2),
                    ..ClientPlaybackCapabilities::default()
                },
                context: PlaybackSelectionContext::default(),
                expected_mode: PlaybackMode::Transcode,
                expected_reason: PlaybackDecisionReason::ClientContainerUnsupported,
                expected_direct_supported: false,
                expected_direct_conditions: vec![
                    PlaybackCompatibilityCondition::ContainerUnsupported,
                ],
                expected_remux_supported: false,
                expected_remux_conditions: vec![
                    PlaybackCompatibilityCondition::AudioChannelsUnsupported,
                ],
                expected_selection_conditions: vec![
                    PlaybackCompatibilityCondition::ContainerUnsupported,
                    PlaybackCompatibilityCondition::AudioChannelsUnsupported,
                ],
                expected_transcode: Some(ExpectedTranscodeMatrixRequirement {
                    audio_target_channels: Some(2),
                    audio_downmix: PlaybackAudioDownmixRequirement::Required,
                    audio_reasons: vec![
                        PlaybackAudioCompatibilityReason::ChannelLimitExceeded,
                        PlaybackAudioCompatibilityReason::DownmixRequired,
                    ],
                    ..default_hls_requirement_expectation()
                }),
            },
            PlaybackCompatibilityMatrixCase {
                name: "requested hls transcode carries selected hls output shape",
                file_name: "movie.mp4",
                probe_container: "mov,mp4,m4a,3gp,3g2,mj2",
                video_codec: "h264",
                hdr: MatrixHdrInput::Sdr,
                audio_codec: "aac",
                audio_channels: Some(2),
                client: ClientPlaybackCapabilities {
                    hls_variant_policy: PlaybackHlsVariantPolicy::Adaptive,
                    hls_segment_container: PlaybackHlsSegmentContainer::Fmp4,
                    ..ClientPlaybackCapabilities::default()
                },
                context: PlaybackSelectionContext {
                    storage: PlaybackStorageContext::default(),
                    preferences: PlaybackPreferenceContext {
                        transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
                        ..PlaybackPreferenceContext::default()
                    },
                },
                expected_mode: PlaybackMode::Transcode,
                expected_reason: PlaybackDecisionReason::RequestedTranscodeOutput,
                expected_direct_supported: true,
                expected_direct_conditions: vec![PlaybackCompatibilityCondition::Compatible],
                expected_remux_supported: true,
                expected_remux_conditions: vec![PlaybackCompatibilityCondition::Compatible],
                expected_selection_conditions: vec![
                    PlaybackCompatibilityCondition::RequestedTranscodeOutput,
                ],
                expected_transcode: Some(ExpectedTranscodeMatrixRequirement {
                    hls_output: Some(PlaybackHlsOutputRequirement {
                        variant_policy: PlaybackHlsVariantPolicy::Adaptive,
                        segment_container: PlaybackHlsSegmentContainer::Fmp4,
                    }),
                    ..default_hls_requirement_expectation()
                }),
            },
        ]
    }

    fn default_hls_requirement_expectation() -> ExpectedTranscodeMatrixRequirement {
        ExpectedTranscodeMatrixRequirement {
            output_container: PlaybackTranscodeContainer::Hls,
            hls_output: Some(PlaybackHlsOutputRequirement::default()),
            audio_target_channels: None,
            audio_downmix: PlaybackAudioDownmixRequirement::None,
            audio_normalization: PlaybackAudioNormalizationRequirement::None,
            audio_reasons: Vec::new(),
            color_target: PlaybackColorPipelineTarget::PreserveSource,
            tone_mapping: PlaybackHdrToneMappingRequirement::None,
        }
    }

    struct AudioOutputRequirementMatrixCase {
        name: &'static str,
        source_channels: Option<u32>,
        max_supported_channels: Option<u32>,
        normalization: PlaybackAudioNormalizationRequirement,
        expected_target_channels: Option<u32>,
        expected_downmix: PlaybackAudioDownmixRequirement,
        expected_reasons: Vec<PlaybackAudioCompatibilityReason>,
    }

    fn audio_output_requirement_matrix_cases() -> Vec<AudioOutputRequirementMatrixCase> {
        vec![
            AudioOutputRequirementMatrixCase {
                name: "stereo passthrough",
                source_channels: Some(2),
                max_supported_channels: Some(2),
                normalization: PlaybackAudioNormalizationRequirement::None,
                expected_target_channels: None,
                expected_downmix: PlaybackAudioDownmixRequirement::None,
                expected_reasons: Vec::new(),
            },
            AudioOutputRequirementMatrixCase {
                name: "surround downmix",
                source_channels: Some(6),
                max_supported_channels: Some(2),
                normalization: PlaybackAudioNormalizationRequirement::None,
                expected_target_channels: Some(2),
                expected_downmix: PlaybackAudioDownmixRequirement::Required,
                expected_reasons: vec![
                    PlaybackAudioCompatibilityReason::ChannelLimitExceeded,
                    PlaybackAudioCompatibilityReason::DownmixRequired,
                ],
            },
            AudioOutputRequirementMatrixCase {
                name: "normalization request",
                source_channels: Some(2),
                max_supported_channels: Some(2),
                normalization: PlaybackAudioNormalizationRequirement::Requested,
                expected_target_channels: None,
                expected_downmix: PlaybackAudioDownmixRequirement::None,
                expected_reasons: vec![PlaybackAudioCompatibilityReason::NormalizationRequested],
            },
            AudioOutputRequirementMatrixCase {
                name: "downmix before normalization request",
                source_channels: Some(8),
                max_supported_channels: Some(2),
                normalization: PlaybackAudioNormalizationRequirement::Requested,
                expected_target_channels: Some(2),
                expected_downmix: PlaybackAudioDownmixRequirement::Required,
                expected_reasons: vec![
                    PlaybackAudioCompatibilityReason::ChannelLimitExceeded,
                    PlaybackAudioCompatibilityReason::DownmixRequired,
                    PlaybackAudioCompatibilityReason::NormalizationRequested,
                ],
            },
        ]
    }

    struct RemuxVideoOutputConstraintCase {
        name: &'static str,
        video_bitrate: Option<u64>,
        video_width: Option<u32>,
        video_height: Option<u32>,
        client: ClientPlaybackCapabilities,
        context: PlaybackSelectionContext,
        expected_remux_condition: PlaybackCompatibilityCondition,
    }

    fn remux_video_output_constraint_cases() -> Vec<RemuxVideoOutputConstraintCase> {
        vec![
            RemuxVideoOutputConstraintCase {
                name: "client bitrate cap requires transcode instead of remux",
                video_bitrate: Some(12_000_000),
                video_width: Some(1920),
                video_height: Some(1080),
                client: ClientPlaybackCapabilities {
                    max_video_bitrate: Some(8_000_000),
                    ..ClientPlaybackCapabilities::default()
                },
                context: PlaybackSelectionContext::default(),
                expected_remux_condition: PlaybackCompatibilityCondition::VideoBitrateUnsupported,
            },
            RemuxVideoOutputConstraintCase {
                name: "client resolution cap requires transcode instead of remux",
                video_bitrate: Some(8_000_000),
                video_width: Some(3840),
                video_height: Some(2160),
                client: ClientPlaybackCapabilities {
                    max_width: Some(1920),
                    max_height: Some(1080),
                    ..ClientPlaybackCapabilities::default()
                },
                context: PlaybackSelectionContext::default(),
                expected_remux_condition:
                    PlaybackCompatibilityCondition::VideoResolutionUnsupported,
            },
            RemuxVideoOutputConstraintCase {
                name: "user bitrate preference requires transcode instead of remux",
                video_bitrate: Some(7_000_000),
                video_width: Some(1920),
                video_height: Some(1080),
                client: ClientPlaybackCapabilities::default(),
                context: PlaybackSelectionContext {
                    storage: PlaybackStorageContext::default(),
                    preferences: PlaybackPreferenceContext {
                        max_video_bitrate: Some(5_000_000),
                        ..PlaybackPreferenceContext::default()
                    },
                },
                expected_remux_condition: PlaybackCompatibilityCondition::VideoBitrateUnsupported,
            },
        ]
    }

    fn matrix_probe(
        container: &str,
        video_codec: &str,
        hdr: MatrixHdrInput,
        audio_codec: &str,
        audio_channels: Option<u32>,
    ) -> MediaProbeResult {
        let mut video = stream(MediaStreamKind::Video, Some(video_codec));
        video.index = 0;
        apply_matrix_hdr(&mut video, hdr);

        let mut audio = stream(MediaStreamKind::Audio, Some(audio_codec));
        audio.index = 1;
        audio.channels = audio_channels;

        MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some(container.to_owned()),
            bit_rate: None,
            streams: vec![video, audio],
        }
    }

    fn apply_matrix_hdr(stream: &mut MediaStreamInfo, hdr: MatrixHdrInput) {
        match hdr {
            MatrixHdrInput::Sdr => {}
            MatrixHdrInput::Hdr10 => {
                stream.technical = MediaStreamTechnicalFacts {
                    color: MediaColorInfo {
                        space: Some("bt2020nc".to_owned()),
                        transfer: Some("smpte2084".to_owned()),
                        primaries: Some("bt2020".to_owned()),
                        ..MediaColorInfo::default()
                    },
                    hdr: MediaHdrMetadata {
                        dynamic_range: Some("hdr10".to_owned()),
                        mastering_display: true,
                        content_light_level: true,
                        ..MediaHdrMetadata::default()
                    },
                    ..MediaStreamTechnicalFacts::default()
                };
            }
        }
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
            decision.rendition,
            PlaybackRenditionPlan::Denied(PlaybackDenial {
                permission: actual_permission,
                reason: actual_reason,
            }) if actual_permission == permission && actual_reason == reason
        ));
        assert_eq!(decision.denial, Some(PlaybackDenial { permission, reason }));
        assert_eq!(
            decision.report.selection_reasons,
            vec![PlaybackCompatibilityCondition::PolicyDenied]
        );
        assert!(decision.direct_play_plan().is_none());
        assert!(decision.transcode_plan().is_none());
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
            technical: Default::default(),
        }
    }
}
