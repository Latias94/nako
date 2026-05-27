use std::path::PathBuf;

use nako_core::{NakoError, Result};
use serde::{Deserialize, Serialize};

use super::ffmpeg::stderr_message;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareAcceleration {
    #[default]
    None,
    Vaapi,
    Nvenc,
    QuickSync,
    Amf,
    VideoToolbox,
}

impl HardwareAcceleration {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Vaapi => "vaapi",
            Self::Nvenc => "nvenc",
            Self::QuickSync => "quick_sync",
            Self::Amf => "amf",
            Self::VideoToolbox => "video_toolbox",
        }
    }

    #[must_use]
    pub const fn is_gpu(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareAccelerationFallback {
    #[default]
    Cpu,
    Fail,
}

impl HardwareAccelerationFallback {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Fail => "fail",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareAccelerationPolicy {
    pub requested: HardwareAcceleration,
    pub fallback: HardwareAccelerationFallback,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareEncoderDiscoveryStatus {
    NotRequired,
    Listed,
    Missing,
    ProbeError,
    Static,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwarePipelineStage {
    Decode,
    Filter,
    Encode,
    Hwaccel,
    BitstreamFilter,
}

impl HardwarePipelineStage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Decode => "decode",
            Self::Filter => "filter",
            Self::Encode => "encode",
            Self::Hwaccel => "hwaccel",
            Self::BitstreamFilter => "bitstream_filter",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareStageCapability {
    pub stage: HardwarePipelineStage,
    pub available: bool,
    pub feature: Option<String>,
    pub discovery_status: HardwareEncoderDiscoveryStatus,
    pub detail: Option<String>,
}

impl HardwareStageCapability {
    #[must_use]
    pub fn not_required(stage: HardwarePipelineStage) -> Self {
        Self {
            stage,
            available: true,
            feature: None,
            discovery_status: HardwareEncoderDiscoveryStatus::NotRequired,
            detail: None,
        }
    }

    #[must_use]
    pub fn static_available(stage: HardwarePipelineStage, feature: impl Into<String>) -> Self {
        Self {
            stage,
            available: true,
            feature: Some(feature.into()),
            discovery_status: HardwareEncoderDiscoveryStatus::Static,
            detail: None,
        }
    }

    #[must_use]
    pub fn listed(stage: HardwarePipelineStage, feature: impl Into<String>) -> Self {
        Self {
            stage,
            available: true,
            feature: Some(feature.into()),
            discovery_status: HardwareEncoderDiscoveryStatus::Listed,
            detail: None,
        }
    }

    #[must_use]
    pub fn missing(stage: HardwarePipelineStage, feature: impl Into<String>) -> Self {
        Self {
            stage,
            available: false,
            feature: Some(feature.into()),
            discovery_status: HardwareEncoderDiscoveryStatus::Missing,
            detail: None,
        }
    }

    #[must_use]
    pub fn probe_error(stage: HardwarePipelineStage, detail: impl Into<String>) -> Self {
        Self {
            stage,
            available: false,
            feature: None,
            discovery_status: HardwareEncoderDiscoveryStatus::ProbeError,
            detail: Some(detail.into()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareEncoderDiscovery {
    pub status: HardwareEncoderDiscoveryStatus,
    pub encoder: Option<String>,
    pub detail: Option<String>,
}

impl HardwareEncoderDiscovery {
    #[must_use]
    pub fn not_required() -> Self {
        Self {
            status: HardwareEncoderDiscoveryStatus::NotRequired,
            encoder: None,
            detail: None,
        }
    }

    #[must_use]
    pub fn listed(encoder: impl Into<String>) -> Self {
        Self {
            status: HardwareEncoderDiscoveryStatus::Listed,
            encoder: Some(encoder.into()),
            detail: None,
        }
    }

    #[must_use]
    pub fn missing(encoder: impl Into<String>) -> Self {
        Self {
            status: HardwareEncoderDiscoveryStatus::Missing,
            encoder: Some(encoder.into()),
            detail: None,
        }
    }

    #[must_use]
    pub fn probe_error(detail: impl Into<String>) -> Self {
        Self {
            status: HardwareEncoderDiscoveryStatus::ProbeError,
            encoder: None,
            detail: Some(detail.into()),
        }
    }

    #[must_use]
    pub fn static_detector() -> Self {
        Self {
            status: HardwareEncoderDiscoveryStatus::Static,
            encoder: None,
            detail: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareDeviceInitializationStatus {
    NotRequired,
    NotRun,
    Passed,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareDeviceInitialization {
    pub status: HardwareDeviceInitializationStatus,
    pub operator_check: String,
    pub detail: Option<String>,
}

impl HardwareDeviceInitialization {
    #[must_use]
    pub fn not_required() -> Self {
        Self {
            status: HardwareDeviceInitializationStatus::NotRequired,
            operator_check: "cpu encode does not require hardware device initialization".to_owned(),
            detail: None,
        }
    }

    #[must_use]
    pub fn not_run(accelerator: HardwareAcceleration) -> Self {
        Self {
            status: HardwareDeviceInitializationStatus::NotRun,
            operator_check: operator_device_initialization_check(accelerator).to_owned(),
            detail: None,
        }
    }

    #[must_use]
    pub fn passed(accelerator: HardwareAcceleration) -> Self {
        Self {
            status: HardwareDeviceInitializationStatus::Passed,
            operator_check: operator_device_initialization_check(accelerator).to_owned(),
            detail: None,
        }
    }

    #[must_use]
    pub fn failed(accelerator: HardwareAcceleration, detail: impl Into<String>) -> Self {
        Self {
            status: HardwareDeviceInitializationStatus::Failed,
            operator_check: operator_device_initialization_check(accelerator).to_owned(),
            detail: Some(detail.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareSmokeProbeStatus {
    NotRequired,
    NotRun,
    Passed,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareSmokeProbe {
    pub status: HardwareSmokeProbeStatus,
    pub operator_check: String,
    pub detail: Option<String>,
}

impl HardwareSmokeProbe {
    #[must_use]
    pub fn not_required() -> Self {
        Self {
            status: HardwareSmokeProbeStatus::NotRequired,
            operator_check: "cpu encode does not require a hardware smoke probe".to_owned(),
            detail: None,
        }
    }

    #[must_use]
    pub fn not_run(accelerator: HardwareAcceleration) -> Self {
        Self {
            status: HardwareSmokeProbeStatus::NotRun,
            operator_check: operator_smoke_check(accelerator).to_owned(),
            detail: None,
        }
    }

    #[must_use]
    pub fn passed(accelerator: HardwareAcceleration) -> Self {
        Self {
            status: HardwareSmokeProbeStatus::Passed,
            operator_check: operator_smoke_check(accelerator).to_owned(),
            detail: None,
        }
    }

    #[must_use]
    pub fn failed(accelerator: HardwareAcceleration, detail: impl Into<String>) -> Self {
        Self {
            status: HardwareSmokeProbeStatus::Failed,
            operator_check: operator_smoke_check(accelerator).to_owned(),
            detail: Some(detail.into()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareAccelerationCapability {
    pub accelerator: HardwareAcceleration,
    pub available: bool,
    pub device: Option<String>,
    pub reason: Option<String>,
    pub stage_capabilities: Vec<HardwareStageCapability>,
    pub encoder_discovery: HardwareEncoderDiscovery,
    pub device_initialization: HardwareDeviceInitialization,
    pub smoke_probe: HardwareSmokeProbe,
}

impl HardwareAccelerationCapability {
    #[must_use]
    pub fn has_probe_error(&self) -> bool {
        self.encoder_discovery.status == HardwareEncoderDiscoveryStatus::ProbeError
            || self
                .stage_capabilities
                .iter()
                .any(|stage| stage.discovery_status == HardwareEncoderDiscoveryStatus::ProbeError)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareAccelerationReport {
    pub capabilities: Vec<HardwareAccelerationCapability>,
}

impl HardwareAccelerationReport {
    #[must_use]
    pub fn cpu_only() -> Self {
        Self {
            capabilities: vec![cpu_capability()],
        }
    }

    #[must_use]
    pub fn with_available(accelerators: impl IntoIterator<Item = HardwareAcceleration>) -> Self {
        Self {
            capabilities: accelerators.into_iter().map(static_capability).collect(),
        }
    }

    #[must_use]
    pub fn is_available(&self, accelerator: HardwareAcceleration) -> bool {
        accelerator == HardwareAcceleration::None
            || self
                .capabilities
                .iter()
                .any(|capability| capability.accelerator == accelerator && capability.available)
    }

    #[must_use]
    pub fn capability_for(
        &self,
        accelerator: HardwareAcceleration,
    ) -> Option<&HardwareAccelerationCapability> {
        self.capabilities
            .iter()
            .find(|capability| capability.accelerator == accelerator)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeResourceBudget {
    pub cpu_slots: usize,
    pub gpu_slots: usize,
}

impl Default for TranscodeResourceBudget {
    fn default() -> Self {
        Self {
            cpu_slots: 1,
            gpu_slots: 1,
        }
    }
}

impl TranscodeResourceBudget {
    #[must_use]
    pub const fn new(cpu_slots: usize, gpu_slots: usize) -> Self {
        Self {
            cpu_slots,
            gpu_slots,
        }
    }

    #[must_use]
    pub fn bounded(self) -> Self {
        Self {
            cpu_slots: self.cpu_slots.max(1),
            gpu_slots: self.gpu_slots.max(1),
        }
    }

    #[must_use]
    pub fn slots_for(self, acceleration: HardwareAcceleration) -> usize {
        let budget = self.bounded();
        if acceleration.is_gpu() {
            budget.gpu_slots
        } else {
            budget.cpu_slots
        }
    }
}

pub trait HardwareAccelerationDetector: Send + Sync {
    fn detect(&self) -> HardwareAccelerationReport;
}

pub trait HardwareSmokeProbeDetector: Send + Sync {
    fn probe(&self, accelerator: HardwareAcceleration) -> HardwareSmokeProbe;
}

pub trait HardwareDeviceInitializationDetector: Send + Sync {
    fn initialize(&self, accelerator: HardwareAcceleration) -> HardwareDeviceInitialization;
}

#[derive(Clone, Debug, Default)]
pub struct OperatorHardwareDeviceInitialization;

impl HardwareDeviceInitializationDetector for OperatorHardwareDeviceInitialization {
    fn initialize(&self, accelerator: HardwareAcceleration) -> HardwareDeviceInitialization {
        if accelerator == HardwareAcceleration::None {
            HardwareDeviceInitialization::not_required()
        } else {
            HardwareDeviceInitialization::not_run(accelerator)
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StaticHardwareDeviceInitialization {
    results: Vec<(HardwareAcceleration, HardwareDeviceInitialization)>,
}

impl StaticHardwareDeviceInitialization {
    #[must_use]
    pub fn new(
        results: impl IntoIterator<Item = (HardwareAcceleration, HardwareDeviceInitialization)>,
    ) -> Self {
        Self {
            results: results.into_iter().collect(),
        }
    }
}

impl HardwareDeviceInitializationDetector for StaticHardwareDeviceInitialization {
    fn initialize(&self, accelerator: HardwareAcceleration) -> HardwareDeviceInitialization {
        self.results
            .iter()
            .find(|(candidate, _)| *candidate == accelerator)
            .map(|(_, initialization)| initialization.clone())
            .unwrap_or_else(|| OperatorHardwareDeviceInitialization.initialize(accelerator))
    }
}

#[derive(Clone, Debug, Default)]
pub struct OperatorHardwareSmokeProbe;

impl HardwareSmokeProbeDetector for OperatorHardwareSmokeProbe {
    fn probe(&self, accelerator: HardwareAcceleration) -> HardwareSmokeProbe {
        if accelerator == HardwareAcceleration::None {
            HardwareSmokeProbe::not_required()
        } else {
            HardwareSmokeProbe::not_run(accelerator)
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StaticHardwareSmokeProbe {
    probes: Vec<(HardwareAcceleration, HardwareSmokeProbe)>,
}

impl StaticHardwareSmokeProbe {
    #[must_use]
    pub fn new(
        probes: impl IntoIterator<Item = (HardwareAcceleration, HardwareSmokeProbe)>,
    ) -> Self {
        Self {
            probes: probes.into_iter().collect(),
        }
    }
}

impl HardwareSmokeProbeDetector for StaticHardwareSmokeProbe {
    fn probe(&self, accelerator: HardwareAcceleration) -> HardwareSmokeProbe {
        self.probes
            .iter()
            .find(|(candidate, _)| *candidate == accelerator)
            .map(|(_, probe)| probe.clone())
            .unwrap_or_else(|| OperatorHardwareSmokeProbe.probe(accelerator))
    }
}

#[derive(Clone, Debug)]
pub struct FfmpegHardwareAccelerationDetector<P = OperatorHardwareSmokeProbe> {
    ffmpeg_path: PathBuf,
    smoke_probe: P,
}

impl FfmpegHardwareAccelerationDetector<OperatorHardwareSmokeProbe> {
    #[must_use]
    pub fn new(ffmpeg_path: impl Into<PathBuf>) -> Self {
        Self {
            ffmpeg_path: ffmpeg_path.into(),
            smoke_probe: OperatorHardwareSmokeProbe,
        }
    }
}

impl<P> FfmpegHardwareAccelerationDetector<P>
where
    P: HardwareSmokeProbeDetector,
{
    #[must_use]
    pub fn with_smoke_probe(ffmpeg_path: impl Into<PathBuf>, smoke_probe: P) -> Self {
        Self {
            ffmpeg_path: ffmpeg_path.into(),
            smoke_probe,
        }
    }

    pub fn detect_result(&self) -> Result<HardwareAccelerationReport> {
        let output = std::process::Command::new(&self.ffmpeg_path)
            .arg("-hide_banner")
            .arg("-encoders")
            .output()
            .map_err(|err| NakoError::Provider {
                provider: "ffmpeg".to_owned(),
                message: format!("failed to run ffmpeg hardware capability probe: {err}"),
            })?;

        if !output.status.success() {
            return Err(NakoError::Provider {
                provider: "ffmpeg".to_owned(),
                message: format!(
                    "ffmpeg hardware capability probe failed: {}",
                    stderr_message(&output.stderr)
                ),
            });
        }

        let encoders = String::from_utf8_lossy(&output.stdout);
        Ok(report_from_ffmpeg_encoders_with_diagnostics(
            &encoders,
            &OperatorHardwareDeviceInitialization,
            &self.smoke_probe,
        ))
    }
}

impl<P> HardwareAccelerationDetector for FfmpegHardwareAccelerationDetector<P>
where
    P: HardwareSmokeProbeDetector,
{
    fn detect(&self) -> HardwareAccelerationReport {
        self.detect_result()
            .unwrap_or_else(|err| hardware_report_with_probe_error(err.to_string()))
    }
}

#[derive(Clone, Debug, Default)]
pub struct StaticHardwareAccelerationDetector {
    report: HardwareAccelerationReport,
}

impl StaticHardwareAccelerationDetector {
    #[must_use]
    pub fn new(report: HardwareAccelerationReport) -> Self {
        Self { report }
    }
}

pub fn report_from_ffmpeg_encoders(encoders: &str) -> HardwareAccelerationReport {
    report_from_ffmpeg_encoders_with_smoke_probe(encoders, &OperatorHardwareSmokeProbe)
}

pub fn report_from_ffmpeg_encoders_with_smoke_probe(
    encoders: &str,
    smoke_probe: &dyn HardwareSmokeProbeDetector,
) -> HardwareAccelerationReport {
    report_from_ffmpeg_encoders_with_diagnostics(
        encoders,
        &OperatorHardwareDeviceInitialization,
        smoke_probe,
    )
}

pub fn report_from_ffmpeg_encoders_with_diagnostics(
    encoders: &str,
    device_initialization: &dyn HardwareDeviceInitializationDetector,
    smoke_probe: &dyn HardwareSmokeProbeDetector,
) -> HardwareAccelerationReport {
    let has_vaapi = encoders.contains("h264_vaapi");
    let has_nvenc = encoders.contains("h264_nvenc");
    let has_qsv = encoders.contains("h264_qsv");
    let has_amf = encoders.contains("h264_amf");
    let has_videotoolbox = encoders.contains("h264_videotoolbox");

    HardwareAccelerationReport {
        capabilities: vec![
            cpu_capability(),
            encoder_capability(
                HardwareAcceleration::Vaapi,
                "h264_vaapi",
                has_vaapi,
                device_initialization,
                smoke_probe,
            ),
            encoder_capability(
                HardwareAcceleration::Nvenc,
                "h264_nvenc",
                has_nvenc,
                device_initialization,
                smoke_probe,
            ),
            encoder_capability(
                HardwareAcceleration::QuickSync,
                "h264_qsv",
                has_qsv,
                device_initialization,
                smoke_probe,
            ),
            encoder_capability(
                HardwareAcceleration::Amf,
                "h264_amf",
                has_amf,
                device_initialization,
                smoke_probe,
            ),
            encoder_capability(
                HardwareAcceleration::VideoToolbox,
                "h264_videotoolbox",
                has_videotoolbox,
                device_initialization,
                smoke_probe,
            ),
        ],
    }
}

fn encoder_capability(
    accelerator: HardwareAcceleration,
    encoder: &'static str,
    encoder_listed: bool,
    device_initialization: &dyn HardwareDeviceInitializationDetector,
    smoke_probe: &dyn HardwareSmokeProbeDetector,
) -> HardwareAccelerationCapability {
    let initialization = if encoder_listed {
        device_initialization.initialize(accelerator)
    } else {
        HardwareDeviceInitialization::not_run(accelerator)
    };
    let smoke = if encoder_listed {
        smoke_probe.probe(accelerator)
    } else {
        HardwareSmokeProbe::not_run(accelerator)
    };
    let available = encoder_listed
        && initialization.status != HardwareDeviceInitializationStatus::Failed
        && smoke.status != HardwareSmokeProbeStatus::Failed;
    let encoder_discovery = if encoder_listed {
        HardwareEncoderDiscovery::listed(encoder)
    } else {
        HardwareEncoderDiscovery::missing(encoder)
    };

    HardwareAccelerationCapability {
        accelerator,
        available,
        device: None,
        reason: Some(if encoder_listed {
            if available {
                format!("ffmpeg encoder {encoder} is available")
            } else if smoke.status == HardwareSmokeProbeStatus::Failed {
                format!("ffmpeg encoder {encoder} is listed but hardware smoke probe failed")
            } else {
                format!(
                    "ffmpeg encoder {encoder} is listed but hardware device initialization failed"
                )
            }
        } else {
            format!("ffmpeg encoder {encoder} is not listed")
        }),
        stage_capabilities: stage_capabilities_for_encoder(accelerator, encoder, encoder_listed),
        encoder_discovery,
        device_initialization: initialization,
        smoke_probe: smoke,
    }
}

fn hardware_report_with_probe_error(message: String) -> HardwareAccelerationReport {
    HardwareAccelerationReport {
        capabilities: vec![
            cpu_capability(),
            probe_error_capability(HardwareAcceleration::Vaapi, &message),
            probe_error_capability(HardwareAcceleration::Nvenc, &message),
            probe_error_capability(HardwareAcceleration::QuickSync, &message),
            probe_error_capability(HardwareAcceleration::Amf, &message),
            probe_error_capability(HardwareAcceleration::VideoToolbox, &message),
        ],
    }
}

fn probe_error_capability(
    accelerator: HardwareAcceleration,
    message: &str,
) -> HardwareAccelerationCapability {
    HardwareAccelerationCapability {
        accelerator,
        available: false,
        device: None,
        reason: Some(message.to_owned()),
        stage_capabilities: vec![HardwareStageCapability::probe_error(
            HardwarePipelineStage::Encode,
            message,
        )],
        encoder_discovery: HardwareEncoderDiscovery::probe_error(message),
        device_initialization: HardwareDeviceInitialization::not_run(accelerator),
        smoke_probe: HardwareSmokeProbe::not_run(accelerator),
    }
}

fn cpu_capability() -> HardwareAccelerationCapability {
    HardwareAccelerationCapability {
        accelerator: HardwareAcceleration::None,
        available: true,
        device: None,
        reason: Some("cpu encode is always available".to_owned()),
        stage_capabilities: vec![
            HardwareStageCapability::static_available(HardwarePipelineStage::Decode, "software"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Filter, "software"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Encode, "libx264"),
        ],
        encoder_discovery: HardwareEncoderDiscovery::not_required(),
        device_initialization: HardwareDeviceInitialization::not_required(),
        smoke_probe: HardwareSmokeProbe::not_required(),
    }
}

fn static_capability(accelerator: HardwareAcceleration) -> HardwareAccelerationCapability {
    if accelerator == HardwareAcceleration::None {
        return cpu_capability();
    }

    HardwareAccelerationCapability {
        accelerator,
        available: true,
        device: None,
        reason: None,
        stage_capabilities: stage_capabilities_for_static_detector(accelerator),
        encoder_discovery: HardwareEncoderDiscovery::static_detector(),
        device_initialization: HardwareDeviceInitialization::not_run(accelerator),
        smoke_probe: HardwareSmokeProbe::not_run(accelerator),
    }
}

fn stage_capabilities_for_encoder(
    accelerator: HardwareAcceleration,
    encoder: &'static str,
    encoder_listed: bool,
) -> Vec<HardwareStageCapability> {
    let encode = if encoder_listed {
        HardwareStageCapability::listed(HardwarePipelineStage::Encode, encoder)
    } else {
        HardwareStageCapability::missing(HardwarePipelineStage::Encode, encoder)
    };

    match accelerator {
        HardwareAcceleration::None => vec![
            HardwareStageCapability::static_available(HardwarePipelineStage::Decode, "software"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Filter, "software"),
            encode,
        ],
        HardwareAcceleration::Nvenc => vec![
            HardwareStageCapability::static_available(HardwarePipelineStage::Decode, "software"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Filter, "software"),
            encode,
        ],
        HardwareAcceleration::Vaapi => vec![
            HardwareStageCapability::static_available(HardwarePipelineStage::Hwaccel, "vaapi"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Decode, "vaapi"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Filter, "vaapi"),
            encode,
        ],
        HardwareAcceleration::QuickSync => vec![
            HardwareStageCapability::static_available(HardwarePipelineStage::Hwaccel, "qsv"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Decode, "qsv"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Filter, "software"),
            encode,
        ],
        HardwareAcceleration::Amf => vec![
            HardwareStageCapability::static_available(HardwarePipelineStage::Decode, "software"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Filter, "software"),
            encode,
        ],
        HardwareAcceleration::VideoToolbox => vec![
            HardwareStageCapability::static_available(
                HardwarePipelineStage::Decode,
                "videotoolbox",
            ),
            HardwareStageCapability::static_available(HardwarePipelineStage::Filter, "software"),
            encode,
        ],
    }
}

fn stage_capabilities_for_static_detector(
    accelerator: HardwareAcceleration,
) -> Vec<HardwareStageCapability> {
    match accelerator {
        HardwareAcceleration::None => cpu_capability().stage_capabilities,
        HardwareAcceleration::Nvenc => vec![
            HardwareStageCapability::static_available(HardwarePipelineStage::Decode, "software"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Filter, "software"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Encode, "h264_nvenc"),
        ],
        HardwareAcceleration::Vaapi => vec![
            HardwareStageCapability::static_available(HardwarePipelineStage::Hwaccel, "vaapi"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Decode, "vaapi"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Filter, "vaapi"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Encode, "h264_vaapi"),
        ],
        HardwareAcceleration::QuickSync => vec![
            HardwareStageCapability::static_available(HardwarePipelineStage::Hwaccel, "qsv"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Decode, "qsv"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Filter, "software"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Encode, "h264_qsv"),
        ],
        HardwareAcceleration::Amf => vec![
            HardwareStageCapability::static_available(HardwarePipelineStage::Decode, "software"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Filter, "software"),
            HardwareStageCapability::static_available(HardwarePipelineStage::Encode, "h264_amf"),
        ],
        HardwareAcceleration::VideoToolbox => vec![
            HardwareStageCapability::static_available(
                HardwarePipelineStage::Decode,
                "videotoolbox",
            ),
            HardwareStageCapability::static_available(HardwarePipelineStage::Filter, "software"),
            HardwareStageCapability::static_available(
                HardwarePipelineStage::Encode,
                "h264_videotoolbox",
            ),
        ],
    }
}

fn operator_device_initialization_check(accelerator: HardwareAcceleration) -> &'static str {
    match accelerator {
        HardwareAcceleration::None => "cpu encode does not require hardware device initialization",
        HardwareAcceleration::Vaapi => {
            "Verify the host exposes a VAAPI render device to Nako and FFmpeg can initialize VAAPI before enabling VAAPI acceleration"
        }
        HardwareAcceleration::Nvenc => {
            "Verify the NVIDIA driver, container runtime, and FFmpeg can initialize NVENC before enabling NVENC acceleration"
        }
        HardwareAcceleration::QuickSync => {
            "Verify the host exposes Intel Quick Sync devices to Nako and FFmpeg can initialize QSV before enabling Quick Sync acceleration"
        }
        HardwareAcceleration::Amf => {
            "Verify the AMD driver stack and FFmpeg can initialize AMF before enabling AMF acceleration"
        }
        HardwareAcceleration::VideoToolbox => {
            "Verify the host platform supports VideoToolbox and FFmpeg can initialize VideoToolbox before enabling VideoToolbox acceleration"
        }
    }
}

fn operator_smoke_check(accelerator: HardwareAcceleration) -> &'static str {
    match accelerator {
        HardwareAcceleration::None => "cpu encode does not require a hardware smoke probe",
        HardwareAcceleration::Vaapi => {
            "Run a VAAPI H.264 encode smoke test on the host and verify h264_vaapi can encode one frame"
        }
        HardwareAcceleration::Nvenc => {
            "Run an NVENC H.264 encode smoke test on the host and verify h264_nvenc can encode one frame"
        }
        HardwareAcceleration::QuickSync => {
            "Run a Quick Sync H.264 encode smoke test on the host and verify h264_qsv can encode one frame"
        }
        HardwareAcceleration::Amf => {
            "Run an AMF H.264 encode smoke test on the host and verify h264_amf can encode one frame"
        }
        HardwareAcceleration::VideoToolbox => {
            "Run a VideoToolbox H.264 encode smoke test on the host and verify h264_videotoolbox can encode one frame"
        }
    }
}

impl HardwareAccelerationDetector for StaticHardwareAccelerationDetector {
    fn detect(&self) -> HardwareAccelerationReport {
        self.report.clone()
    }
}
