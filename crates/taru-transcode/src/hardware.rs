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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareAccelerationCapability {
    pub accelerator: HardwareAcceleration,
    pub available: bool,
    pub device: Option<String>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareAccelerationReport {
    pub capabilities: Vec<HardwareAccelerationCapability>,
}

impl HardwareAccelerationReport {
    #[must_use]
    pub fn cpu_only() -> Self {
        Self {
            capabilities: vec![HardwareAccelerationCapability {
                accelerator: HardwareAcceleration::None,
                available: true,
                device: None,
                reason: Some("cpu encode is always available".to_owned()),
            }],
        }
    }

    #[must_use]
    pub fn with_available(accelerators: impl IntoIterator<Item = HardwareAcceleration>) -> Self {
        Self {
            capabilities: accelerators
                .into_iter()
                .map(|accelerator| HardwareAccelerationCapability {
                    accelerator,
                    available: true,
                    device: None,
                    reason: None,
                })
                .collect(),
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

#[derive(Clone, Debug)]
pub struct FfmpegHardwareAccelerationDetector {
    ffmpeg_path: PathBuf,
}

impl FfmpegHardwareAccelerationDetector {
    #[must_use]
    pub fn new(ffmpeg_path: impl Into<PathBuf>) -> Self {
        Self {
            ffmpeg_path: ffmpeg_path.into(),
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
        Ok(report_from_ffmpeg_encoders(&encoders))
    }
}

impl HardwareAccelerationDetector for FfmpegHardwareAccelerationDetector {
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
    let has_vaapi = encoders.contains("h264_vaapi");
    let has_nvenc = encoders.contains("h264_nvenc");
    let has_qsv = encoders.contains("h264_qsv");

    HardwareAccelerationReport {
        capabilities: vec![
            HardwareAccelerationCapability {
                accelerator: HardwareAcceleration::None,
                available: true,
                device: None,
                reason: Some("cpu encode is always available".to_owned()),
            },
            encoder_capability(HardwareAcceleration::Vaapi, "h264_vaapi", has_vaapi),
            encoder_capability(HardwareAcceleration::Nvenc, "h264_nvenc", has_nvenc),
            encoder_capability(HardwareAcceleration::QuickSync, "h264_qsv", has_qsv),
        ],
    }
}

fn encoder_capability(
    accelerator: HardwareAcceleration,
    encoder: &'static str,
    available: bool,
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
    }
}

fn hardware_report_with_probe_error(message: String) -> HardwareAccelerationReport {
    HardwareAccelerationReport {
        capabilities: vec![
            HardwareAccelerationCapability {
                accelerator: HardwareAcceleration::None,
                available: true,
                device: None,
                reason: Some("cpu encode is always available".to_owned()),
            },
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
