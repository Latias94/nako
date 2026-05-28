use nako_core::{MediaStreamInfo, NakoError, Result};
use serde::{Deserialize, Serialize};

use super::{
    HardwareAcceleration, HardwareAccelerationFallback, HardwareAccelerationPolicy,
    HardwareAccelerationReport, TranscodeAccelerationFallbackPlan, TranscodeAccelerationPlan,
    TranscodeExecutionPolicy, TranscodeOutputConstraints, TranscodeSubtitleStrategy,
    TranscodeTrackSelection,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodePipelineReadinessStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodePipelineReadinessReason {
    CpuRequested,
    RequestedPipelineReady,
    RequestedPipelineUnavailableFallbackToCpu,
    RequestedPipelineUnavailableFailPolicy,
    SoftwarePipelineUnavailable,
    CpuFallbackUnavailable,
    ProbeError,
    DeviceInitializationFailed,
    SmokeProbeFailed,
    SourceVideoCodecUnsupportedByRequestedPipeline,
    SourceVideoBitDepthUnsupportedByRequestedPipeline,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodePipelineReadiness {
    pub status: TranscodePipelineReadinessStatus,
    pub reason: TranscodePipelineReadinessReason,
    pub requested: HardwareAcceleration,
    pub selected: HardwareAcceleration,
    pub fallback_used: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodePipelineRequest {
    pub hardware_policy: HardwareAccelerationPolicy,
    pub track_selection: TranscodeTrackSelection,
    pub output_constraints: TranscodeOutputConstraints,
    pub subtitle_strategy: TranscodeSubtitleStrategy,
    pub source: Option<TranscodePipelineSourceFacts>,
}

impl TranscodePipelineRequest {
    #[must_use]
    pub fn hls_single_variant(
        hardware_policy: HardwareAccelerationPolicy,
        track_selection: TranscodeTrackSelection,
        output_constraints: TranscodeOutputConstraints,
    ) -> Self {
        Self {
            hardware_policy,
            track_selection,
            output_constraints,
            subtitle_strategy: if track_selection.subtitle_stream.is_some() {
                TranscodeSubtitleStrategy::OmitSelected
            } else {
                TranscodeSubtitleStrategy::None
            },
            source: None,
        }
    }

    #[must_use]
    pub fn with_source(mut self, source: TranscodePipelineSourceFacts) -> Self {
        self.source = Some(source);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodePipelineSourceFacts {
    pub video: Option<MediaStreamInfo>,
    pub audio: Option<MediaStreamInfo>,
    pub subtitle: Option<MediaStreamInfo>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodePipelinePlan {
    pub acceleration: TranscodeAccelerationPlan,
    pub output_constraints: TranscodeOutputConstraints,
    pub subtitle_strategy: TranscodeSubtitleStrategy,
    pub readiness: TranscodePipelineReadiness,
}

impl TranscodePipelinePlan {
    #[must_use]
    pub const fn execution_policy(self) -> TranscodeExecutionPolicy {
        TranscodeExecutionPolicy {
            acceleration: self.acceleration,
            output_constraints: self.output_constraints,
            subtitle_strategy: self.subtitle_strategy,
        }
    }

    #[must_use]
    pub const fn selected_acceleration(self) -> HardwareAcceleration {
        self.readiness.selected
    }

    #[must_use]
    pub const fn fallback_used(self) -> bool {
        self.readiness.fallback_used
    }
}

#[derive(Clone, Debug, Default)]
pub struct TranscodePipelinePlanner;

impl TranscodePipelinePlanner {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub fn plan_hls_single_variant(
        &self,
        request: TranscodePipelineRequest,
        report: &HardwareAccelerationReport,
    ) -> Result<TranscodePipelinePlan> {
        let selection =
            select_pipeline_acceleration(request.hardware_policy, report, request.source.as_ref())?;
        let fallback = TranscodeAccelerationFallbackPlan {
            requested: request.hardware_policy.requested,
            selected: selection.selected,
            fallback: request.hardware_policy.fallback,
            fallback_used: selection.fallback_used,
        };

        Ok(TranscodePipelinePlan {
            acceleration: TranscodeAccelerationPlan::from_pipeline_selection(
                selection.selected,
                fallback,
            ),
            output_constraints: request.output_constraints,
            subtitle_strategy: request.subtitle_strategy,
            readiness: selection,
        })
    }
}

fn select_pipeline_acceleration(
    policy: HardwareAccelerationPolicy,
    report: &HardwareAccelerationReport,
    source: Option<&TranscodePipelineSourceFacts>,
) -> Result<TranscodePipelineReadiness> {
    if policy.requested == HardwareAcceleration::None {
        if !report.is_available(HardwareAcceleration::None) {
            return Err(NakoError::Unsupported(
                "software transcode pipeline is unavailable",
            ));
        }

        return Ok(TranscodePipelineReadiness {
            status: TranscodePipelineReadinessStatus::Ready,
            reason: TranscodePipelineReadinessReason::CpuRequested,
            requested: HardwareAcceleration::None,
            selected: HardwareAcceleration::None,
            fallback_used: false,
        });
    }

    if report.is_available(policy.requested) {
        if let Some(reason) = source_incompatibility_reason(policy.requested, source) {
            return source_incompatible_readiness(policy, report, reason);
        }

        return Ok(TranscodePipelineReadiness {
            status: TranscodePipelineReadinessStatus::Ready,
            reason: TranscodePipelineReadinessReason::RequestedPipelineReady,
            requested: policy.requested,
            selected: policy.requested,
            fallback_used: false,
        });
    }

    let reason = unavailable_reason(policy, report);
    match policy.fallback {
        HardwareAccelerationFallback::Cpu => {
            if !report.is_available(HardwareAcceleration::None) {
                return Err(NakoError::Unsupported(
                    "requested hardware pipeline is unavailable and cpu fallback is unavailable",
                ));
            }

            Ok(TranscodePipelineReadiness {
                status: TranscodePipelineReadinessStatus::Degraded,
                reason,
                requested: policy.requested,
                selected: HardwareAcceleration::None,
                fallback_used: true,
            })
        }
        HardwareAccelerationFallback::Fail => Err(NakoError::Unsupported(
            "requested hardware pipeline is unavailable",
        )),
    }
}

#[must_use]
pub fn transcode_pipeline_readiness_without_selection(
    policy: HardwareAccelerationPolicy,
    report: &HardwareAccelerationReport,
) -> TranscodePipelineReadiness {
    select_pipeline_acceleration(policy, report, None)
        .unwrap_or_else(|_| unavailable_pipeline_readiness(policy, report))
}

fn source_incompatible_readiness(
    policy: HardwareAccelerationPolicy,
    report: &HardwareAccelerationReport,
    reason: TranscodePipelineReadinessReason,
) -> Result<TranscodePipelineReadiness> {
    match policy.fallback {
        HardwareAccelerationFallback::Cpu => {
            if !report.is_available(HardwareAcceleration::None) {
                return Err(NakoError::Unsupported(
                    "requested hardware pipeline is incompatible with source media and cpu fallback is unavailable",
                ));
            }

            Ok(TranscodePipelineReadiness {
                status: TranscodePipelineReadinessStatus::Degraded,
                reason,
                requested: policy.requested,
                selected: HardwareAcceleration::None,
                fallback_used: true,
            })
        }
        HardwareAccelerationFallback::Fail => Err(NakoError::Unsupported(
            "requested hardware pipeline is incompatible with source media",
        )),
    }
}

fn source_incompatibility_reason(
    accelerator: HardwareAcceleration,
    source: Option<&TranscodePipelineSourceFacts>,
) -> Option<TranscodePipelineReadinessReason> {
    if !uses_source_aware_hardware_decode(accelerator) {
        return None;
    }

    let video = source.and_then(|source| source.video.as_ref())?;
    if !video
        .codec
        .as_deref()
        .is_none_or(|codec| codec.eq_ignore_ascii_case("h264"))
    {
        return Some(
            TranscodePipelineReadinessReason::SourceVideoCodecUnsupportedByRequestedPipeline,
        );
    }

    if video
        .technical
        .bits_per_raw_sample
        .is_some_and(|bits| bits > 8)
        || video.technical.bits_per_sample.is_some_and(|bits| bits > 8)
    {
        return Some(
            TranscodePipelineReadinessReason::SourceVideoBitDepthUnsupportedByRequestedPipeline,
        );
    }

    None
}

fn uses_source_aware_hardware_decode(accelerator: HardwareAcceleration) -> bool {
    matches!(
        accelerator,
        HardwareAcceleration::Vaapi
            | HardwareAcceleration::QuickSync
            | HardwareAcceleration::VideoToolbox
    )
}

fn unavailable_pipeline_readiness(
    policy: HardwareAccelerationPolicy,
    report: &HardwareAccelerationReport,
) -> TranscodePipelineReadiness {
    let reason = if policy.requested == HardwareAcceleration::None {
        TranscodePipelineReadinessReason::SoftwarePipelineUnavailable
    } else if policy.fallback == HardwareAccelerationFallback::Cpu
        && !report.is_available(HardwareAcceleration::None)
    {
        TranscodePipelineReadinessReason::CpuFallbackUnavailable
    } else {
        TranscodePipelineReadinessReason::RequestedPipelineUnavailableFailPolicy
    };

    let selected = match reason {
        TranscodePipelineReadinessReason::SoftwarePipelineUnavailable
        | TranscodePipelineReadinessReason::CpuFallbackUnavailable => HardwareAcceleration::None,
        _ => policy.requested,
    };

    TranscodePipelineReadiness {
        status: TranscodePipelineReadinessStatus::Unavailable,
        reason,
        requested: policy.requested,
        selected,
        fallback_used: false,
    }
}

fn unavailable_reason(
    policy: HardwareAccelerationPolicy,
    report: &HardwareAccelerationReport,
) -> TranscodePipelineReadinessReason {
    if policy.fallback == HardwareAccelerationFallback::Fail {
        return TranscodePipelineReadinessReason::RequestedPipelineUnavailableFailPolicy;
    }

    let Some(capability) = report.capability_for(policy.requested) else {
        return TranscodePipelineReadinessReason::RequestedPipelineUnavailableFallbackToCpu;
    };

    if capability.has_probe_error() {
        return TranscodePipelineReadinessReason::ProbeError;
    }

    if capability.device_initialization.status == super::HardwareDeviceInitializationStatus::Failed
    {
        return TranscodePipelineReadinessReason::DeviceInitializationFailed;
    }

    if capability.smoke_probe.status == super::HardwareSmokeProbeStatus::Failed {
        return TranscodePipelineReadinessReason::SmokeProbeFailed;
    }

    TranscodePipelineReadinessReason::RequestedPipelineUnavailableFallbackToCpu
}
