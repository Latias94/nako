use serde::{Deserialize, Serialize};

use crate::{Result, StagingManifestId, TaruError};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StagingPurpose {
    ProbeInput,
    FfmpegInput,
}

impl StagingPurpose {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProbeInput => "probe_input",
            Self::FfmpegInput => "ffmpeg_input",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "probe_input" => Ok(Self::ProbeInput),
            "ffmpeg_input" => Ok(Self::FfmpegInput),
            _ => Err(TaruError::Database {
                message: format!("unknown staging purpose stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StagingState {
    Staging,
    Ready,
    Leased,
    Expired,
    Deleted,
    Failed,
}

impl StagingState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Staging => "staging",
            Self::Ready => "ready",
            Self::Leased => "leased",
            Self::Expired => "expired",
            Self::Deleted => "deleted",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "staging" => Ok(Self::Staging),
            "ready" => Ok(Self::Ready),
            "leased" => Ok(Self::Leased),
            "expired" => Ok(Self::Expired),
            "deleted" => Ok(Self::Deleted),
            "failed" => Ok(Self::Failed),
            _ => Err(TaruError::Database {
                message: format!("unknown staging state stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewStagingManifestRecord {
    pub id: StagingManifestId,
    pub source_uri: String,
    pub source_scheme: String,
    pub purpose: StagingPurpose,
    pub local_path: String,
    pub size_bytes: Option<u64>,
    pub etag: Option<String>,
    pub fingerprint: Option<String>,
    pub state: StagingState,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub last_accessed_at_ms: i64,
    pub expires_at_ms: Option<i64>,
    pub active_leases: u32,
    pub validation_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StagingManifestRecord {
    pub id: StagingManifestId,
    pub source_uri: String,
    pub source_scheme: String,
    pub purpose: StagingPurpose,
    pub local_path: String,
    pub size_bytes: Option<u64>,
    pub etag: Option<String>,
    pub fingerprint: Option<String>,
    pub state: StagingState,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub last_accessed_at_ms: i64,
    pub expires_at_ms: Option<i64>,
    pub active_leases: u32,
    pub validation_error: Option<String>,
}

impl StagingManifestRecord {
    #[must_use]
    pub const fn is_cleanup_candidate_at(&self, now_ms: i64) -> bool {
        matches!(
            self.state,
            StagingState::Staging | StagingState::Ready | StagingState::Failed
        ) && self.active_leases == 0
            && matches!(self.expires_at_ms, Some(expires_at_ms) if expires_at_ms <= now_ms)
    }
}
