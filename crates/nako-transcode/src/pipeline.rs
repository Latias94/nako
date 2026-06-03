use nako_core::{MediaProbeResult, MediaSource, MediaStreamInfo, NakoError, Result};
use serde::{Deserialize, Serialize};

use super::{
    HardwareAcceleration, HardwareAccelerationCapability, HardwareAccelerationFallback,
    HardwareAccelerationPolicy, HardwareAccelerationReport, HardwarePipelineStage,
    HlsAdaptiveLadderPlan, HlsMediaRenditionPlan, HlsOutputRequirement, HlsPlaybackGeneration,
    HlsRequestVariantPlan, HlsSubtitleBurnInPlan, HlsVariantPolicy, PlaybackHlsProfileRequest,
    TranscodeAccelerationFallbackPlan, TranscodeAccelerationPlan, TranscodeAudioOutputRequirement,
    TranscodeColorPipelineRequirement, TranscodeExecutionPolicy, TranscodeOutputConstraints,
    TranscodePlan, TranscodeProfile, TranscodeProfileIdentity, TranscodeRequestIdentity,
    TranscodeSourceIdentity, TranscodeSubtitleStrategy, TranscodeTrackSelection,
    build_playback_hls_profile,
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

impl TranscodePipelineReadiness {
    const fn cpu_requested() -> Self {
        Self::ready(
            HardwareAcceleration::None,
            TranscodePipelineReadinessReason::CpuRequested,
        )
    }

    const fn requested_pipeline_ready(policy: HardwareAccelerationPolicy) -> Self {
        Self::ready(
            policy.requested,
            TranscodePipelineReadinessReason::RequestedPipelineReady,
        )
    }

    const fn ready(
        selected: HardwareAcceleration,
        reason: TranscodePipelineReadinessReason,
    ) -> Self {
        Self {
            status: TranscodePipelineReadinessStatus::Ready,
            reason,
            requested: selected,
            selected,
            fallback_used: false,
        }
    }

    const fn degraded_to_cpu(
        policy: HardwareAccelerationPolicy,
        reason: TranscodePipelineReadinessReason,
    ) -> Self {
        Self {
            status: TranscodePipelineReadinessStatus::Degraded,
            reason,
            requested: policy.requested,
            selected: HardwareAcceleration::None,
            fallback_used: true,
        }
    }

    const fn unavailable(
        policy: HardwareAccelerationPolicy,
        reason: TranscodePipelineReadinessReason,
        selected: HardwareAcceleration,
    ) -> Self {
        Self {
            status: TranscodePipelineReadinessStatus::Unavailable,
            reason,
            requested: policy.requested,
            selected,
            fallback_used: false,
        }
    }

    const fn acceleration_fallback_plan(
        self,
        fallback: HardwareAccelerationFallback,
    ) -> TranscodeAccelerationFallbackPlan {
        TranscodeAccelerationFallbackPlan {
            requested: self.requested,
            selected: self.selected,
            fallback,
            fallback_used: self.fallback_used,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodePipelineRequest {
    pub hardware_policy: HardwareAccelerationPolicy,
    pub track_selection: TranscodeTrackSelection,
    pub output_constraints: TranscodeOutputConstraints,
    pub subtitle_strategy: TranscodeSubtitleStrategy,
    pub color_pipeline: TranscodeColorPipelineRequirement,
    pub audio_output: TranscodeAudioOutputRequirement,
    pub source: Option<TranscodePipelineSourceFacts>,
}

impl TranscodePipelineRequest {
    #[must_use]
    pub fn hls_single_variant(
        hardware_policy: HardwareAccelerationPolicy,
        track_selection: TranscodeTrackSelection,
        output_constraints: TranscodeOutputConstraints,
    ) -> Self {
        Self::hls_single_variant_with_audio_output(
            hardware_policy,
            track_selection,
            output_constraints,
            TranscodeAudioOutputRequirement::none(),
        )
    }

    #[must_use]
    pub fn hls_single_variant_with_audio_output(
        hardware_policy: HardwareAccelerationPolicy,
        track_selection: TranscodeTrackSelection,
        output_constraints: TranscodeOutputConstraints,
        audio_output: TranscodeAudioOutputRequirement,
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
            color_pipeline: TranscodeColorPipelineRequirement::none(),
            audio_output,
            source: None,
        }
    }

    #[must_use]
    pub const fn with_color_pipeline(
        mut self,
        color_pipeline: TranscodeColorPipelineRequirement,
    ) -> Self {
        self.color_pipeline = color_pipeline;
        self
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
    pub color_pipeline: TranscodeColorPipelineRequirement,
    pub audio_output: TranscodeAudioOutputRequirement,
    pub readiness: TranscodePipelineReadiness,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsRuntimePlanRequest {
    pub source: MediaSource,
    pub plan: TranscodePlan,
    pub hardware_policy: HardwareAccelerationPolicy,
    pub track_selection: TranscodeTrackSelection,
    pub output_constraints: TranscodeOutputConstraints,
    pub audio_output: TranscodeAudioOutputRequirement,
    pub color_pipeline: TranscodeColorPipelineRequirement,
    pub subtitle_strategy: TranscodeSubtitleStrategy,
    pub hls_output: HlsOutputRequirement,
    pub source_facts: Option<TranscodePipelineSourceFacts>,
    pub media_probe: Option<MediaProbeResult>,
    pub playback_generation: HlsPlaybackGeneration,
    pub remote_input: bool,
    pub playback_profile_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsRuntimePlan {
    pub profile: TranscodeProfile,
    pub profile_identity: TranscodeProfileIdentity,
    pub request_variant: HlsRequestVariantPlan,
    pub request_identity: TranscodeRequestIdentity,
    pub pipeline: TranscodePipelinePlan,
    pub execution_policy: TranscodeExecutionPolicy,
    pub hls_output: HlsOutputRequirement,
    pub track_selection: TranscodeTrackSelection,
    pub subtitle_burn_in: Option<HlsSubtitleBurnInPlan>,
}

impl TranscodePipelinePlan {
    #[must_use]
    pub const fn execution_policy(self) -> TranscodeExecutionPolicy {
        TranscodeExecutionPolicy {
            acceleration: self.acceleration,
            output_constraints: self.output_constraints,
            subtitle_strategy: self.subtitle_strategy,
            color_pipeline: self.color_pipeline,
            audio_output: self.audio_output,
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
        validate_color_pipeline_requirement(request.color_pipeline)?;
        let selection = select_pipeline_acceleration(
            request.hardware_policy,
            report,
            request.source.as_ref(),
            request.color_pipeline,
        )?;
        let fallback = selection.acceleration_fallback_plan(request.hardware_policy.fallback);

        Ok(TranscodePipelinePlan {
            acceleration: TranscodeAccelerationPlan::from_pipeline_selection(
                selection.selected,
                fallback,
            ),
            output_constraints: request.output_constraints,
            subtitle_strategy: request.subtitle_strategy,
            color_pipeline: request.color_pipeline,
            audio_output: request.audio_output,
            readiness: selection,
        })
    }

    pub fn plan_hls_runtime(
        &self,
        request: HlsRuntimePlanRequest,
        report: &HardwareAccelerationReport,
    ) -> Result<HlsRuntimePlan> {
        let mut pipeline_request = TranscodePipelineRequest::hls_single_variant_with_audio_output(
            request.hardware_policy,
            request.track_selection,
            request.output_constraints,
            request.audio_output,
        )
        .with_color_pipeline(request.color_pipeline);
        pipeline_request.subtitle_strategy = request.subtitle_strategy;
        if let Some(source_facts) = request.source_facts.clone() {
            pipeline_request = pipeline_request.with_source(source_facts);
        }

        let media_renditions = match request.subtitle_strategy {
            TranscodeSubtitleStrategy::SidecarSelected => {
                HlsMediaRenditionPlan::selected_from_probe(
                    request.media_probe.as_ref(),
                    request.source_facts.as_ref(),
                    request.track_selection,
                )?
            }
            _ => HlsMediaRenditionPlan::selected_from_probe(
                request.media_probe.as_ref(),
                request.source_facts.as_ref(),
                TranscodeTrackSelection {
                    audio_stream: request.track_selection.audio_stream,
                    subtitle_stream: None,
                },
            )?,
        };
        let pipeline = self.plan_hls_single_variant(pipeline_request, report)?;
        let execution_policy = pipeline.execution_policy();
        let subtitle_burn_in = if request.subtitle_strategy
            == TranscodeSubtitleStrategy::BurnInSelected
        {
            Some(
                HlsSubtitleBurnInPlan::selected_from_probe(
                    request.media_probe.as_ref(),
                    request.track_selection,
                )?
                .ok_or_else(|| NakoError::InvalidInput {
                    message: "hls subtitle burn-in requires a selected subtitle stream".to_owned(),
                })?,
            )
        } else {
            None
        };
        let profile = build_playback_hls_profile(PlaybackHlsProfileRequest {
            plan: request.plan,
            execution_policy,
            hls_output: request.hls_output,
            track_selection: request.track_selection,
            remote_input: request.remote_input,
            playback_profile_key: request.playback_profile_key,
        })?;
        let hls_output =
            profile
                .hls_output_requirement()
                .ok_or_else(|| NakoError::InvalidInput {
                    message: "hls transcode profile did not carry HLS output requirements"
                        .to_owned(),
                })?;
        let adaptive_ladder_plan =
            (hls_output.variant_policy == HlsVariantPolicy::Adaptive).then(|| {
                HlsAdaptiveLadderPlan::from_source_facts(
                    request.source_facts.as_ref(),
                    execution_policy.output_constraints,
                )
            });
        let request_variant = HlsRequestVariantPlan::new(adaptive_ladder_plan, media_renditions)
            .with_playback_generation(request.playback_generation);
        let profile_identity = profile.identity();
        let source_identity = TranscodeSourceIdentity::from_media_source(&request.source);
        let request_identity = if let Some(request_variant_key) = request_variant.identity_key() {
            profile_identity.bind_source_with_request_variant(&source_identity, request_variant_key)
        } else {
            profile_identity.bind_source(&source_identity)
        };

        Ok(HlsRuntimePlan {
            profile,
            profile_identity,
            request_variant,
            request_identity,
            pipeline,
            execution_policy,
            hls_output,
            track_selection: request.track_selection,
            subtitle_burn_in,
        })
    }
}

fn select_pipeline_acceleration(
    policy: HardwareAccelerationPolicy,
    report: &HardwareAccelerationReport,
    source: Option<&TranscodePipelineSourceFacts>,
    color_pipeline: TranscodeColorPipelineRequirement,
) -> Result<TranscodePipelineReadiness> {
    if color_pipeline.requires_hdr_to_sdr_tone_mapping() {
        return select_software_hdr_tone_mapping_pipeline(policy, report);
    }

    if policy.requested == HardwareAcceleration::None {
        if !report.is_available(HardwareAcceleration::None) {
            return Err(NakoError::Unsupported(
                "software transcode pipeline is unavailable",
            ));
        }

        return Ok(TranscodePipelineReadiness::cpu_requested());
    }

    if report.is_available(policy.requested) {
        if let Some(reason) = source_incompatibility_reason(
            policy.requested,
            report.capability_for(policy.requested),
            source,
        ) {
            return source_incompatible_readiness(policy, report, reason);
        }

        return Ok(TranscodePipelineReadiness::requested_pipeline_ready(policy));
    }

    let reason = unavailable_reason(policy, report);
    match policy.fallback {
        HardwareAccelerationFallback::Cpu => {
            if !report.is_available(HardwareAcceleration::None) {
                return Err(NakoError::Unsupported(
                    "requested hardware pipeline is unavailable and cpu fallback is unavailable",
                ));
            }

            Ok(TranscodePipelineReadiness::degraded_to_cpu(policy, reason))
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
    select_pipeline_acceleration(
        policy,
        report,
        None,
        TranscodeColorPipelineRequirement::none(),
    )
    .unwrap_or_else(|_| unavailable_pipeline_readiness(policy, report))
}

fn validate_color_pipeline_requirement(
    color_pipeline: TranscodeColorPipelineRequirement,
) -> Result<()> {
    if color_pipeline.is_deferred_unsupported() {
        return Err(NakoError::Unsupported(
            "hls hdr tone mapping for deferred dynamic hdr formats is not implemented",
        ));
    }

    Ok(())
}

fn select_software_hdr_tone_mapping_pipeline(
    policy: HardwareAccelerationPolicy,
    report: &HardwareAccelerationReport,
) -> Result<TranscodePipelineReadiness> {
    if !report.is_available(HardwareAcceleration::None) {
        return Err(NakoError::Unsupported(
            "hdr-to-sdr tone mapping requires the software transcode pipeline",
        ));
    }

    if policy.requested == HardwareAcceleration::None {
        return Ok(TranscodePipelineReadiness::cpu_requested());
    }

    match policy.fallback {
        HardwareAccelerationFallback::Cpu => Ok(TranscodePipelineReadiness::degraded_to_cpu(
            policy,
            TranscodePipelineReadinessReason::RequestedPipelineUnavailableFallbackToCpu,
        )),
        HardwareAccelerationFallback::Fail => Err(NakoError::Unsupported(
            "hdr-to-sdr tone mapping requires software fallback",
        )),
    }
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

            Ok(TranscodePipelineReadiness::degraded_to_cpu(policy, reason))
        }
        HardwareAccelerationFallback::Fail => Err(NakoError::Unsupported(
            "requested hardware pipeline is incompatible with source media",
        )),
    }
}

fn source_incompatibility_reason(
    accelerator: HardwareAcceleration,
    capability: Option<&HardwareAccelerationCapability>,
    source: Option<&TranscodePipelineSourceFacts>,
) -> Option<TranscodePipelineReadinessReason> {
    if !uses_source_aware_hardware_decode(accelerator) {
        return None;
    }

    let video = source.and_then(|source| source.video.as_ref())?;
    if !video.codec.as_deref().is_none_or(|codec| {
        source_video_codec_supported_by_requested_pipeline(accelerator, capability, codec)
    }) {
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

fn source_video_codec_supported_by_requested_pipeline(
    accelerator: HardwareAcceleration,
    capability: Option<&HardwareAccelerationCapability>,
    codec: &str,
) -> bool {
    let Some(codec) = pipeline_source_video_codec(codec) else {
        return false;
    };

    if codec == PipelineSourceVideoCodec::H264 {
        return true;
    }

    let Some(decoder) = source_decoder_stage_feature(accelerator, codec) else {
        return false;
    };

    capability.is_some_and(|capability| {
        capability.has_available_stage_feature(HardwarePipelineStage::Decode, decoder)
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PipelineSourceVideoCodec {
    H264,
    Hevc,
    Av1,
}

fn pipeline_source_video_codec(codec: &str) -> Option<PipelineSourceVideoCodec> {
    let normalized = codec
        .trim()
        .to_ascii_lowercase()
        .replace(['.', '-', '_'], "");

    match normalized.as_str() {
        "h264" | "avc" => Some(PipelineSourceVideoCodec::H264),
        "h265" | "hevc" => Some(PipelineSourceVideoCodec::Hevc),
        "av1" => Some(PipelineSourceVideoCodec::Av1),
        _ => None,
    }
}

fn source_decoder_stage_feature(
    accelerator: HardwareAcceleration,
    codec: PipelineSourceVideoCodec,
) -> Option<&'static str> {
    match (accelerator, codec) {
        (_, PipelineSourceVideoCodec::H264) => Some("h264"),
        (
            HardwareAcceleration::Vaapi | HardwareAcceleration::VideoToolbox,
            PipelineSourceVideoCodec::Hevc,
        ) => Some("hevc"),
        (
            HardwareAcceleration::Vaapi | HardwareAcceleration::VideoToolbox,
            PipelineSourceVideoCodec::Av1,
        ) => Some("av1"),
        (HardwareAcceleration::QuickSync, PipelineSourceVideoCodec::Hevc) => Some("hevc_qsv"),
        (HardwareAcceleration::QuickSync, PipelineSourceVideoCodec::Av1) => Some("av1_qsv"),
        (
            HardwareAcceleration::None | HardwareAcceleration::Nvenc | HardwareAcceleration::Amf,
            PipelineSourceVideoCodec::Hevc | PipelineSourceVideoCodec::Av1,
        ) => None,
    }
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
        unavailable_reason(policy, report)
    };

    let selected = match reason {
        TranscodePipelineReadinessReason::SoftwarePipelineUnavailable
        | TranscodePipelineReadinessReason::CpuFallbackUnavailable => HardwareAcceleration::None,
        _ => policy.requested,
    };

    TranscodePipelineReadiness::unavailable(policy, reason, selected)
}

fn unavailable_reason(
    policy: HardwareAccelerationPolicy,
    report: &HardwareAccelerationReport,
) -> TranscodePipelineReadinessReason {
    let Some(capability) = report.capability_for(policy.requested) else {
        return fallback_unavailable_reason(policy);
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

    fallback_unavailable_reason(policy)
}

fn fallback_unavailable_reason(
    policy: HardwareAccelerationPolicy,
) -> TranscodePipelineReadinessReason {
    match policy.fallback {
        HardwareAccelerationFallback::Cpu => {
            TranscodePipelineReadinessReason::RequestedPipelineUnavailableFallbackToCpu
        }
        HardwareAccelerationFallback::Fail => {
            TranscodePipelineReadinessReason::RequestedPipelineUnavailableFailPolicy
        }
    }
}

#[cfg(test)]
mod tests {
    use nako_core::{
        LibraryId, MediaItemId, MediaProbeResult, MediaSource, MediaSourceId, MediaStreamInfo,
        MediaStreamKind, MediaStreamTechnicalFacts,
    };

    use crate::{
        HlsSegmentContainer, OutputContainer, TranscodeAudioCompatibilityReasons,
        TranscodeAudioDownmixRequirement, TranscodeAudioNormalizationRequirement,
        TranscodeProfileKind,
    };

    use super::*;

    fn demo_source() -> MediaSource {
        MediaSource {
            id: MediaSourceId::new(),
            library_id: LibraryId::new(),
            item_id: MediaItemId::new(),
            locator: "local:///Movies/Demo.mkv".to_owned(),
            file_name: "Demo.mkv".to_owned(),
            size_bytes: Some(42),
            fingerprint: Some("sha256:demo".to_owned()),
        }
    }

    fn media_stream(
        index: u32,
        kind: MediaStreamKind,
        codec: &str,
        language: Option<&str>,
    ) -> MediaStreamInfo {
        MediaStreamInfo {
            index,
            kind,
            codec: Some(codec.to_owned()),
            language: language.map(str::to_owned),
            duration_ms: None,
            bit_rate: None,
            width: None,
            height: None,
            channels: None,
            sample_rate: None,
            technical: MediaStreamTechnicalFacts::default(),
        }
    }

    fn video_stream(width: u32, height: u32, bit_rate: u64) -> MediaStreamInfo {
        MediaStreamInfo {
            width: Some(width),
            height: Some(height),
            bit_rate: Some(bit_rate),
            ..media_stream(0, MediaStreamKind::Video, "h264", None)
        }
    }

    #[test]
    fn hls_pipeline_plan_carries_audio_output_requirement_into_execution_policy() {
        let audio_output = TranscodeAudioOutputRequirement {
            source_channels: Some(6),
            max_supported_channels: Some(2),
            target_channels: Some(2),
            downmix: TranscodeAudioDownmixRequirement::Required,
            normalization: TranscodeAudioNormalizationRequirement::Requested,
            reasons: TranscodeAudioCompatibilityReasons {
                channel_limit_exceeded: true,
                downmix_required: true,
                normalization_requested: true,
            },
        };
        let request = TranscodePipelineRequest::hls_single_variant_with_audio_output(
            HardwareAccelerationPolicy::default(),
            TranscodeTrackSelection {
                audio_stream: Some(1),
                subtitle_stream: None,
            },
            TranscodeOutputConstraints::default(),
            audio_output,
        );
        let report = HardwareAccelerationReport::with_available([HardwareAcceleration::None]);

        let plan = TranscodePipelinePlanner::new()
            .plan_hls_single_variant(request, &report)
            .unwrap();

        assert_eq!(plan.audio_output, audio_output);
        assert_eq!(plan.execution_policy().audio_output, audio_output);
    }

    #[test]
    fn hls_pipeline_hdr_to_sdr_tone_mapping_falls_back_to_software_execution_policy() {
        let color_pipeline = TranscodeColorPipelineRequirement::hdr_to_sdr_required();
        let request = TranscodePipelineRequest::hls_single_variant(
            HardwareAccelerationPolicy {
                requested: HardwareAcceleration::Nvenc,
                fallback: HardwareAccelerationFallback::Cpu,
            },
            TranscodeTrackSelection::default(),
            TranscodeOutputConstraints::default(),
        )
        .with_color_pipeline(color_pipeline);
        let report = HardwareAccelerationReport::with_available([
            HardwareAcceleration::None,
            HardwareAcceleration::Nvenc,
        ]);

        let plan = TranscodePipelinePlanner::new()
            .plan_hls_single_variant(request, &report)
            .unwrap();

        assert_eq!(plan.color_pipeline, color_pipeline);
        assert_eq!(
            plan.readiness.status,
            TranscodePipelineReadinessStatus::Degraded
        );
        assert_eq!(
            plan.readiness.reason,
            TranscodePipelineReadinessReason::RequestedPipelineUnavailableFallbackToCpu
        );
        assert_eq!(plan.readiness.selected, HardwareAcceleration::None);
        assert!(plan.fallback_used());
        assert!(plan.execution_policy().acceleration.is_software_only());
        assert_eq!(plan.execution_policy().color_pipeline, color_pipeline);
    }

    #[test]
    fn hls_pipeline_hdr_deferred_tone_mapping_is_not_executable() {
        let request = TranscodePipelineRequest::hls_single_variant(
            HardwareAccelerationPolicy::default(),
            TranscodeTrackSelection::default(),
            TranscodeOutputConstraints::default(),
        )
        .with_color_pipeline(TranscodeColorPipelineRequirement::hdr_to_sdr_deferred_unsupported());
        let report = HardwareAccelerationReport::with_available([HardwareAcceleration::None]);

        let err = TranscodePipelinePlanner::new()
            .plan_hls_single_variant(request, &report)
            .unwrap_err();

        assert!(err.to_string().contains("deferred dynamic hdr"));
    }

    #[test]
    fn hls_pipeline_source_fallback_plan_mirrors_readiness() {
        let request = TranscodePipelineRequest::hls_single_variant(
            HardwareAccelerationPolicy {
                requested: HardwareAcceleration::Vaapi,
                fallback: HardwareAccelerationFallback::Cpu,
            },
            TranscodeTrackSelection::default(),
            TranscodeOutputConstraints::default(),
        )
        .with_source(TranscodePipelineSourceFacts {
            video: Some(media_stream(0, MediaStreamKind::Video, "hevc", None)),
            ..TranscodePipelineSourceFacts::default()
        });
        let report = HardwareAccelerationReport::with_available([
            HardwareAcceleration::None,
            HardwareAcceleration::Vaapi,
        ]);

        let plan = TranscodePipelinePlanner::new()
            .plan_hls_single_variant(request, &report)
            .unwrap();

        assert_eq!(
            plan.readiness.status,
            TranscodePipelineReadinessStatus::Degraded
        );
        assert_eq!(
            plan.readiness.reason,
            TranscodePipelineReadinessReason::SourceVideoCodecUnsupportedByRequestedPipeline
        );
        assert_eq!(plan.readiness.requested, HardwareAcceleration::Vaapi);
        assert_eq!(plan.readiness.selected, HardwareAcceleration::None);
        assert!(plan.readiness.fallback_used);
        assert_eq!(
            plan.acceleration.fallback.requested,
            plan.readiness.requested
        );
        assert_eq!(plan.acceleration.fallback.selected, plan.readiness.selected);
        assert_eq!(
            plan.acceleration.fallback.fallback,
            HardwareAccelerationFallback::Cpu
        );
        assert_eq!(
            plan.acceleration.fallback.fallback_used,
            plan.readiness.fallback_used
        );
    }

    #[test]
    fn hls_runtime_plan_carries_audio_output_and_request_variant_identity() {
        let source = demo_source();
        let video = video_stream(1920, 1080, 4_000_000);
        let selected_audio = MediaStreamInfo {
            channels: Some(6),
            ..media_stream(1, MediaStreamKind::Audio, "aac", Some("eng"))
        };
        let alternate_audio = MediaStreamInfo {
            channels: Some(2),
            ..media_stream(2, MediaStreamKind::Audio, "aac", Some("jpn"))
        };
        let selected_subtitle = media_stream(3, MediaStreamKind::Subtitle, "subrip", Some("jpn"));
        let audio_output = TranscodeAudioOutputRequirement {
            source_channels: Some(6),
            max_supported_channels: Some(2),
            target_channels: Some(2),
            downmix: TranscodeAudioDownmixRequirement::Required,
            normalization: TranscodeAudioNormalizationRequirement::Requested,
            reasons: TranscodeAudioCompatibilityReasons {
                channel_limit_exceeded: true,
                downmix_required: true,
                normalization_requested: true,
            },
        };
        let track_selection = TranscodeTrackSelection {
            audio_stream: Some(1),
            subtitle_stream: Some(3),
        };
        let request = HlsRuntimePlanRequest {
            source,
            plan: TranscodePlan {
                input_locator: "local:///Movies/Demo.mkv".to_owned(),
                output_container: OutputContainer::Hls,
                video_codec: Some("h264".to_owned()),
                audio_codec: Some("aac".to_owned()),
            },
            hardware_policy: HardwareAccelerationPolicy::default(),
            track_selection,
            output_constraints: TranscodeOutputConstraints {
                max_video_bitrate: Some(2_000_000),
                max_width: Some(1280),
                max_height: Some(720),
                prefer_hdr: Some(false),
            },
            audio_output,
            color_pipeline: TranscodeColorPipelineRequirement::none(),
            subtitle_strategy: TranscodeSubtitleStrategy::SidecarSelected,
            hls_output: HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::Adaptive,
                segment_container: HlsSegmentContainer::Fmp4,
            },
            source_facts: Some(TranscodePipelineSourceFacts {
                video: Some(video.clone()),
                audio: Some(selected_audio.clone()),
                subtitle: Some(selected_subtitle.clone()),
            }),
            media_probe: Some(MediaProbeResult {
                duration_ms: Some(1_000),
                container: Some("matroska,webm".to_owned()),
                bit_rate: None,
                streams: vec![video, selected_audio, alternate_audio, selected_subtitle],
            }),
            playback_generation: HlsPlaybackGeneration::from_start_position_ms(45_000),
            remote_input: true,
            playback_profile_key: "playback-target-profile:v1;demo=true".to_owned(),
        };
        let report = HardwareAccelerationReport::with_available([HardwareAcceleration::None]);

        let runtime = TranscodePipelinePlanner::new()
            .plan_hls_runtime(request, &report)
            .unwrap();

        assert_eq!(runtime.profile.kind(), TranscodeProfileKind::HlsAdaptive);
        assert_eq!(runtime.execution_policy.audio_output, audio_output);
        assert_eq!(
            runtime.execution_policy.subtitle_strategy,
            TranscodeSubtitleStrategy::SidecarSelected
        );
        assert_eq!(runtime.track_selection, track_selection);
        assert!(runtime.request_variant.media_renditions.has_audios());
        assert!(runtime.request_variant.media_renditions.has_subtitles());
        assert_eq!(
            runtime.request_variant.playback_generation,
            HlsPlaybackGeneration::from_start_position_ms(45_000)
        );
        assert!(runtime.request_variant.adaptive_ladder.is_some());

        let request_key = runtime.request_identity.persisted_request_key();
        assert!(request_key.contains(";request_variant=hls-request-variant:v1"));
        assert!(request_key.contains("hls-adaptive-ladder:v1"));
        assert!(request_key.contains("hls-media-renditions:v1"));
        assert!(request_key.contains("hls-main-output:v1"));
        assert!(request_key.contains("hls-playback-generation:v1"));
    }

    #[test]
    fn hls_runtime_plan_preserves_burn_in_strategy_without_subtitle_renditions() {
        let source = demo_source();
        let video = video_stream(1920, 1080, 4_000_000);
        let subtitle = media_stream(3, MediaStreamKind::Subtitle, "subrip", Some("jpn"));
        let track_selection = TranscodeTrackSelection {
            audio_stream: None,
            subtitle_stream: Some(3),
        };
        let request = HlsRuntimePlanRequest {
            source,
            plan: TranscodePlan {
                input_locator: "local:///Movies/Demo.mkv".to_owned(),
                output_container: OutputContainer::Hls,
                video_codec: Some("h264".to_owned()),
                audio_codec: Some("aac".to_owned()),
            },
            hardware_policy: HardwareAccelerationPolicy::default(),
            track_selection,
            output_constraints: TranscodeOutputConstraints::default(),
            audio_output: TranscodeAudioOutputRequirement::none(),
            color_pipeline: TranscodeColorPipelineRequirement::none(),
            subtitle_strategy: TranscodeSubtitleStrategy::BurnInSelected,
            hls_output: HlsOutputRequirement::default(),
            source_facts: Some(TranscodePipelineSourceFacts {
                video: Some(video.clone()),
                audio: None,
                subtitle: Some(subtitle.clone()),
            }),
            media_probe: Some(MediaProbeResult {
                duration_ms: Some(1_000),
                container: Some("matroska,webm".to_owned()),
                bit_rate: None,
                streams: vec![video, subtitle],
            }),
            playback_generation: HlsPlaybackGeneration::default(),
            remote_input: false,
            playback_profile_key: "playback-target-profile:v1;demo=true".to_owned(),
        };
        let report = HardwareAccelerationReport::with_available([HardwareAcceleration::None]);

        let runtime = TranscodePipelinePlanner::new()
            .plan_hls_runtime(request, &report)
            .unwrap();

        assert_eq!(
            runtime.execution_policy.subtitle_strategy,
            TranscodeSubtitleStrategy::BurnInSelected
        );
        assert_eq!(
            runtime.subtitle_burn_in,
            Some(HlsSubtitleBurnInPlan::new(3, 0))
        );
        assert!(runtime.request_variant.media_renditions.is_empty());
        assert!(runtime.request_variant.identity_key().is_none());
        assert!(
            runtime
                .profile_identity
                .persisted_request_key()
                .contains("subtitle_strategy=burn_in_selected")
        );
    }

    #[test]
    fn hls_runtime_plan_does_not_infer_sidecar_from_omitted_subtitle_selection() {
        let source = demo_source();
        let video = video_stream(1920, 1080, 4_000_000);
        let selected_audio = media_stream(1, MediaStreamKind::Audio, "aac", Some("eng"));
        let alternate_audio = media_stream(2, MediaStreamKind::Audio, "aac", Some("jpn"));
        let subtitle = media_stream(3, MediaStreamKind::Subtitle, "subrip", Some("jpn"));
        let track_selection = TranscodeTrackSelection {
            audio_stream: Some(1),
            subtitle_stream: Some(3),
        };
        let request = HlsRuntimePlanRequest {
            source,
            plan: TranscodePlan {
                input_locator: "local:///Movies/Demo.mkv".to_owned(),
                output_container: OutputContainer::Hls,
                video_codec: Some("h264".to_owned()),
                audio_codec: Some("aac".to_owned()),
            },
            hardware_policy: HardwareAccelerationPolicy::default(),
            track_selection,
            output_constraints: TranscodeOutputConstraints::default(),
            audio_output: TranscodeAudioOutputRequirement::none(),
            color_pipeline: TranscodeColorPipelineRequirement::none(),
            subtitle_strategy: TranscodeSubtitleStrategy::OmitSelected,
            hls_output: HlsOutputRequirement::default(),
            source_facts: Some(TranscodePipelineSourceFacts {
                video: Some(video.clone()),
                audio: Some(selected_audio.clone()),
                subtitle: Some(subtitle.clone()),
            }),
            media_probe: Some(MediaProbeResult {
                duration_ms: Some(1_000),
                container: Some("matroska,webm".to_owned()),
                bit_rate: None,
                streams: vec![video, selected_audio, alternate_audio, subtitle],
            }),
            playback_generation: HlsPlaybackGeneration::default(),
            remote_input: false,
            playback_profile_key: "playback-target-profile:v1;demo=true".to_owned(),
        };
        let report = HardwareAccelerationReport::with_available([HardwareAcceleration::None]);

        let runtime = TranscodePipelinePlanner::new()
            .plan_hls_runtime(request, &report)
            .unwrap();

        assert_eq!(
            runtime.execution_policy.subtitle_strategy,
            TranscodeSubtitleStrategy::OmitSelected
        );
        assert!(runtime.request_variant.media_renditions.has_audios());
        assert!(!runtime.request_variant.media_renditions.has_subtitles());
        assert!(
            runtime
                .profile_identity
                .persisted_request_key()
                .contains("subtitle_strategy=omit_selected")
        );
    }

    #[test]
    fn hls_runtime_plan_carries_hdr_color_pipeline_into_profile_identity() {
        let source = demo_source();
        let color_pipeline = TranscodeColorPipelineRequirement::hdr_to_sdr_required();
        let request = HlsRuntimePlanRequest {
            source,
            plan: TranscodePlan {
                input_locator: "local:///Movies/Demo.mkv".to_owned(),
                output_container: OutputContainer::Hls,
                video_codec: Some("h264".to_owned()),
                audio_codec: Some("aac".to_owned()),
            },
            hardware_policy: HardwareAccelerationPolicy::default(),
            track_selection: TranscodeTrackSelection::default(),
            output_constraints: TranscodeOutputConstraints::default(),
            audio_output: TranscodeAudioOutputRequirement::none(),
            color_pipeline,
            subtitle_strategy: TranscodeSubtitleStrategy::None,
            hls_output: HlsOutputRequirement::default(),
            source_facts: None,
            media_probe: None,
            playback_generation: HlsPlaybackGeneration::default(),
            remote_input: false,
            playback_profile_key: "playback-target-profile:v1;hdr=false".to_owned(),
        };
        let report = HardwareAccelerationReport::with_available([HardwareAcceleration::None]);

        let runtime = TranscodePipelinePlanner::new()
            .plan_hls_runtime(request, &report)
            .unwrap();

        assert_eq!(runtime.execution_policy.color_pipeline, color_pipeline);
        assert!(runtime.profile_identity.persisted_request_key().contains(
            "color_pipeline=target:sdr,tone_mapping:required,reasons:source_hdr_detected|client_hdr_unsupported|tone_mapping_required"
        ));
    }

    #[test]
    fn hls_runtime_plan_keeps_default_single_variant_request_identity_plain() {
        let source = demo_source();
        let request = HlsRuntimePlanRequest {
            source,
            plan: TranscodePlan {
                input_locator: "local:///Movies/Demo.mkv".to_owned(),
                output_container: OutputContainer::Hls,
                video_codec: Some("h264".to_owned()),
                audio_codec: Some("aac".to_owned()),
            },
            hardware_policy: HardwareAccelerationPolicy::default(),
            track_selection: TranscodeTrackSelection::default(),
            output_constraints: TranscodeOutputConstraints::default(),
            audio_output: TranscodeAudioOutputRequirement::none(),
            color_pipeline: TranscodeColorPipelineRequirement::none(),
            subtitle_strategy: TranscodeSubtitleStrategy::None,
            hls_output: HlsOutputRequirement::default(),
            source_facts: None,
            media_probe: None,
            playback_generation: HlsPlaybackGeneration::default(),
            remote_input: false,
            playback_profile_key: "playback-target-profile:v1;demo=true".to_owned(),
        };
        let report = HardwareAccelerationReport::with_available([HardwareAcceleration::None]);

        let runtime = TranscodePipelinePlanner::new()
            .plan_hls_runtime(request, &report)
            .unwrap();

        assert_eq!(
            runtime.profile.kind(),
            TranscodeProfileKind::HlsSingleVariant
        );
        assert_eq!(runtime.hls_output, HlsOutputRequirement::default());
        assert!(runtime.request_variant.is_empty());
        assert!(
            !runtime
                .request_identity
                .persisted_request_key()
                .contains(";request_variant=")
        );
    }
}
