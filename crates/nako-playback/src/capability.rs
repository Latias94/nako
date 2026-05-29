use nako_core::{MediaProbeResult, MediaSourceId, MediaStreamKind};
use serde::{Deserialize, Serialize};

use crate::{
    ClientPlaybackCapabilities, PlaybackDenial, PlaybackHlsOutputRequirement, PlaybackMode,
    PlaybackOutputConstraints, PlaybackPreferenceContext, PlaybackRemuxContainer,
    PlaybackSelectionContext, PlaybackStorageContext, PlaybackTarget, PlaybackTrackSelection,
    PlaybackTranscodeContainer,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackCompatibilityCondition {
    Compatible,
    DirectPlayDisabled,
    MediaTechnicalFactsMissing,
    ContainerUnknown,
    ContainerUnsupported,
    RemuxContainerUnsupported,
    VideoCodecUnsupported,
    AudioCodecUnsupported,
    VideoBitrateUnsupported,
    VideoResolutionUnsupported,
    VideoHdrUnsupported,
    AudioChannelsUnsupported,
    SubtitleDeliveryUnsupported,
    RequestedTranscodeOutput,
    TranscodeProfileUnsupported,
    PolicyDenied,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackCapabilityEvaluation {
    pub supported: bool,
    pub reasons: Vec<PlaybackCompatibilityCondition>,
}

impl PlaybackCapabilityEvaluation {
    #[must_use]
    pub fn supported() -> Self {
        Self {
            supported: true,
            reasons: vec![PlaybackCompatibilityCondition::Compatible],
        }
    }

    #[must_use]
    pub fn unsupported(reasons: Vec<PlaybackCompatibilityCondition>) -> Self {
        Self {
            supported: false,
            reasons,
        }
    }

    #[must_use]
    pub fn has(&self, reason: PlaybackCompatibilityCondition) -> bool {
        self.reasons.contains(&reason)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackDecisionReport {
    pub source_id: MediaSourceId,
    pub profile_key: String,
    pub selected_mode: PlaybackMode,
    pub direct_play: PlaybackCapabilityEvaluation,
    pub remux: PlaybackCapabilityEvaluation,
    pub transcode: PlaybackCapabilityEvaluation,
    pub denial: Option<PlaybackDenial>,
}

impl PlaybackDecisionReport {
    #[must_use]
    pub fn new(source_id: MediaSourceId, profile_key: String) -> Self {
        Self {
            source_id,
            profile_key,
            selected_mode: PlaybackMode::Denied,
            direct_play: PlaybackCapabilityEvaluation::default(),
            remux: PlaybackCapabilityEvaluation::default(),
            transcode: PlaybackCapabilityEvaluation::default(),
            denial: None,
        }
    }

    #[must_use]
    pub fn with_selected_mode(mut self, selected_mode: PlaybackMode) -> Self {
        self.selected_mode = selected_mode;
        self
    }

    #[must_use]
    pub fn with_denial(mut self, denial: PlaybackDenial) -> Self {
        self.denial = Some(denial);
        self.direct_play = PlaybackCapabilityEvaluation::unsupported(vec![
            PlaybackCompatibilityCondition::PolicyDenied,
        ]);
        self.remux = PlaybackCapabilityEvaluation::unsupported(vec![
            PlaybackCompatibilityCondition::PolicyDenied,
        ]);
        self.transcode = PlaybackCapabilityEvaluation::unsupported(vec![
            PlaybackCompatibilityCondition::PolicyDenied,
        ]);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackTargetProfile {
    pub direct_play: bool,
    pub direct_play_profiles: Vec<DirectPlayCapabilityProfile>,
    pub remux_profiles: Vec<RemuxCapabilityProfile>,
    pub transcode_profiles: Vec<TranscodeCapabilityProfile>,
    pub hls_output: PlaybackHlsOutputRequirement,
    pub storage: PlaybackStorageContext,
    pub preferences: PlaybackPreferenceContext,
}

impl PlaybackTargetProfile {
    #[must_use]
    pub fn from_target(target: &PlaybackTarget, context: PlaybackSelectionContext) -> Self {
        Self::from_capabilities(&target.media_capabilities, context)
    }

    #[must_use]
    pub fn from_capabilities(
        capabilities: &ClientPlaybackCapabilities,
        context: PlaybackSelectionContext,
    ) -> Self {
        let containers = normalized_values(&capabilities.containers);
        let video_codecs = normalized_values(&capabilities.video_codecs);
        let audio_codecs = normalized_values(&capabilities.audio_codecs);

        Self {
            direct_play: capabilities.direct_play,
            direct_play_profiles: vec![DirectPlayCapabilityProfile {
                containers: containers.clone(),
                video_codecs: video_codecs.clone(),
                audio_codecs: audio_codecs.clone(),
                max_video_bitrate: min_optional_u64(
                    capabilities.max_video_bitrate,
                    context.preferences.max_video_bitrate,
                ),
                max_width: capabilities.max_width,
                max_height: capabilities.max_height,
                max_audio_channels: capabilities.max_audio_channels,
                supports_hdr: capabilities.supports_hdr,
                supports_subtitles: capabilities.supports_subtitles,
            }],
            remux_profiles: vec![RemuxCapabilityProfile {
                output_containers: vec![PlaybackRemuxContainer::Mp4, PlaybackRemuxContainer::Mkv],
                video_codecs,
                audio_codecs,
            }],
            transcode_profiles: vec![TranscodeCapabilityProfile {
                output_container: PlaybackTranscodeContainer::Hls,
                video_codec: Some("h264".to_owned()),
                audio_codec: Some("aac".to_owned()),
            }],
            hls_output: PlaybackHlsOutputRequirement {
                variant_policy: capabilities.hls_variant_policy,
                segment_container: capabilities.hls_segment_container,
            },
            storage: context.storage,
            preferences: context.preferences,
        }
    }

    #[must_use]
    pub fn identity(&self) -> crate::PlaybackProfileIdentity {
        crate::PlaybackProfileIdentity {
            request_key: format!(
                "playback-target-profile:v1;direct={};direct={};remux={};transcode={};hls_variant={};hls_segment={};remote={};range={};audio={};subtitle={};max_video_bitrate={};prefer_hdr={};remux_pref={};transcode_pref={}",
                self.direct_play,
                direct_play_profiles_key(&self.direct_play_profiles),
                remux_profiles_key(&self.remux_profiles),
                transcode_profiles_key(&self.transcode_profiles),
                self.hls_output.variant_policy.as_str(),
                self.hls_output.segment_container.as_str(),
                self.storage.remote,
                optional_bool(self.storage.range_readable),
                optional_u32(self.preferences.requested_audio_stream),
                optional_u32(self.preferences.requested_subtitle_stream),
                optional_u64(self.preferences.max_video_bitrate),
                optional_bool(self.preferences.prefer_hdr),
                self.preferences
                    .remux_output_container
                    .map_or("auto", PlaybackRemuxContainer::file_extension),
                self.preferences
                    .transcode_output_container
                    .map_or("auto", PlaybackTranscodeContainer::as_str),
            ),
        }
    }

    #[must_use]
    pub fn identity_key(&self) -> String {
        self.identity().persisted_request_key().to_owned()
    }

    #[must_use]
    pub fn track_selection(&self) -> PlaybackTrackSelection {
        PlaybackTrackSelection {
            audio_stream: self.preferences.requested_audio_stream,
            subtitle_stream: self.preferences.requested_subtitle_stream,
        }
    }

    #[must_use]
    pub fn output_constraints(&self) -> PlaybackOutputConstraints {
        PlaybackOutputConstraints {
            max_video_bitrate: min_optional_u64(
                self.preferences.max_video_bitrate,
                self.direct_play_profiles
                    .iter()
                    .filter_map(|profile| profile.max_video_bitrate)
                    .min(),
            ),
            max_width: self
                .direct_play_profiles
                .iter()
                .filter_map(|profile| profile.max_width)
                .min(),
            max_height: self
                .direct_play_profiles
                .iter()
                .filter_map(|profile| profile.max_height)
                .min(),
            prefer_hdr: if self
                .direct_play_profiles
                .iter()
                .any(|profile| !profile.supports_hdr)
            {
                Some(false)
            } else {
                self.preferences.prefer_hdr
            },
        }
    }

    #[must_use]
    pub const fn hls_output_requirement(&self) -> PlaybackHlsOutputRequirement {
        self.hls_output
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DirectPlayCapabilityProfile {
    pub containers: Vec<String>,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
    pub max_video_bitrate: Option<u64>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub max_audio_channels: Option<u32>,
    pub supports_hdr: bool,
    pub supports_subtitles: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemuxCapabilityProfile {
    pub output_containers: Vec<PlaybackRemuxContainer>,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeCapabilityProfile {
    pub output_container: PlaybackTranscodeContainer,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
}

pub(crate) fn evaluate_direct_play(
    probe: Option<&MediaProbeResult>,
    profile: &PlaybackTargetProfile,
    container: Option<&str>,
) -> PlaybackCapabilityEvaluation {
    let mut reasons = Vec::new();

    if !profile.direct_play {
        reasons.push(PlaybackCompatibilityCondition::DirectPlayDisabled);
    }

    let Some(container) = container else {
        reasons.push(PlaybackCompatibilityCondition::ContainerUnknown);
        return PlaybackCapabilityEvaluation::unsupported(reasons);
    };

    let direct_play_profile = profile
        .direct_play_profiles
        .iter()
        .find(|candidate| value_allowed(container, &candidate.containers));

    let Some(direct_play_profile) = direct_play_profile else {
        reasons.push(PlaybackCompatibilityCondition::ContainerUnsupported);
        return PlaybackCapabilityEvaluation::unsupported(reasons);
    };

    if let Some(probe) = probe {
        append_stream_compatibility_reasons(
            probe,
            &direct_play_profile.video_codecs,
            &direct_play_profile.audio_codecs,
            direct_play_profile.max_video_bitrate,
            direct_play_profile.max_width,
            direct_play_profile.max_height,
            direct_play_profile.max_audio_channels,
            direct_play_profile.supports_hdr,
            &mut reasons,
        );
    }

    if profile.preferences.requested_subtitle_stream.is_some()
        && !direct_play_profile.supports_subtitles
    {
        push_unique(
            &mut reasons,
            PlaybackCompatibilityCondition::SubtitleDeliveryUnsupported,
        );
    }

    if reasons.is_empty() {
        PlaybackCapabilityEvaluation::supported()
    } else {
        PlaybackCapabilityEvaluation::unsupported(reasons)
    }
}

pub(crate) fn evaluate_remux(
    probe: Option<&MediaProbeResult>,
    profile: &PlaybackTargetProfile,
) -> PlaybackCapabilityEvaluation {
    let mut reasons = Vec::new();
    let requested_container = profile
        .preferences
        .remux_output_container
        .unwrap_or(PlaybackRemuxContainer::Mp4);

    let remux_profile = profile.remux_profiles.iter().find(|candidate| {
        candidate
            .output_containers
            .iter()
            .any(|container| *container == requested_container)
    });

    let Some(remux_profile) = remux_profile else {
        return PlaybackCapabilityEvaluation::unsupported(vec![
            PlaybackCompatibilityCondition::RemuxContainerUnsupported,
        ]);
    };

    let Some(probe) = probe else {
        return PlaybackCapabilityEvaluation::unsupported(vec![
            PlaybackCompatibilityCondition::MediaTechnicalFactsMissing,
        ]);
    };

    append_stream_compatibility_reasons(
        probe,
        &remux_profile.video_codecs,
        &remux_profile.audio_codecs,
        None,
        None,
        None,
        None,
        true,
        &mut reasons,
    );

    if reasons.is_empty() {
        PlaybackCapabilityEvaluation::supported()
    } else {
        PlaybackCapabilityEvaluation::unsupported(reasons)
    }
}

pub(crate) fn evaluate_transcode(profile: &PlaybackTargetProfile) -> PlaybackCapabilityEvaluation {
    let requested_container = profile
        .preferences
        .transcode_output_container
        .unwrap_or(PlaybackTranscodeContainer::Hls);

    if profile
        .transcode_profiles
        .iter()
        .any(|candidate| candidate.output_container == requested_container)
    {
        PlaybackCapabilityEvaluation::supported()
    } else {
        PlaybackCapabilityEvaluation::unsupported(vec![
            PlaybackCompatibilityCondition::TranscodeProfileUnsupported,
        ])
    }
}

fn append_stream_compatibility_reasons(
    probe: &MediaProbeResult,
    video_codecs: &[String],
    audio_codecs: &[String],
    max_video_bitrate: Option<u64>,
    max_width: Option<u32>,
    max_height: Option<u32>,
    max_audio_channels: Option<u32>,
    supports_hdr: bool,
    reasons: &mut Vec<PlaybackCompatibilityCondition>,
) {
    for stream in &probe.streams {
        match stream.kind {
            MediaStreamKind::Video => {
                if !codec_allowed(stream.codec.as_deref(), video_codecs) {
                    push_unique(
                        reasons,
                        PlaybackCompatibilityCondition::VideoCodecUnsupported,
                    );
                }
                if max_video_bitrate
                    .zip(stream.bit_rate)
                    .is_some_and(|(max, actual)| actual > max)
                {
                    push_unique(
                        reasons,
                        PlaybackCompatibilityCondition::VideoBitrateUnsupported,
                    );
                }
                if max_width
                    .zip(stream.width)
                    .is_some_and(|(max, actual)| actual > max)
                    || max_height
                        .zip(stream.height)
                        .is_some_and(|(max, actual)| actual > max)
                {
                    push_unique(
                        reasons,
                        PlaybackCompatibilityCondition::VideoResolutionUnsupported,
                    );
                }
                if !supports_hdr && stream_has_hdr(stream) {
                    push_unique(reasons, PlaybackCompatibilityCondition::VideoHdrUnsupported);
                }
            }
            MediaStreamKind::Audio => {
                if !codec_allowed(stream.codec.as_deref(), audio_codecs) {
                    push_unique(
                        reasons,
                        PlaybackCompatibilityCondition::AudioCodecUnsupported,
                    );
                }
                if max_audio_channels
                    .zip(stream.channels)
                    .is_some_and(|(max, actual)| actual > max)
                {
                    push_unique(
                        reasons,
                        PlaybackCompatibilityCondition::AudioChannelsUnsupported,
                    );
                }
            }
            MediaStreamKind::Subtitle | MediaStreamKind::Data | MediaStreamKind::Attachment => {}
            MediaStreamKind::Other(_) => {}
        }
    }
}

fn codec_allowed(codec: Option<&str>, allowed: &[String]) -> bool {
    allowed.is_empty()
        || codec.is_none_or(|codec| {
            allowed
                .iter()
                .any(|value| value.eq_ignore_ascii_case(codec))
        })
}

fn value_allowed(value: &str, allowed: &[String]) -> bool {
    allowed.is_empty()
        || allowed
            .iter()
            .any(|allowed_value| allowed_value.eq_ignore_ascii_case(value))
}

fn push_unique(
    reasons: &mut Vec<PlaybackCompatibilityCondition>,
    reason: PlaybackCompatibilityCondition,
) {
    if !reasons.contains(&reason) {
        reasons.push(reason);
    }
}

fn direct_play_profiles_key(profiles: &[DirectPlayCapabilityProfile]) -> String {
    if profiles.is_empty() {
        return "none".to_owned();
    }

    profiles
        .iter()
        .map(|profile| {
            format!(
                "containers={},vcodecs={},acodecs={},maxv={},maxw={},maxh={},maxac={},hdr={},subtitles={}",
                list_key(&profile.containers),
                list_key(&profile.video_codecs),
                list_key(&profile.audio_codecs),
                optional_u64(profile.max_video_bitrate),
                optional_u32(profile.max_width),
                optional_u32(profile.max_height),
                optional_u32(profile.max_audio_channels),
                profile.supports_hdr,
                profile.supports_subtitles,
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn remux_profiles_key(profiles: &[RemuxCapabilityProfile]) -> String {
    if profiles.is_empty() {
        return "none".to_owned();
    }

    profiles
        .iter()
        .map(|profile| {
            let mut output_containers = profile
                .output_containers
                .iter()
                .map(|container| container.file_extension().to_owned())
                .collect::<Vec<_>>();
            output_containers.sort();
            output_containers.dedup();
            format!(
                "containers={},vcodecs={},acodecs={}",
                list_key(&output_containers),
                list_key(&profile.video_codecs),
                list_key(&profile.audio_codecs),
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn transcode_profiles_key(profiles: &[TranscodeCapabilityProfile]) -> String {
    if profiles.is_empty() {
        return "none".to_owned();
    }

    profiles
        .iter()
        .map(|profile| {
            format!(
                "container={},vcodec={},acodec={}",
                profile.output_container.as_str(),
                optional_codec(profile.video_codec.as_deref()),
                optional_codec(profile.audio_codec.as_deref()),
            )
        })
        .collect::<Vec<_>>()
        .join(";")
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

fn min_optional_u64(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn stream_has_hdr(stream: &nako_core::MediaStreamInfo) -> bool {
    stream.technical.hdr.dynamic_range.is_some()
        || stream.technical.hdr.mastering_display
        || stream.technical.hdr.content_light_level
        || stream
            .technical
            .color
            .transfer
            .as_deref()
            .is_some_and(|transfer| {
                transfer.eq_ignore_ascii_case("smpte2084")
                    || transfer.eq_ignore_ascii_case("arib-std-b67")
                    || transfer.eq_ignore_ascii_case("hlg")
            })
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

fn optional_codec(value: Option<&str>) -> String {
    value.map_or_else(|| "copy".to_owned(), ToOwned::to_owned)
}
