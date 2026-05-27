use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    MediaItemId, MediaSourceId, NakoError, PlaybackSessionId, StorageErrorKind, TranscodeSessionId,
    UserPrincipalId,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackSessionMode {
    Direct,
    Remux,
    Hls,
}

impl PlaybackSessionMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Remux => "remux",
            Self::Hls => "hls",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "direct" => Some(Self::Direct),
            "remux" => Some(Self::Remux),
            "hls" => Some(Self::Hls),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackSessionState {
    Active,
    Paused,
    CancelRequested,
    Cancelled,
    Ended,
    Failed,
}

impl PlaybackSessionState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::CancelRequested => "cancel_requested",
            Self::Cancelled => "cancelled",
            Self::Ended => "ended",
            Self::Failed => "failed",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "paused" => Some(Self::Paused),
            "cancel_requested" => Some(Self::CancelRequested),
            "cancelled" => Some(Self::Cancelled),
            "ended" => Some(Self::Ended),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active | Self::Paused | Self::CancelRequested)
    }

    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Cancelled | Self::Ended | Self::Failed)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewPlaybackSession {
    pub id: PlaybackSessionId,
    pub principal_id: UserPrincipalId,
    pub source_id: MediaSourceId,
    pub item_id: MediaItemId,
    pub mode: PlaybackSessionMode,
    pub state: PlaybackSessionState,
    pub client_capabilities_json: Option<String>,
    pub started_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackSessionRecord {
    pub id: PlaybackSessionId,
    pub principal_id: UserPrincipalId,
    pub source_id: MediaSourceId,
    pub item_id: MediaItemId,
    pub mode: PlaybackSessionMode,
    pub state: PlaybackSessionState,
    pub client_capabilities_json: Option<String>,
    pub transcode_session_id: Option<TranscodeSessionId>,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub last_heartbeat_at_ms: Option<i64>,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackSessionHeartbeat {
    pub id: PlaybackSessionId,
    pub state: PlaybackSessionState,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub heartbeat_at_ms: i64,
}

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
    pub fn from_error(error: &NakoError) -> Self {
        match error {
            NakoError::InvalidInput { message } if is_plan_failure_message(message) => Self::Plan,
            NakoError::InvalidInput { .. } => Self::InvalidRequest,
            NakoError::Unsupported(message) if is_hardware_fallback_message(message) => {
                Self::HardwareFallback
            }
            NakoError::Unsupported(_) => Self::InvalidRequest,
            NakoError::Unauthorized { .. } | NakoError::Forbidden { .. } => Self::InvalidRequest,
            NakoError::Provider { provider, .. } if provider == "ffprobe" => Self::Probe,
            NakoError::Provider { message, .. } if is_timeout_message(message) => Self::Timeout,
            NakoError::Provider { message, .. } if is_hardware_fallback_message(message) => {
                Self::HardwareFallback
            }
            NakoError::Provider { message, .. } if is_plan_failure_message(message) => Self::Plan,
            NakoError::Provider { .. } => Self::Runner,
            NakoError::Storage {
                kind:
                    StorageErrorKind::StagingBudgetExhausted | StorageErrorKind::ResourceBudgetClosed,
                ..
            } => Self::Budget,
            NakoError::Storage {
                kind:
                    StorageErrorKind::Io
                    | StorageErrorKind::StagingValidationMismatch
                    | StorageErrorKind::SecurityViolation
                    | StorageErrorKind::Backup,
                message,
                ..
            } if is_staging_message(message) => Self::Staging,
            NakoError::Storage { .. } => Self::Storage,
            NakoError::NotFound { .. }
            | NakoError::Conflict { .. }
            | NakoError::Database { .. } => Self::Unknown,
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
    use crate::{NakoError, StorageErrorKind, TranscodeFailureCategory};

    #[test]
    fn transcode_failure_category_maps_support_boundaries() {
        let cases = [
            (
                NakoError::Provider {
                    provider: "ffprobe".to_owned(),
                    message: "failed to probe C:\\secret\\movie.mkv".to_owned(),
                },
                TranscodeFailureCategory::Probe,
            ),
            (
                NakoError::InvalidInput {
                    message: "invalid hls request: hls command plan does not contain expected playlist path: C:\\secret\\playlist.m3u8".to_owned(),
                },
                TranscodeFailureCategory::Plan,
            ),
            (
                NakoError::storage_io(
                    "C:\\secret\\nako-cache\\hls\\playlist.m3u8",
                    "failed to promote hls output directory",
                ),
                TranscodeFailureCategory::Staging,
            ),
            (
                NakoError::storage(
                    "webdav:///Movies/secret.mkv",
                    StorageErrorKind::StagingBudgetExhausted,
                    "used=10, additional=4, max=12",
                ),
                TranscodeFailureCategory::Budget,
            ),
            (
                NakoError::Unsupported("requested hardware accelerator is unavailable"),
                TranscodeFailureCategory::HardwareFallback,
            ),
            (
                NakoError::Provider {
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
    pub runtime_metrics: TranscodeSessionRuntimeMetrics,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeSessionRuntimeProgress {
    Continue,
    End,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeSessionRuntimeMetrics {
    pub frame_count: Option<u64>,
    pub fps_millis: Option<u64>,
    pub bitrate_kbps: Option<u64>,
    pub total_size_bytes: Option<u64>,
    pub output_time_ms: Option<u64>,
    pub dup_frames: Option<u64>,
    pub drop_frames: Option<u64>,
    pub speed_millis: Option<u64>,
    pub progress: Option<TranscodeSessionRuntimeProgress>,
}

impl TranscodeSessionRuntimeMetrics {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.frame_count.is_none()
            && self.fps_millis.is_none()
            && self.bitrate_kbps.is_none()
            && self.total_size_bytes.is_none()
            && self.output_time_ms.is_none()
            && self.dup_frames.is_none()
            && self.drop_frames.is_none()
            && self.speed_millis.is_none()
            && self.progress.is_none()
    }
}
