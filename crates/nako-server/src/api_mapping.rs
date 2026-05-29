use nako_api::admin::{
    AdminHardwareAcceleration, AdminHardwareAccelerationFallback, AdminHardwareAccelerationPolicy,
    AdminHardwarePipelineStage, AdminTranscodePipelineReadiness,
    AdminTranscodePipelineReadinessReason, AdminTranscodePipelineReadinessStatus,
};
use nako_transcode::{
    HardwareAcceleration, HardwareAccelerationFallback, HardwareAccelerationPolicy,
    HardwarePipelineStage, TranscodePipelineReadiness, TranscodePipelineReadinessReason,
    TranscodePipelineReadinessStatus,
};

pub(crate) const fn admin_hardware_acceleration(
    acceleration: HardwareAcceleration,
) -> AdminHardwareAcceleration {
    match acceleration {
        HardwareAcceleration::None => AdminHardwareAcceleration::None,
        HardwareAcceleration::Vaapi => AdminHardwareAcceleration::Vaapi,
        HardwareAcceleration::Nvenc => AdminHardwareAcceleration::Nvenc,
        HardwareAcceleration::QuickSync => AdminHardwareAcceleration::QuickSync,
        HardwareAcceleration::Amf => AdminHardwareAcceleration::Amf,
        HardwareAcceleration::VideoToolbox => AdminHardwareAcceleration::VideoToolbox,
    }
}

pub(crate) const fn transcode_hardware_acceleration(
    acceleration: AdminHardwareAcceleration,
) -> HardwareAcceleration {
    match acceleration {
        AdminHardwareAcceleration::None => HardwareAcceleration::None,
        AdminHardwareAcceleration::Vaapi => HardwareAcceleration::Vaapi,
        AdminHardwareAcceleration::Nvenc => HardwareAcceleration::Nvenc,
        AdminHardwareAcceleration::QuickSync => HardwareAcceleration::QuickSync,
        AdminHardwareAcceleration::Amf => HardwareAcceleration::Amf,
        AdminHardwareAcceleration::VideoToolbox => HardwareAcceleration::VideoToolbox,
    }
}

pub(crate) const fn admin_hardware_fallback(
    fallback: HardwareAccelerationFallback,
) -> AdminHardwareAccelerationFallback {
    match fallback {
        HardwareAccelerationFallback::Cpu => AdminHardwareAccelerationFallback::Cpu,
        HardwareAccelerationFallback::Fail => AdminHardwareAccelerationFallback::Fail,
    }
}

pub(crate) const fn transcode_hardware_fallback(
    fallback: AdminHardwareAccelerationFallback,
) -> HardwareAccelerationFallback {
    match fallback {
        AdminHardwareAccelerationFallback::Cpu => HardwareAccelerationFallback::Cpu,
        AdminHardwareAccelerationFallback::Fail => HardwareAccelerationFallback::Fail,
    }
}

pub(crate) const fn admin_hardware_policy(
    policy: HardwareAccelerationPolicy,
) -> AdminHardwareAccelerationPolicy {
    AdminHardwareAccelerationPolicy {
        requested: admin_hardware_acceleration(policy.requested),
        fallback: admin_hardware_fallback(policy.fallback),
    }
}

pub(crate) const fn admin_transcode_pipeline_readiness(
    readiness: TranscodePipelineReadiness,
) -> AdminTranscodePipelineReadiness {
    AdminTranscodePipelineReadiness {
        status: admin_transcode_pipeline_readiness_status(readiness.status),
        reason: admin_transcode_pipeline_readiness_reason(readiness.reason),
        requested: admin_hardware_acceleration(readiness.requested),
        selected: admin_hardware_acceleration(readiness.selected),
        fallback_used: readiness.fallback_used,
    }
}

const fn admin_transcode_pipeline_readiness_status(
    status: TranscodePipelineReadinessStatus,
) -> AdminTranscodePipelineReadinessStatus {
    match status {
        TranscodePipelineReadinessStatus::Ready => AdminTranscodePipelineReadinessStatus::Ready,
        TranscodePipelineReadinessStatus::Degraded => {
            AdminTranscodePipelineReadinessStatus::Degraded
        }
        TranscodePipelineReadinessStatus::Unavailable => {
            AdminTranscodePipelineReadinessStatus::Unavailable
        }
    }
}

const fn admin_transcode_pipeline_readiness_reason(
    reason: TranscodePipelineReadinessReason,
) -> AdminTranscodePipelineReadinessReason {
    match reason {
        TranscodePipelineReadinessReason::CpuRequested => {
            AdminTranscodePipelineReadinessReason::CpuRequested
        }
        TranscodePipelineReadinessReason::RequestedPipelineReady => {
            AdminTranscodePipelineReadinessReason::RequestedPipelineReady
        }
        TranscodePipelineReadinessReason::RequestedPipelineUnavailableFallbackToCpu => {
            AdminTranscodePipelineReadinessReason::RequestedPipelineUnavailableFallbackToCpu
        }
        TranscodePipelineReadinessReason::RequestedPipelineUnavailableFailPolicy => {
            AdminTranscodePipelineReadinessReason::RequestedPipelineUnavailableFailPolicy
        }
        TranscodePipelineReadinessReason::SoftwarePipelineUnavailable => {
            AdminTranscodePipelineReadinessReason::SoftwarePipelineUnavailable
        }
        TranscodePipelineReadinessReason::CpuFallbackUnavailable => {
            AdminTranscodePipelineReadinessReason::CpuFallbackUnavailable
        }
        TranscodePipelineReadinessReason::ProbeError => {
            AdminTranscodePipelineReadinessReason::ProbeError
        }
        TranscodePipelineReadinessReason::DeviceInitializationFailed => {
            AdminTranscodePipelineReadinessReason::DeviceInitializationFailed
        }
        TranscodePipelineReadinessReason::SmokeProbeFailed => {
            AdminTranscodePipelineReadinessReason::SmokeProbeFailed
        }
        TranscodePipelineReadinessReason::SourceVideoCodecUnsupportedByRequestedPipeline => {
            AdminTranscodePipelineReadinessReason::SourceVideoCodecUnsupportedByRequestedPipeline
        }
        TranscodePipelineReadinessReason::SourceVideoBitDepthUnsupportedByRequestedPipeline => {
            AdminTranscodePipelineReadinessReason::SourceVideoBitDepthUnsupportedByRequestedPipeline
        }
    }
}

pub(crate) const fn admin_hardware_pipeline_stage(
    stage: HardwarePipelineStage,
) -> AdminHardwarePipelineStage {
    match stage {
        HardwarePipelineStage::Decode => AdminHardwarePipelineStage::Decode,
        HardwarePipelineStage::Filter => AdminHardwarePipelineStage::Filter,
        HardwarePipelineStage::Encode => AdminHardwarePipelineStage::Encode,
        HardwarePipelineStage::Hwaccel => AdminHardwarePipelineStage::Hwaccel,
        HardwarePipelineStage::BitstreamFilter => AdminHardwarePipelineStage::BitstreamFilter,
    }
}
