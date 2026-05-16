use serde::{Deserialize, Serialize};

use crate::{LibraryId, MediaSourceId, ScanSnapshotId};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ScanSnapshot {
    pub id: ScanSnapshotId,
    pub library_id: LibraryId,
    pub root: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: ScanStatus,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    Running,
    Succeeded,
    Failed,
}

impl ScanStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            _ => Err(crate::TaruError::Database {
                message: format!("unknown scan status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DirectorySnapshot {
    pub scan_id: ScanSnapshotId,
    pub uri: String,
    pub etag: Option<String>,
    pub modified_at: Option<String>,
    pub child_count: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceState {
    pub library_id: LibraryId,
    pub source_id: Option<MediaSourceId>,
    pub uri: String,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<String>,
    pub etag: Option<String>,
    pub fingerprint: Option<String>,
    pub last_seen_scan_id: ScanSnapshotId,
    pub tombstoned: bool,
}
