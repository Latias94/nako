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
    pub prefer_hdr: Option<bool>,
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
}

impl TranscodeExecutionPolicy {
    #[must_use]
    pub const fn remux() -> Self {
        Self {
            acceleration: TranscodeAccelerationPlan::software(),
            output_constraints: TranscodeOutputConstraints {
                max_video_bitrate: None,
                prefer_hdr: None,
            },
            subtitle_strategy: TranscodeSubtitleStrategy::PreserveInContainer,
        }
    }

    #[must_use]
    pub fn hls_single_variant(
        acceleration: TranscodeAccelerationPlan,
        track_selection: TranscodeTrackSelection,
        output_constraints: TranscodeOutputConstraints,
    ) -> Self {
        Self {
            acceleration,
            output_constraints,
            subtitle_strategy: if track_selection.subtitle_stream.is_some() {
                TranscodeSubtitleStrategy::OmitSelected
            } else {
                TranscodeSubtitleStrategy::None
            },
        }
    }
}
