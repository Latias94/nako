use serde::{Deserialize, Serialize};

use crate::{LibraryId, NakoError, Result, StagingManifestId};

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
            _ => Err(NakoError::Database {
                message: format!("unknown staging purpose stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StagingState {
    Reserved,
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
            Self::Reserved => "reserved",
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
            "reserved" => Ok(Self::Reserved),
            "staging" => Ok(Self::Staging),
            "ready" => Ok(Self::Ready),
            "leased" => Ok(Self::Leased),
            "expired" => Ok(Self::Expired),
            "deleted" => Ok(Self::Deleted),
            "failed" => Ok(Self::Failed),
            _ => Err(NakoError::Database {
                message: format!("unknown staging state stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StagingAttributionKind {
    Attributed,
    Ambiguous,
    Unknown,
}

impl StagingAttributionKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Attributed => "attributed",
            Self::Ambiguous => "ambiguous",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "attributed" => Ok(Self::Attributed),
            "ambiguous" => Ok(Self::Ambiguous),
            "unknown" => Ok(Self::Unknown),
            _ => Err(NakoError::Database {
                message: format!("unknown staging attribution kind stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StagingAttribution {
    Attributed { library_id: LibraryId },
    Ambiguous,
    Unknown,
}

impl StagingAttribution {
    #[must_use]
    pub const fn attributed(library_id: LibraryId) -> Self {
        Self::Attributed { library_id }
    }

    #[must_use]
    pub const fn ambiguous() -> Self {
        Self::Ambiguous
    }

    #[must_use]
    pub const fn unknown() -> Self {
        Self::Unknown
    }

    #[must_use]
    pub const fn kind(self) -> StagingAttributionKind {
        match self {
            Self::Attributed { .. } => StagingAttributionKind::Attributed,
            Self::Ambiguous => StagingAttributionKind::Ambiguous,
            Self::Unknown => StagingAttributionKind::Unknown,
        }
    }

    #[must_use]
    pub const fn library_id(self) -> Option<LibraryId> {
        match self {
            Self::Attributed { library_id } => Some(library_id),
            Self::Ambiguous | Self::Unknown => None,
        }
    }

    #[must_use]
    pub const fn as_parts(self) -> (StagingAttributionKind, Option<LibraryId>) {
        (self.kind(), self.library_id())
    }

    pub fn from_parts(kind: StagingAttributionKind, library_id: Option<LibraryId>) -> Result<Self> {
        match (kind, library_id) {
            (StagingAttributionKind::Attributed, Some(library_id)) => {
                Ok(Self::Attributed { library_id })
            }
            (StagingAttributionKind::Attributed, None) => Err(NakoError::Database {
                message: "staging attribution persisted as attributed without a library id"
                    .to_owned(),
            }),
            (StagingAttributionKind::Ambiguous, None) => Ok(Self::Ambiguous),
            (StagingAttributionKind::Unknown, None) => Ok(Self::Unknown),
            (StagingAttributionKind::Ambiguous | StagingAttributionKind::Unknown, Some(_)) => {
                Err(NakoError::Database {
                    message: format!(
                        "staging attribution persisted as {} with a library id",
                        kind.as_str()
                    ),
                })
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewStagingManifestRecord {
    pub id: StagingManifestId,
    pub attribution: StagingAttribution,
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
    pub attribution: StagingAttribution,
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
            StagingState::Reserved
                | StagingState::Staging
                | StagingState::Ready
                | StagingState::Expired
                | StagingState::Failed
        ) && self.active_leases == 0
            && matches!(self.expires_at_ms, Some(expires_at_ms) if expires_at_ms <= now_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staging_attribution_rejects_invalid_persisted_combinations() {
        let library_id = LibraryId::new();

        assert_eq!(
            StagingAttribution::from_parts(StagingAttributionKind::Attributed, Some(library_id))
                .unwrap(),
            StagingAttribution::attributed(library_id)
        );
        assert!(StagingAttribution::from_parts(StagingAttributionKind::Attributed, None).is_err());
        assert!(
            StagingAttribution::from_parts(StagingAttributionKind::Unknown, Some(library_id))
                .is_err()
        );
        assert!(
            StagingAttribution::from_parts(StagingAttributionKind::Ambiguous, Some(library_id))
                .is_err()
        );
    }
}
