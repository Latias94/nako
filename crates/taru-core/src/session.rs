use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{MediaSourceId, TaruError, TranscodeSessionId};

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
            TaruError::InvalidInput { .. } | TaruError::Unsupported(_) => Self::InvalidRequest,
            TaruError::Provider { message, .. }
                if message.to_ascii_lowercase().contains("timed out") =>
            {
                Self::Timeout
            }
            TaruError::Provider { .. } => Self::Runner,
            TaruError::Storage { .. } => Self::Storage,
            TaruError::NotFound { .. }
            | TaruError::Conflict { .. }
            | TaruError::Database { .. } => Self::Unknown,
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
