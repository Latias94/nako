pub use nako_core::{
    EffectivePlaybackPolicy, EffectivePlaybackPolicyReason, PlaybackPermission,
    PlaybackPermissionDecision, PlaybackPermissionDecisionReason, PlaybackPermissionPolicy,
    PlaybackTargetId, PlaybackTargetKind, PlaybackTargetNetworkScope, PlaybackTargetTransportAuth,
    RendererControlCapabilities, RendererControlCommand,
};
use nako_core::{
    LibraryId, MediaProbeResult, MediaSource, MediaSourceId, MediaStreamInfo, MediaStreamKind,
};
use nako_transcode::{
    HlsOutputRequirement, HlsSegmentContainer, HlsVariantPolicy, OutputContainer, RemuxContainer,
    TranscodeOutputConstraints, TranscodePlan, TranscodeSubtitleStrategy, TranscodeTrackSelection,
};
use serde::{Deserialize, Serialize};

mod capability;

pub use capability::{
    DirectPlayCapabilityProfile, PlaybackCapabilityEvaluation, PlaybackCompatibilityCondition,
    PlaybackDecisionReport, PlaybackTargetProfile, RemuxCapabilityProfile,
    TranscodeCapabilityProfile,
};
use capability::{evaluate_direct_play, evaluate_remux, evaluate_transcode};

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
    pub fn transcode_plan(&self) -> Option<&TranscodePlan> {
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
    pub requested_subtitle_stream: Option<u32>,
    pub max_video_bitrate: Option<u64>,
    pub prefer_hdr: Option<bool>,
    pub remux_output_container: Option<RemuxContainer>,
    pub transcode_output_container: Option<OutputContainer>,
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
    pub plan: TranscodePlan,
    pub requirement: TranscodeRequirement,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemuxPlaybackPlan {
    pub source_id: MediaSourceId,
    pub input_locator: String,
    pub output_container: RemuxContainer,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeRequirement {
    pub source_id: MediaSourceId,
    pub input_locator: String,
    pub output_container: OutputContainer,
    pub output_video_codec: Option<String>,
    pub output_audio_codec: Option<String>,
    pub track_selection: TranscodeTrackSelection,
    pub output_constraints: TranscodeOutputConstraints,
    pub hls_output: Option<HlsOutputRequirement>,
    pub subtitle_strategy: TranscodeSubtitleStrategy,
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
    pub hls_variant_policy: HlsVariantPolicy,
    #[serde(default)]
    pub hls_segment_container: HlsSegmentContainer,
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
            hls_variant_policy: HlsVariantPolicy::SingleVariant,
            hls_segment_container: HlsSegmentContainer::MpegTs,
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
            OutputContainer::Hls,
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
            OutputContainer::Hls,
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
                    .unwrap_or(RemuxContainer::Mp4),
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
                OutputContainer::Hls,
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
            OutputContainer::Hls,
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
    PlaybackDecision {
        mode: PlaybackMode::DirectPlay,
        reason,
        selected_source,
        rendition: PlaybackRenditionPlan::DirectPlay(direct_play),
        report: report.with_selected_mode(PlaybackMode::DirectPlay),
        denial: None,
    }
}

fn remux_decision(
    selected_source: PlaybackSelectedSource,
    input_locator: String,
    output_container: RemuxContainer,
    reason: PlaybackDecisionReason,
    report: PlaybackDecisionReport,
) -> PlaybackDecision {
    PlaybackDecision {
        mode: PlaybackMode::Remux,
        reason,
        rendition: PlaybackRenditionPlan::Remux(RemuxPlaybackPlan {
            source_id: selected_source.source_id,
            input_locator,
            output_container,
        }),
        report: report.with_selected_mode(PlaybackMode::Remux),
        selected_source,
        denial: None,
    }
}

fn transcode_decision(
    selected_source: PlaybackSelectedSource,
    input_locator: String,
    output_container: OutputContainer,
    reason: PlaybackDecisionReason,
    report: PlaybackDecisionReport,
    target_profile: &PlaybackTargetProfile,
    probe: Option<&MediaProbeResult>,
) -> PlaybackDecision {
    let output_profile = target_profile
        .transcode_profiles
        .iter()
        .find(|profile| profile.output_container == output_container);
    let video_codec = output_profile
        .and_then(|profile| profile.video_codec.clone())
        .or_else(|| (output_container == OutputContainer::Hls).then(|| "h264".to_owned()));
    let audio_codec = output_profile
        .and_then(|profile| profile.audio_codec.clone())
        .or_else(|| (output_container == OutputContainer::Hls).then(|| "aac".to_owned()));
    let transcode_plan = TranscodePlan {
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
        reason,
        &report,
    );

    PlaybackDecision {
        mode: PlaybackMode::Transcode,
        reason,
        rendition: PlaybackRenditionPlan::Transcode(TranscodeRenditionPlan {
            plan: transcode_plan,
            requirement: transcode_requirement,
        }),
        report: report.with_selected_mode(PlaybackMode::Transcode),
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
    output_container: OutputContainer,
    output_video_codec: Option<String>,
    output_audio_codec: Option<String>,
    target_profile: &PlaybackTargetProfile,
    probe: Option<&MediaProbeResult>,
    reason: PlaybackDecisionReason,
    report: &PlaybackDecisionReport,
) -> TranscodeRequirement {
    let track_selection = target_profile.track_selection();
    TranscodeRequirement {
        source_id,
        input_locator,
        output_container,
        output_video_codec,
        output_audio_codec,
        track_selection,
        output_constraints: target_profile.output_constraints(),
        hls_output: (output_container == OutputContainer::Hls)
            .then_some(target_profile.hls_output_requirement()),
        subtitle_strategy: if track_selection.subtitle_stream.is_some() {
            TranscodeSubtitleStrategy::OmitSelected
        } else {
            TranscodeSubtitleStrategy::None
        },
        selected_streams: selected_transcode_streams(probe, track_selection),
        reasons: transcode_requirement_reasons(reason, report),
    }
}

fn selected_transcode_streams(
    probe: Option<&MediaProbeResult>,
    track_selection: TranscodeTrackSelection,
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
            channel_layout: stream.technical.channel_layout.clone(),
            forced: stream.technical.disposition.forced,
            default: stream.technical.disposition.default,
        }
    }
}

fn transcode_requirement_reasons(
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
            decision.rendition,
            PlaybackRenditionPlan::Remux(RemuxPlaybackPlan {
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
        assert!(
            decision
                .report
                .transcode
                .has(PlaybackCompatibilityCondition::RequestedTranscodeOutput)
        );
        assert!(matches!(
            decision.rendition,
            PlaybackRenditionPlan::Transcode(TranscodeRenditionPlan {
                plan: nako_transcode::TranscodePlan {
                    output_container: nako_transcode::OutputContainer::Hls,
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
        assert_eq!(requirement.output_container, OutputContainer::Hls);
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
            TranscodeSubtitleStrategy::OmitSelected
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
            hls_variant_policy: nako_transcode::HlsVariantPolicy::Adaptive,
            hls_segment_container: nako_transcode::HlsSegmentContainer::Fmp4,
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
            Some(nako_transcode::HlsOutputRequirement {
                variant_policy: nako_transcode::HlsVariantPolicy::Adaptive,
                segment_container: nako_transcode::HlsSegmentContainer::Fmp4,
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
                    requested_subtitle_stream: None,
                    max_video_bitrate: Some(8_000_000),
                    prefer_hdr: Some(true),
                    remux_output_container: Some(nako_transcode::RemuxContainer::Mp4),
                    transcode_output_container: Some(nako_transcode::OutputContainer::Hls),
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
        assert!(left.identity_key().contains("hls_variant=single_variant"));
        assert!(left.identity_key().contains("hls_segment=mpeg_ts"));
        assert!(
            left.identity_key()
                .contains("transcode=container=hls,vcodec=h264,acodec=aac")
        );
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
        assert!(decision.report.transcode.supported);
    }

    #[test]
    fn playback_target_profile_builds_hls_execution_policy_from_runtime_acceleration() {
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
        let plan = nako_transcode::TranscodePlan {
            input_locator: "local:///demo.mkv".to_owned(),
            output_container: nako_transcode::OutputContainer::Hls,
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
        };

        let hls_profile = profile
            .try_hls_transcode_profile(
                &plan,
                nako_transcode::TranscodeExecutionPolicy::hls_single_variant(
                    nako_transcode::TranscodeAccelerationPlan::for_selected_hardware(
                        nako_transcode::HardwareAcceleration::Nvenc,
                    ),
                    profile.track_selection(),
                    nako_transcode::TranscodeOutputConstraints {
                        max_video_bitrate: profile.preferences.max_video_bitrate,
                        max_width: None,
                        max_height: None,
                        prefer_hdr: profile.preferences.prefer_hdr,
                    },
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
            decision.rendition,
            PlaybackRenditionPlan::Denied(PlaybackDenial {
                permission: actual_permission,
                reason: actual_reason,
            }) if actual_permission == permission && actual_reason == reason
        ));
        assert_eq!(decision.denial, Some(PlaybackDenial { permission, reason }));
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
