use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackRemuxContainer {
    Mp4,
    Mkv,
}

impl PlaybackRemuxContainer {
    #[must_use]
    pub const fn file_extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackTranscodeContainer {
    Hls,
    Mp4,
    Mkv,
}

impl PlaybackTranscodeContainer {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hls => "hls",
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackTranscodePlan {
    pub input_locator: String,
    pub output_container: PlaybackTranscodeContainer,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PlaybackTrackSelection {
    pub audio_stream: Option<u32>,
    pub subtitle_stream: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PlaybackOutputConstraints {
    pub max_video_bitrate: Option<u64>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub prefer_hdr: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackColorPipelineSource {
    pub dynamic_range: Option<String>,
    pub color_space: Option<String>,
    pub color_transfer: Option<String>,
    pub color_primaries: Option<String>,
    pub mastering_display: bool,
    pub content_light_level: bool,
    pub dolby_vision: bool,
    pub hdr10_plus: bool,
}

impl PlaybackColorPipelineSource {
    fn has_hdr(&self) -> bool {
        self.dynamic_range.is_some()
            || self.mastering_display
            || self.content_light_level
            || self.dolby_vision
            || self.hdr10_plus
            || self.color_transfer.as_deref().is_some_and(is_hdr_transfer)
    }

    fn has_deferred_unsupported_hdr(&self) -> bool {
        self.dolby_vision
            || self.hdr10_plus
            || self.dynamic_range.as_deref().is_some_and(|dynamic_range| {
                dynamic_range.eq_ignore_ascii_case("dolby_vision")
                    || dynamic_range.eq_ignore_ascii_case("dovi")
                    || dynamic_range.eq_ignore_ascii_case("hdr10_plus")
            })
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackColorPipelineRequirement {
    pub source: Option<PlaybackColorPipelineSource>,
    pub target: PlaybackColorPipelineTarget,
    pub tone_mapping: PlaybackHdrToneMappingRequirement,
    pub reasons: Vec<PlaybackColorCompatibilityReason>,
}

impl PlaybackColorPipelineRequirement {
    #[must_use]
    pub fn from_source(
        source: Option<PlaybackColorPipelineSource>,
        client_supports_hdr: bool,
    ) -> Self {
        let Some(source) = source else {
            return Self::default();
        };

        let mut requirement = Self {
            source: Some(source),
            ..Self::default()
        };

        let source = requirement
            .source
            .as_ref()
            .expect("source is set before HDR classification");
        if !source.has_hdr() {
            return requirement;
        }

        requirement.push_reason(PlaybackColorCompatibilityReason::SourceHdrDetected);
        if client_supports_hdr {
            requirement.push_reason(PlaybackColorCompatibilityReason::HdrPassthroughSupported);
            return requirement;
        }

        requirement.target = PlaybackColorPipelineTarget::Sdr;
        requirement.push_reason(PlaybackColorCompatibilityReason::ClientHdrUnsupported);
        if requirement
            .source
            .as_ref()
            .is_some_and(PlaybackColorPipelineSource::has_deferred_unsupported_hdr)
        {
            requirement.tone_mapping = PlaybackHdrToneMappingRequirement::DeferredUnsupported;
            requirement.push_reason(PlaybackColorCompatibilityReason::UnsupportedHdrFormatDeferred);
        } else {
            requirement.tone_mapping = PlaybackHdrToneMappingRequirement::Required;
            requirement.push_reason(PlaybackColorCompatibilityReason::ToneMappingRequired);
        }

        requirement
    }

    fn push_reason(&mut self, reason: PlaybackColorCompatibilityReason) {
        if !self.reasons.contains(&reason) {
            self.reasons.push(reason);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackColorPipelineTarget {
    #[default]
    PreserveSource,
    Sdr,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackHdrToneMappingRequirement {
    #[default]
    None,
    Required,
    DeferredUnsupported,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackColorCompatibilityReason {
    SourceHdrDetected,
    ClientHdrUnsupported,
    HdrPassthroughSupported,
    ToneMappingRequired,
    UnsupportedHdrFormatDeferred,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackAudioOutputRequirement {
    pub source_channels: Option<u32>,
    pub max_supported_channels: Option<u32>,
    pub target_channels: Option<u32>,
    pub downmix: PlaybackAudioDownmixRequirement,
    pub normalization: PlaybackAudioNormalizationRequirement,
    pub reasons: Vec<PlaybackAudioCompatibilityReason>,
}

impl PlaybackAudioOutputRequirement {
    #[must_use]
    pub fn from_channel_support(
        source_channels: Option<u32>,
        max_supported_channels: Option<u32>,
    ) -> Self {
        let mut requirement = Self {
            source_channels,
            max_supported_channels,
            ..Self::default()
        };

        if let (Some(source), Some(max_supported)) = (source_channels, max_supported_channels)
            && max_supported > 0
            && source > max_supported
        {
            requirement.target_channels = Some(max_supported);
            requirement.downmix = PlaybackAudioDownmixRequirement::Required;
            requirement.push_reason(PlaybackAudioCompatibilityReason::ChannelLimitExceeded);
            requirement.push_reason(PlaybackAudioCompatibilityReason::DownmixRequired);
        }

        requirement
    }

    #[must_use]
    pub fn with_normalization(
        mut self,
        normalization: PlaybackAudioNormalizationRequirement,
    ) -> Self {
        self.normalization = normalization;
        self.reasons
            .retain(|reason| *reason != PlaybackAudioCompatibilityReason::NormalizationRequested);
        if normalization.is_requested() {
            self.push_reason(PlaybackAudioCompatibilityReason::NormalizationRequested);
        }
        self
    }

    fn push_reason(&mut self, reason: PlaybackAudioCompatibilityReason) {
        if !self.reasons.contains(&reason) {
            self.reasons.push(reason);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackAudioDownmixRequirement {
    #[default]
    None,
    Required,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackAudioNormalizationRequirement {
    #[default]
    None,
    Requested,
}

impl PlaybackAudioNormalizationRequirement {
    #[must_use]
    pub const fn is_requested(self) -> bool {
        matches!(self, Self::Requested)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackAudioCompatibilityReason {
    ChannelLimitExceeded,
    DownmixRequired,
    NormalizationRequested,
}

fn is_hdr_transfer(transfer: &str) -> bool {
    transfer.eq_ignore_ascii_case("smpte2084")
        || transfer.eq_ignore_ascii_case("arib-std-b67")
        || transfer.eq_ignore_ascii_case("hlg")
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackHlsVariantPolicy {
    #[default]
    SingleVariant,
    Adaptive,
}

impl PlaybackHlsVariantPolicy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SingleVariant => "single_variant",
            Self::Adaptive => "adaptive",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackHlsSegmentContainer {
    #[default]
    MpegTs,
    Fmp4,
}

impl PlaybackHlsSegmentContainer {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MpegTs => "mpeg_ts",
            Self::Fmp4 => "fmp4",
        }
    }

    #[must_use]
    pub const fn segment_extension(self) -> &'static str {
        match self {
            Self::MpegTs => "ts",
            Self::Fmp4 => "m4s",
        }
    }

    #[must_use]
    pub const fn segment_content_type(self) -> &'static str {
        match self {
            Self::MpegTs => "video/mp2t",
            Self::Fmp4 => "video/mp4",
        }
    }

    #[must_use]
    pub const fn init_segment_file_name(self) -> Option<&'static str> {
        match self {
            Self::MpegTs => None,
            Self::Fmp4 => Some("init.mp4"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PlaybackHlsOutputRequirement {
    pub variant_policy: PlaybackHlsVariantPolicy,
    pub segment_container: PlaybackHlsSegmentContainer,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackSubtitleStrategy {
    #[default]
    None,
    PreserveInContainer,
    OmitSelected,
    BurnInSelected,
    SidecarSelected,
}

impl PlaybackSubtitleStrategy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::PreserveInContainer => "preserve_in_container",
            Self::OmitSelected => "omit_selected",
            Self::BurnInSelected => "burn_in_selected",
            Self::SidecarSelected => "sidecar_selected",
        }
    }
}
