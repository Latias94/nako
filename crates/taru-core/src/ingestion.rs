use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use crate::{JobId, LibraryId, MediaSourceId, ScanSnapshotId};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestionFailurePhase {
    Scan,
    Probe,
}

impl IngestionFailurePhase {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scan => "scan",
            Self::Probe => "probe",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "scan" => Ok(Self::Scan),
            "probe" => Ok(Self::Probe),
            _ => Err(crate::TaruError::Database {
                message: format!("unknown ingestion failure phase stored in database: {value}"),
            }),
        }
    }
}

impl fmt::Display for IngestionFailurePhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for IngestionFailurePhase {
    type Err = crate::TaruError;

    fn from_str(value: &str) -> crate::Result<Self> {
        Self::parse(value)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestionFailureClass {
    Storage,
    Probe,
    Database,
    InvalidInput,
    Unsupported,
    Unknown,
}

impl IngestionFailureClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Storage => "storage",
            Self::Probe => "probe",
            Self::Database => "database",
            Self::InvalidInput => "invalid_input",
            Self::Unsupported => "unsupported",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "storage" => Ok(Self::Storage),
            "probe" => Ok(Self::Probe),
            "database" => Ok(Self::Database),
            "invalid_input" => Ok(Self::InvalidInput),
            "unsupported" => Ok(Self::Unsupported),
            "unknown" => Ok(Self::Unknown),
            _ => Err(crate::TaruError::Database {
                message: format!("unknown ingestion failure class stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestionFailureStatus {
    Open,
    Resolved,
    Ignored,
}

impl IngestionFailureStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Resolved => "resolved",
            Self::Ignored => "ignored",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "open" => Ok(Self::Open),
            "resolved" => Ok(Self::Resolved),
            "ignored" => Ok(Self::Ignored),
            _ => Err(crate::TaruError::Database {
                message: format!("unknown ingestion failure status stored in database: {value}"),
            }),
        }
    }
}

impl fmt::Display for IngestionFailureStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for IngestionFailureStatus {
    type Err = crate::TaruError;

    fn from_str(value: &str) -> crate::Result<Self> {
        Self::parse(value)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewIngestionFailure {
    pub library_id: LibraryId,
    pub job_id: Option<JobId>,
    pub scan_id: Option<ScanSnapshotId>,
    pub source_id: Option<MediaSourceId>,
    pub phase: IngestionFailurePhase,
    pub target_uri: String,
    pub target_kind: String,
    pub failure_class: IngestionFailureClass,
    pub message: String,
    pub retryable: bool,
    pub failed_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IngestionFailureRecord {
    pub library_id: LibraryId,
    pub job_id: Option<JobId>,
    pub scan_id: Option<ScanSnapshotId>,
    pub source_id: Option<MediaSourceId>,
    pub phase: IngestionFailurePhase,
    pub target_uri: String,
    pub target_kind: String,
    pub failure_class: IngestionFailureClass,
    pub status: IngestionFailureStatus,
    pub message: String,
    pub retryable: bool,
    pub attempts: u32,
    pub first_failed_at_ms: i64,
    pub last_failed_at_ms: i64,
    pub resolved_at_ms: Option<i64>,
    pub ignored_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct IngestionFailureFilter {
    pub library_id: Option<LibraryId>,
    pub phase: Option<IngestionFailurePhase>,
    pub status: Option<IngestionFailureStatus>,
}
