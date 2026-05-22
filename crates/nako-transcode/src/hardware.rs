use std::path::PathBuf;

use nako_core::{NakoError, Result};
use serde::{Deserialize, Serialize};

use super::ffmpeg::stderr_message;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareAcceleration {
    #[default]
    None,
    Vaapi,
    Nvenc,
    QuickSync,
}

impl HardwareAcceleration {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Vaapi => "vaapi",
            Self::Nvenc => "nvenc",
            Self::QuickSync => "quick_sync",
        }
    }

    #[must_use]
    pub const fn is_gpu(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareAccelerationFallback {
    #[default]
    Cpu,
    Fail,
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
    pub encoder_discovery: HardwareEncoderDiscovery,
    pub device_initialization: HardwareDeviceInitialization,
    pub smoke_probe: HardwareSmokeProbe,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareAccelerationSelection {
    pub acceleration: HardwareAcceleration,
    pub fallback_used: bool,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareAccelerationReadinessStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareAccelerationReadinessReason {
    CpuRequested,
    RequestedAcceleratorReady,
    RequestedAcceleratorUnavailableFallbackToCpu,
    RequestedAcceleratorUnavailableFailPolicy,
    ProbeError,
    DeviceInitializationFailed,
    SmokeProbeFailed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareAccelerationReadiness {
    pub status: HardwareAccelerationReadinessStatus,
    pub reason: HardwareAccelerationReadinessReason,
    pub requested: HardwareAcceleration,
    pub selected: HardwareAcceleration,
    pub fallback_used: bool,
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
        encoder_discovery: HardwareEncoderDiscovery::static_detector(),
        device_initialization: HardwareDeviceInitialization::not_run(accelerator),
        smoke_probe: HardwareSmokeProbe::not_run(accelerator),
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
    }
}

impl HardwareAccelerationDetector for StaticHardwareAccelerationDetector {
    fn detect(&self) -> HardwareAccelerationReport {
        self.report.clone()
    }
}

pub fn select_hardware_acceleration(
    policy: HardwareAccelerationPolicy,
    report: &HardwareAccelerationReport,
) -> Result<HardwareAccelerationSelection> {
    if policy.requested == HardwareAcceleration::None {
        return Ok(HardwareAccelerationSelection {
            acceleration: HardwareAcceleration::None,
            fallback_used: false,
            reason: "cpu encode requested".to_owned(),
        });
    }

    if report.is_available(policy.requested) {
        return Ok(HardwareAccelerationSelection {
            acceleration: policy.requested,
            fallback_used: false,
            reason: format!("{} is available", policy.requested.as_str()),
        });
    }

    match policy.fallback {
        HardwareAccelerationFallback::Cpu => Ok(HardwareAccelerationSelection {
            acceleration: HardwareAcceleration::None,
            fallback_used: true,
            reason: format!(
                "{} is unavailable; falling back to cpu",
                policy.requested.as_str()
            ),
        }),
        HardwareAccelerationFallback::Fail => Err(NakoError::Unsupported(
            "requested hardware accelerator is unavailable",
        )),
    }
}

#[must_use]
pub fn hardware_acceleration_readiness(
    policy: HardwareAccelerationPolicy,
    selection: &HardwareAccelerationSelection,
    report: &HardwareAccelerationReport,
) -> HardwareAccelerationReadiness {
    let reason = if policy.requested == HardwareAcceleration::None {
        HardwareAccelerationReadinessReason::CpuRequested
    } else if !selection.fallback_used && selection.acceleration == policy.requested {
        HardwareAccelerationReadinessReason::RequestedAcceleratorReady
    } else {
        requested_accelerator_unavailable_reason(policy, report)
    };
    let status = match reason {
        HardwareAccelerationReadinessReason::CpuRequested
        | HardwareAccelerationReadinessReason::RequestedAcceleratorReady => {
            HardwareAccelerationReadinessStatus::Ready
        }
        HardwareAccelerationReadinessReason::RequestedAcceleratorUnavailableFallbackToCpu
        | HardwareAccelerationReadinessReason::ProbeError
        | HardwareAccelerationReadinessReason::DeviceInitializationFailed
        | HardwareAccelerationReadinessReason::SmokeProbeFailed => {
            HardwareAccelerationReadinessStatus::Degraded
        }
        HardwareAccelerationReadinessReason::RequestedAcceleratorUnavailableFailPolicy => {
            HardwareAccelerationReadinessStatus::Unavailable
        }
    };

    HardwareAccelerationReadiness {
        status,
        reason,
        requested: policy.requested,
        selected: selection.acceleration,
        fallback_used: selection.fallback_used,
    }
}

#[must_use]
pub fn hardware_acceleration_readiness_without_selection(
    policy: HardwareAccelerationPolicy,
    report: &HardwareAccelerationReport,
) -> HardwareAccelerationReadiness {
    if policy.requested == HardwareAcceleration::None {
        return HardwareAccelerationReadiness {
            status: HardwareAccelerationReadinessStatus::Ready,
            reason: HardwareAccelerationReadinessReason::CpuRequested,
            requested: policy.requested,
            selected: HardwareAcceleration::None,
            fallback_used: false,
        };
    }

    if report.is_available(policy.requested) {
        return HardwareAccelerationReadiness {
            status: HardwareAccelerationReadinessStatus::Ready,
            reason: HardwareAccelerationReadinessReason::RequestedAcceleratorReady,
            requested: policy.requested,
            selected: policy.requested,
            fallback_used: false,
        };
    }

    let reason = requested_accelerator_unavailable_reason(policy, report);
    HardwareAccelerationReadiness {
        status: match reason {
            HardwareAccelerationReadinessReason::RequestedAcceleratorUnavailableFailPolicy => {
                HardwareAccelerationReadinessStatus::Unavailable
            }
            _ => HardwareAccelerationReadinessStatus::Degraded,
        },
        reason,
        requested: policy.requested,
        selected: if policy.fallback == HardwareAccelerationFallback::Cpu {
            HardwareAcceleration::None
        } else {
            policy.requested
        },
        fallback_used: policy.fallback == HardwareAccelerationFallback::Cpu,
    }
}

fn requested_accelerator_unavailable_reason(
    policy: HardwareAccelerationPolicy,
    report: &HardwareAccelerationReport,
) -> HardwareAccelerationReadinessReason {
    if policy.fallback == HardwareAccelerationFallback::Fail {
        return HardwareAccelerationReadinessReason::RequestedAcceleratorUnavailableFailPolicy;
    }

    let Some(capability) = report.capability_for(policy.requested) else {
        return HardwareAccelerationReadinessReason::RequestedAcceleratorUnavailableFallbackToCpu;
    };

    match capability.encoder_discovery.status {
        HardwareEncoderDiscoveryStatus::ProbeError => {
            return HardwareAccelerationReadinessReason::ProbeError;
        }
        HardwareEncoderDiscoveryStatus::Missing
        | HardwareEncoderDiscoveryStatus::NotRequired
        | HardwareEncoderDiscoveryStatus::Listed
        | HardwareEncoderDiscoveryStatus::Static => {}
    }

    if capability.device_initialization.status == HardwareDeviceInitializationStatus::Failed {
        return HardwareAccelerationReadinessReason::DeviceInitializationFailed;
    }

    if capability.smoke_probe.status == HardwareSmokeProbeStatus::Failed {
        return HardwareAccelerationReadinessReason::SmokeProbeFailed;
    }

    HardwareAccelerationReadinessReason::RequestedAcceleratorUnavailableFallbackToCpu
}
