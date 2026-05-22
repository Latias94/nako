use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{MediaSourceId, StorageErrorKind, TaruError, TranscodeSessionId};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeSessionKind {
    Remux,
    HlsTranscode,
}

impl TranscodeSessionKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Remux => "remux",
            Self::HlsTranscode => "hls_transcode",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "remux" => Some(Self::Remux),
            "hls_transcode" => Some(Self::HlsTranscode),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeSessionState {
    Planned,
    Starting,
    Running,
    CancelRequested,
    Cancelled,
    Failed,
    Finished,
}

impl TranscodeSessionState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::CancelRequested => "cancel_requested",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
            Self::Finished => "finished",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "planned" => Some(Self::Planned),
            "starting" => Some(Self::Starting),
            "running" => Some(Self::Running),
            "cancel_requested" => Some(Self::CancelRequested),
            "cancelled" => Some(Self::Cancelled),
            "failed" => Some(Self::Failed),
            "finished" => Some(Self::Finished),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(
            self,
            Self::Planned | Self::Starting | Self::Running | Self::CancelRequested
        )
    }

    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Cancelled | Self::Failed | Self::Finished)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeFailureCategory {
    InvalidRequest,
    Probe,
    Plan,
    Staging,
    Budget,
    HardwareFallback,
    Runner,
    Timeout,
    Storage,
    Stale,
    Cancelled,
    Unknown,
}

impl TranscodeFailureCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::Probe => "probe",
            Self::Plan => "plan",
            Self::Staging => "staging",
            Self::Budget => "budget",
            Self::HardwareFallback => "hardware_fallback",
            Self::Runner => "runner",
            Self::Timeout => "timeout",
            Self::Storage => "storage",
            Self::Stale => "stale",
            Self::Cancelled => "cancelled",
            Self::Unknown => "unknown",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "invalid_request" => Some(Self::InvalidRequest),
            "probe" => Some(Self::Probe),
            "plan" => Some(Self::Plan),
            "staging" => Some(Self::Staging),
            "budget" => Some(Self::Budget),
            "hardware_fallback" => Some(Self::HardwareFallback),
            "runner" => Some(Self::Runner),
            "timeout" => Some(Self::Timeout),
            "storage" => Some(Self::Storage),
            "stale" => Some(Self::Stale),
            "cancelled" => Some(Self::Cancelled),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    #[must_use]
    pub fn from_error(error: &TaruError) -> Self {
        match error {
            TaruError::InvalidInput { message } if is_plan_failure_message(message) => Self::Plan,
            TaruError::InvalidInput { .. } => Self::InvalidRequest,
            TaruError::Unsupported(message) if is_hardware_fallback_message(message) => {
                Self::HardwareFallback
            }
            TaruError::Unsupported(_) => Self::InvalidRequest,
            TaruError::Unauthorized { .. } | TaruError::Forbidden { .. } => Self::InvalidRequest,
            TaruError::Provider { provider, .. } if provider == "ffprobe" => Self::Probe,
            TaruError::Provider { message, .. } if is_timeout_message(message) => Self::Timeout,
            TaruError::Provider { message, .. } if is_hardware_fallback_message(message) => {
                Self::HardwareFallback
            }
            TaruError::Provider { message, .. } if is_plan_failure_message(message) => Self::Plan,
            TaruError::Provider { .. } => Self::Runner,
            TaruError::Storage {
                kind:
                    StorageErrorKind::StagingBudgetExhausted | StorageErrorKind::ResourceBudgetClosed,
                ..
            } => Self::Budget,
            TaruError::Storage {
                kind:
                    StorageErrorKind::Io
                    | StorageErrorKind::StagingValidationMismatch
                    | StorageErrorKind::SecurityViolation
                    | StorageErrorKind::Backup,
                message,
                ..
            } if is_staging_message(message) => Self::Staging,
            TaruError::Storage { .. } => Self::Storage,
            TaruError::NotFound { .. }
            | TaruError::Conflict { .. }
            | TaruError::Database { .. } => Self::Unknown,
        }
    }
}

fn is_timeout_message(message: &str) -> bool {
    message.to_ascii_lowercase().contains("timed out")
}

fn is_plan_failure_message(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("command plan")
        || message.contains("invalid remux request")
        || message.contains("invalid hls request")
        || message.contains("playback transcode plan")
        || message.contains("transcode profile")
}

fn is_staging_message(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("staging")
        || message.contains("staged")
        || message.contains("temporary")
        || message.contains("promote hls")
        || message.contains("promote remux")
        || message.contains("output directory")
        || message.contains("output path")
}

fn is_hardware_fallback_message(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("hardware accelerator") || message.contains("hardware acceleration")
}

#[cfg(test)]
mod tests {
    use crate::{StorageErrorKind, TaruError, TranscodeFailureCategory};

    #[test]
    fn transcode_failure_category_maps_support_boundaries() {
        let cases = [
            (
                TaruError::Provider {
                    provider: "ffprobe".to_owned(),
                    message: "failed to probe C:\\secret\\movie.mkv".to_owned(),
                },
                TranscodeFailureCategory::Probe,
            ),
            (
                TaruError::InvalidInput {
                    message: "invalid hls request: hls command plan does not contain expected playlist path: C:\\secret\\playlist.m3u8".to_owned(),
                },
                TranscodeFailureCategory::Plan,
            ),
            (
                TaruError::storage_io(
                    "C:\\secret\\taru-cache\\hls\\playlist.m3u8",
                    "failed to promote hls output directory",
                ),
                TranscodeFailureCategory::Staging,
            ),
            (
                TaruError::storage(
                    "webdav:///Movies/secret.mkv",
                    StorageErrorKind::StagingBudgetExhausted,
                    "used=10, additional=4, max=12",
                ),
                TranscodeFailureCategory::Budget,
            ),
            (
                TaruError::Unsupported("requested hardware accelerator is unavailable"),
                TranscodeFailureCategory::HardwareFallback,
            ),
            (
                TaruError::Provider {
                    provider: "ffmpeg_hls".to_owned(),
                    message: "hls runner timed out after 100 ms".to_owned(),
                },
                TranscodeFailureCategory::Timeout,
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(TranscodeFailureCategory::from_error(&error), expected);
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewTranscodeSession {
    pub id: TranscodeSessionId,
    pub source_id: MediaSourceId,
    pub kind: TranscodeSessionKind,
    pub request_key: String,
    pub output_path: PathBuf,
    pub state: TranscodeSessionState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeSessionRecord {
    pub id: TranscodeSessionId,
    pub source_id: MediaSourceId,
    pub kind: TranscodeSessionKind,
    pub request_key: String,
    pub output_path: PathBuf,
    pub state: TranscodeSessionState,
    pub failure_category: Option<TranscodeFailureCategory>,
    pub failure_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}
