use serde::{Deserialize, Serialize};

use super::{HardwareAcceleration, HardwareAccelerationFallback, TranscodeTrackSelection};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeAccelerationStage {
    Decode,
    Filter,
    Encode,
}

impl TranscodeAccelerationStage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Decode => "decode",
            Self::Filter => "filter",
            Self::Encode => "encode",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct AccelerationStageSelection {
    pub stage: TranscodeAccelerationStage,
    pub accelerator: HardwareAcceleration,
}

impl AccelerationStageSelection {
    #[must_use]
    pub const fn new(stage: TranscodeAccelerationStage, accelerator: HardwareAcceleration) -> Self {
        Self { stage, accelerator }
    }

    #[must_use]
    pub const fn software(stage: TranscodeAccelerationStage) -> Self {
        Self::new(stage, HardwareAcceleration::None)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeAccelerationFallbackPlan {
    pub requested: HardwareAcceleration,
    pub selected: HardwareAcceleration,
    pub fallback: HardwareAccelerationFallback,
    pub fallback_used: bool,
}

impl TranscodeAccelerationFallbackPlan {
    #[must_use]
    pub const fn software() -> Self {
        Self {
            requested: HardwareAcceleration::None,
            selected: HardwareAcceleration::None,
            fallback: HardwareAccelerationFallback::Cpu,
            fallback_used: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeAccelerationPlan {
    pub decode: AccelerationStageSelection,
    pub filter: AccelerationStageSelection,
    pub encode: AccelerationStageSelection,
    pub fallback: TranscodeAccelerationFallbackPlan,
}

impl Default for TranscodeAccelerationPlan {
    fn default() -> Self {
        Self::software()
    }
}

impl TranscodeAccelerationPlan {
    #[must_use]
    pub const fn software() -> Self {
        Self {
            decode: AccelerationStageSelection::software(TranscodeAccelerationStage::Decode),
            filter: AccelerationStageSelection::software(TranscodeAccelerationStage::Filter),
            encode: AccelerationStageSelection::software(TranscodeAccelerationStage::Encode),
            fallback: TranscodeAccelerationFallbackPlan::software(),
        }
    }

    #[must_use]
    pub const fn for_selected_hardware(acceleration: HardwareAcceleration) -> Self {
        Self::from_selected_parts(
            acceleration,
            TranscodeAccelerationFallbackPlan {
                requested: acceleration,
                selected: acceleration,
                fallback: HardwareAccelerationFallback::Cpu,
                fallback_used: false,
            },
        )
    }

    #[must_use]
    pub const fn from_pipeline_selection(
        acceleration: HardwareAcceleration,
        fallback: TranscodeAccelerationFallbackPlan,
    ) -> Self {
        Self::from_selected_parts(acceleration, fallback)
    }

    const fn from_selected_parts(
        acceleration: HardwareAcceleration,
        fallback: TranscodeAccelerationFallbackPlan,
    ) -> Self {
        match acceleration {
            HardwareAcceleration::None => Self {
                fallback,
                ..Self::software()
            },
            HardwareAcceleration::Nvenc => Self {
                decode: AccelerationStageSelection::software(TranscodeAccelerationStage::Decode),
                filter: AccelerationStageSelection::software(TranscodeAccelerationStage::Filter),
                encode: AccelerationStageSelection::new(
                    TranscodeAccelerationStage::Encode,
                    HardwareAcceleration::Nvenc,
                ),
                fallback,
            },
            HardwareAcceleration::Vaapi => Self {
                decode: AccelerationStageSelection::new(
                    TranscodeAccelerationStage::Decode,
                    HardwareAcceleration::Vaapi,
                ),
                filter: AccelerationStageSelection::new(
                    TranscodeAccelerationStage::Filter,
                    HardwareAcceleration::Vaapi,
                ),
                encode: AccelerationStageSelection::new(
                    TranscodeAccelerationStage::Encode,
                    HardwareAcceleration::Vaapi,
                ),
                fallback,
            },
            HardwareAcceleration::QuickSync => Self {
                decode: AccelerationStageSelection::new(
                    TranscodeAccelerationStage::Decode,
                    HardwareAcceleration::QuickSync,
                ),
                filter: AccelerationStageSelection::software(TranscodeAccelerationStage::Filter),
                encode: AccelerationStageSelection::new(
                    TranscodeAccelerationStage::Encode,
                    HardwareAcceleration::QuickSync,
                ),
                fallback,
            },
            HardwareAcceleration::Amf => Self {
                decode: AccelerationStageSelection::software(TranscodeAccelerationStage::Decode),
                filter: AccelerationStageSelection::software(TranscodeAccelerationStage::Filter),
                encode: AccelerationStageSelection::new(
                    TranscodeAccelerationStage::Encode,
                    HardwareAcceleration::Amf,
                ),
                fallback,
            },
            HardwareAcceleration::VideoToolbox => Self {
                decode: AccelerationStageSelection::new(
                    TranscodeAccelerationStage::Decode,
                    HardwareAcceleration::VideoToolbox,
                ),
                filter: AccelerationStageSelection::software(TranscodeAccelerationStage::Filter),
                encode: AccelerationStageSelection::new(
                    TranscodeAccelerationStage::Encode,
                    HardwareAcceleration::VideoToolbox,
                ),
                fallback,
            },
        }
    }

    #[must_use]
    pub const fn resource_acceleration(self) -> HardwareAcceleration {
        if self.encode.accelerator.is_gpu() {
            self.encode.accelerator
        } else if self.decode.accelerator.is_gpu() {
            self.decode.accelerator
        } else if self.filter.accelerator.is_gpu() {
            self.filter.accelerator
        } else {
            HardwareAcceleration::None
        }
    }

    #[must_use]
    pub const fn is_software_only(self) -> bool {
        matches!(self.resource_acceleration(), HardwareAcceleration::None)
    }

    #[must_use]
    pub fn identity_key(self) -> String {
        format!(
            "accel:v1:decode={},filter={},encode={},requested={},fallback={},fallback_used={}",
            self.decode.accelerator.as_str(),
            self.filter.accelerator.as_str(),
            self.encode.accelerator.as_str(),
            self.fallback.requested.as_str(),
            self.fallback.fallback.as_str(),
            self.fallback.fallback_used,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeOutputConstraints {
    pub max_video_bitrate: Option<u64>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub prefer_hdr: Option<bool>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeColorPipelineRequirement {
    pub target: TranscodeColorPipelineTarget,
    pub tone_mapping: TranscodeHdrToneMappingRequirement,
    pub reasons: TranscodeColorCompatibilityReasons,
}

impl TranscodeColorPipelineRequirement {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            target: TranscodeColorPipelineTarget::PreserveSource,
            tone_mapping: TranscodeHdrToneMappingRequirement::None,
            reasons: TranscodeColorCompatibilityReasons::none(),
        }
    }

    #[must_use]
    pub const fn hdr_to_sdr_required() -> Self {
        Self {
            target: TranscodeColorPipelineTarget::Sdr,
            tone_mapping: TranscodeHdrToneMappingRequirement::Required,
            reasons: TranscodeColorCompatibilityReasons {
                source_hdr_detected: true,
                client_hdr_unsupported: true,
                hdr_passthrough_supported: false,
                tone_mapping_required: true,
                unsupported_hdr_format_deferred: false,
            },
        }
    }

    #[must_use]
    pub const fn hdr_to_sdr_deferred_unsupported() -> Self {
        Self {
            target: TranscodeColorPipelineTarget::Sdr,
            tone_mapping: TranscodeHdrToneMappingRequirement::DeferredUnsupported,
            reasons: TranscodeColorCompatibilityReasons {
                source_hdr_detected: true,
                client_hdr_unsupported: true,
                hdr_passthrough_supported: false,
                tone_mapping_required: false,
                unsupported_hdr_format_deferred: true,
            },
        }
    }

    #[must_use]
    pub const fn requires_hdr_to_sdr_tone_mapping(self) -> bool {
        matches!(
            (self.target, self.tone_mapping),
            (
                TranscodeColorPipelineTarget::Sdr,
                TranscodeHdrToneMappingRequirement::Required
            )
        )
    }

    #[must_use]
    pub const fn is_deferred_unsupported(self) -> bool {
        matches!(
            self.tone_mapping,
            TranscodeHdrToneMappingRequirement::DeferredUnsupported
        )
    }

    #[must_use]
    pub fn persisted_identity_key(self) -> String {
        format!(
            "target:{},tone_mapping:{},reasons:{}",
            self.target.as_str(),
            self.tone_mapping.as_str(),
            self.reasons.identity_key(),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeColorPipelineTarget {
    #[default]
    PreserveSource,
    Sdr,
}

impl TranscodeColorPipelineTarget {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreserveSource => "preserve_source",
            Self::Sdr => "sdr",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeHdrToneMappingRequirement {
    #[default]
    None,
    Required,
    DeferredUnsupported,
}

impl TranscodeHdrToneMappingRequirement {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Required => "required",
            Self::DeferredUnsupported => "deferred_unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeColorCompatibilityReasons {
    pub source_hdr_detected: bool,
    pub client_hdr_unsupported: bool,
    pub hdr_passthrough_supported: bool,
    pub tone_mapping_required: bool,
    pub unsupported_hdr_format_deferred: bool,
}

impl TranscodeColorCompatibilityReasons {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            source_hdr_detected: false,
            client_hdr_unsupported: false,
            hdr_passthrough_supported: false,
            tone_mapping_required: false,
            unsupported_hdr_format_deferred: false,
        }
    }

    #[must_use]
    pub const fn has(self, reason: TranscodeColorCompatibilityReason) -> bool {
        match reason {
            TranscodeColorCompatibilityReason::SourceHdrDetected => self.source_hdr_detected,
            TranscodeColorCompatibilityReason::ClientHdrUnsupported => self.client_hdr_unsupported,
            TranscodeColorCompatibilityReason::HdrPassthroughSupported => {
                self.hdr_passthrough_supported
            }
            TranscodeColorCompatibilityReason::ToneMappingRequired => self.tone_mapping_required,
            TranscodeColorCompatibilityReason::UnsupportedHdrFormatDeferred => {
                self.unsupported_hdr_format_deferred
            }
        }
    }

    #[must_use]
    pub fn identity_key(self) -> String {
        let mut reasons = Vec::new();
        if self.source_hdr_detected {
            reasons.push(TranscodeColorCompatibilityReason::SourceHdrDetected.as_str());
        }
        if self.client_hdr_unsupported {
            reasons.push(TranscodeColorCompatibilityReason::ClientHdrUnsupported.as_str());
        }
        if self.hdr_passthrough_supported {
            reasons.push(TranscodeColorCompatibilityReason::HdrPassthroughSupported.as_str());
        }
        if self.tone_mapping_required {
            reasons.push(TranscodeColorCompatibilityReason::ToneMappingRequired.as_str());
        }
        if self.unsupported_hdr_format_deferred {
            reasons.push(TranscodeColorCompatibilityReason::UnsupportedHdrFormatDeferred.as_str());
        }

        if reasons.is_empty() {
            "none".to_owned()
        } else {
            reasons.join("|")
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeColorCompatibilityReason {
    SourceHdrDetected,
    ClientHdrUnsupported,
    HdrPassthroughSupported,
    ToneMappingRequired,
    UnsupportedHdrFormatDeferred,
}

impl TranscodeColorCompatibilityReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceHdrDetected => "source_hdr_detected",
            Self::ClientHdrUnsupported => "client_hdr_unsupported",
            Self::HdrPassthroughSupported => "hdr_passthrough_supported",
            Self::ToneMappingRequired => "tone_mapping_required",
            Self::UnsupportedHdrFormatDeferred => "unsupported_hdr_format_deferred",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeAudioOutputRequirement {
    pub source_channels: Option<u32>,
    pub max_supported_channels: Option<u32>,
    pub target_channels: Option<u32>,
    pub downmix: TranscodeAudioDownmixRequirement,
    pub normalization: TranscodeAudioNormalizationRequirement,
    pub reasons: TranscodeAudioCompatibilityReasons,
}

impl TranscodeAudioOutputRequirement {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            source_channels: None,
            max_supported_channels: None,
            target_channels: None,
            downmix: TranscodeAudioDownmixRequirement::None,
            normalization: TranscodeAudioNormalizationRequirement::None,
            reasons: TranscodeAudioCompatibilityReasons::none(),
        }
    }

    #[must_use]
    pub fn persisted_identity_key(self) -> String {
        format!(
            "source:{},max:{},target:{},downmix:{},normalization:{},reasons:{}",
            optional_audio_channels(self.source_channels),
            optional_audio_channels(self.max_supported_channels),
            optional_audio_channels(self.target_channels),
            self.downmix.as_str(),
            self.normalization.as_str(),
            self.reasons.identity_key(),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeAudioDownmixRequirement {
    #[default]
    None,
    Required,
}

impl TranscodeAudioDownmixRequirement {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Required => "required",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeAudioNormalizationRequirement {
    #[default]
    None,
    Requested,
}

impl TranscodeAudioNormalizationRequirement {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Requested => "requested",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeAudioCompatibilityReasons {
    pub channel_limit_exceeded: bool,
    pub downmix_required: bool,
    pub normalization_requested: bool,
}

impl TranscodeAudioCompatibilityReasons {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            channel_limit_exceeded: false,
            downmix_required: false,
            normalization_requested: false,
        }
    }

    #[must_use]
    pub const fn has(self, reason: TranscodeAudioCompatibilityReason) -> bool {
        match reason {
            TranscodeAudioCompatibilityReason::ChannelLimitExceeded => self.channel_limit_exceeded,
            TranscodeAudioCompatibilityReason::DownmixRequired => self.downmix_required,
            TranscodeAudioCompatibilityReason::NormalizationRequested => {
                self.normalization_requested
            }
        }
    }

    #[must_use]
    pub fn identity_key(self) -> String {
        let mut reasons = Vec::new();
        if self.channel_limit_exceeded {
            reasons.push(TranscodeAudioCompatibilityReason::ChannelLimitExceeded.as_str());
        }
        if self.downmix_required {
            reasons.push(TranscodeAudioCompatibilityReason::DownmixRequired.as_str());
        }
        if self.normalization_requested {
            reasons.push(TranscodeAudioCompatibilityReason::NormalizationRequested.as_str());
        }

        if reasons.is_empty() {
            "none".to_owned()
        } else {
            reasons.join("|")
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeAudioCompatibilityReason {
    ChannelLimitExceeded,
    DownmixRequired,
    NormalizationRequested,
}

impl TranscodeAudioCompatibilityReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ChannelLimitExceeded => "channel_limit_exceeded",
            Self::DownmixRequired => "downmix_required",
            Self::NormalizationRequested => "normalization_requested",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HlsVariantPolicy {
    #[default]
    SingleVariant,
    Adaptive,
}

impl HlsVariantPolicy {
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
pub enum HlsSegmentContainer {
    #[default]
    MpegTs,
    Fmp4,
}

impl HlsSegmentContainer {
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
pub struct HlsOutputRequirement {
    pub variant_policy: HlsVariantPolicy,
    pub segment_container: HlsSegmentContainer,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeSubtitleStrategy {
    #[default]
    None,
    PreserveInContainer,
    OmitSelected,
    BurnInSelected,
    SidecarSelected,
}

impl TranscodeSubtitleStrategy {
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeExecutionPolicy {
    pub acceleration: TranscodeAccelerationPlan,
    pub output_constraints: TranscodeOutputConstraints,
    pub subtitle_strategy: TranscodeSubtitleStrategy,
    pub color_pipeline: TranscodeColorPipelineRequirement,
    pub audio_output: TranscodeAudioOutputRequirement,
}

impl TranscodeExecutionPolicy {
    #[must_use]
    pub const fn remux() -> Self {
        Self {
            acceleration: TranscodeAccelerationPlan::software(),
            output_constraints: TranscodeOutputConstraints {
                max_video_bitrate: None,
                max_width: None,
                max_height: None,
                prefer_hdr: None,
            },
            subtitle_strategy: TranscodeSubtitleStrategy::PreserveInContainer,
            color_pipeline: TranscodeColorPipelineRequirement::none(),
            audio_output: TranscodeAudioOutputRequirement::none(),
        }
    }

    #[must_use]
    pub fn hls_single_variant(
        acceleration: TranscodeAccelerationPlan,
        track_selection: TranscodeTrackSelection,
        output_constraints: TranscodeOutputConstraints,
    ) -> Self {
        Self::hls_single_variant_with_audio_output(
            acceleration,
            track_selection,
            output_constraints,
            TranscodeAudioOutputRequirement::none(),
        )
    }

    #[must_use]
    pub fn hls_single_variant_with_audio_output(
        acceleration: TranscodeAccelerationPlan,
        track_selection: TranscodeTrackSelection,
        output_constraints: TranscodeOutputConstraints,
        audio_output: TranscodeAudioOutputRequirement,
    ) -> Self {
        Self {
            acceleration,
            output_constraints,
            subtitle_strategy: if track_selection.subtitle_stream.is_some() {
                TranscodeSubtitleStrategy::OmitSelected
            } else {
                TranscodeSubtitleStrategy::None
            },
            color_pipeline: TranscodeColorPipelineRequirement::none(),
            audio_output,
        }
    }
}

fn optional_audio_channels(value: Option<u32>) -> String {
    value.map_or_else(|| "auto".to_owned(), |value| value.to_string())
}
