use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use taru_core::{Result, TaruError};

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
pub enum HardwareCapabilityEvidence {
    CpuAlwaysAvailable,
    FfmpegEncoderListed,
    FfmpegEncoderMissing,
    FfmpegProbeError,
    StaticDetector,
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
    pub evidence: HardwareCapabilityEvidence,
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
            .map_err(|err| TaruError::Provider {
                provider: "ffmpeg".to_owned(),
                message: format!("failed to run ffmpeg hardware capability probe: {err}"),
            })?;

        if !output.status.success() {
            return Err(TaruError::Provider {
                provider: "ffmpeg".to_owned(),
                message: format!(
                    "ffmpeg hardware capability probe failed: {}",
                    stderr_message(&output.stderr)
                ),
            });
        }

        let encoders = String::from_utf8_lossy(&output.stdout);
        Ok(report_from_ffmpeg_encoders_with_smoke_probe(
            &encoders,
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
                smoke_probe,
            ),
            encoder_capability(
                HardwareAcceleration::Nvenc,
                "h264_nvenc",
                has_nvenc,
                smoke_probe,
            ),
            encoder_capability(
                HardwareAcceleration::QuickSync,
                "h264_qsv",
                has_qsv,
                smoke_probe,
            ),
        ],
    }
}

fn encoder_capability(
    accelerator: HardwareAcceleration,
    encoder: &'static str,
    available: bool,
    smoke_probe: &dyn HardwareSmokeProbeDetector,
) -> HardwareAccelerationCapability {
    HardwareAccelerationCapability {
        accelerator,
        available,
        device: None,
        reason: Some(if available {
            format!("ffmpeg encoder {encoder} is available")
        } else {
            format!("ffmpeg encoder {encoder} is not listed")
        }),
        evidence: if available {
            HardwareCapabilityEvidence::FfmpegEncoderListed
        } else {
            HardwareCapabilityEvidence::FfmpegEncoderMissing
        },
        smoke_probe: smoke_probe.probe(accelerator),
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
        evidence: HardwareCapabilityEvidence::FfmpegProbeError,
        smoke_probe: HardwareSmokeProbe::not_run(accelerator),
    }
}

fn cpu_capability() -> HardwareAccelerationCapability {
    HardwareAccelerationCapability {
        accelerator: HardwareAcceleration::None,
        available: true,
        device: None,
        reason: Some("cpu encode is always available".to_owned()),
        evidence: HardwareCapabilityEvidence::CpuAlwaysAvailable,
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
        evidence: HardwareCapabilityEvidence::StaticDetector,
        smoke_probe: HardwareSmokeProbe::not_run(accelerator),
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
        HardwareAccelerationFallback::Fail => Err(TaruError::Unsupported(
            "requested hardware accelerator is unavailable",
        )),
    }
}
