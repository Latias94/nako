use serde::{Deserialize, Serialize};

use crate::{
    AddonId, AddonSideEffectId, ArtworkCandidateId, ArtworkTaskId, ImageAssetId, ImageKind, Job,
    JobId, JobStatus, LibraryId, ManagedArtworkArtifactId, ManagedArtworkIngestId, MediaItemId,
    SelectedArtworkId,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewArtworkCandidate {
    pub id: ArtworkCandidateId,
    pub addon_id: AddonId,
    pub side_effect_id: AddonSideEffectId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub source_kind: ArtworkCandidateSourceKind,
    pub source_uri: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArtworkCandidateRecord {
    pub id: ArtworkCandidateId,
    pub addon_id: AddonId,
    pub side_effect_id: AddonSideEffectId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub source_kind: ArtworkCandidateSourceKind,
    pub source_uri: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub language: Option<String>,
    pub status: ArtworkCandidateStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewManagedArtworkIngest {
    pub id: ManagedArtworkIngestId,
    pub candidate_id: ArtworkCandidateId,
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub status: ManagedArtworkIngestStatus,
    pub artifact_id: Option<ManagedArtworkArtifactId>,
    pub failure_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedArtworkIngestRecord {
    pub id: ManagedArtworkIngestId,
    pub candidate_id: ArtworkCandidateId,
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub status: ManagedArtworkIngestStatus,
    pub artifact_id: Option<ManagedArtworkArtifactId>,
    pub failure_code: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedArtworkAcceptanceRecord {
    pub candidate: ArtworkCandidateRecord,
    pub ingest: ManagedArtworkIngestRecord,
    pub job: Job,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedArtworkIngestClaimRecord {
    pub candidate: ArtworkCandidateRecord,
    pub ingest: ManagedArtworkIngestRecord,
    pub job: Job,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewManagedArtworkArtifact {
    pub id: ManagedArtworkArtifactId,
    pub ingest_id: ManagedArtworkIngestId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub storage_uri: String,
    pub content_hash: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub byte_len: Option<u64>,
    pub media_type: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedArtworkArtifactRecord {
    pub id: ManagedArtworkArtifactId,
    pub ingest_id: ManagedArtworkIngestId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub storage_uri: String,
    pub content_hash: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub byte_len: Option<u64>,
    pub media_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedArtworkIngestProcessingRecord {
    pub ingest: ManagedArtworkIngestRecord,
    pub artifact: Option<ManagedArtworkArtifactRecord>,
    pub job: Job,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SelectedArtworkRecord {
    pub id: SelectedArtworkId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub artifact_id: ManagedArtworkArtifactId,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SelectedArtworkPublicationRecord {
    pub selected_artwork: SelectedArtworkRecord,
    pub artifact: ManagedArtworkArtifactRecord,
    pub changed: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtworkCandidateSourceKind {
    RemoteUrl,
}

impl ArtworkCandidateSourceKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RemoteUrl => "remote_url",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "remote_url" => Ok(Self::RemoteUrl),
            _ => Err(crate::TaruError::Database {
                message: format!(
                    "unknown artwork candidate source kind stored in database: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtworkCandidateStatus {
    Proposed,
    Accepted,
    Rejected,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedArtworkIngestStatus {
    Queued,
    Fetching,
    Validating,
    Stored,
    Failed,
}

impl ManagedArtworkIngestStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Fetching => "fetching",
            Self::Validating => "validating",
            Self::Stored => "stored",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "queued" => Ok(Self::Queued),
            "fetching" => Ok(Self::Fetching),
            "validating" => Ok(Self::Validating),
            "stored" => Ok(Self::Stored),
            "failed" => Ok(Self::Failed),
            _ => Err(crate::TaruError::Database {
                message: format!(
                    "unknown managed artwork ingest status stored in database: {value}"
                ),
            }),
        }
    }
}

impl ArtworkCandidateStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "proposed" => Ok(Self::Proposed),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            _ => Err(crate::TaruError::Database {
                message: format!("unknown artwork candidate status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArtworkTask {
    pub id: ArtworkTaskId,
    pub image_id: ImageAssetId,
    pub kind: ArtworkTaskKind,
    pub status: JobStatus,
    pub resource_class: String,
    pub attempts: u32,
    pub max_attempts: u32,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtworkTaskKind {
    Fetch,
    Resize,
    Preview,
    Cleanup,
}

impl ArtworkTaskKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fetch => "fetch",
            Self::Resize => "resize",
            Self::Preview => "preview",
            Self::Cleanup => "cleanup",
        }
    }

    #[must_use]
    pub const fn resource_class(self) -> &'static str {
        match self {
            Self::Fetch => "artwork.fetch",
            Self::Resize => "artwork.resize",
            Self::Preview => "artwork.preview",
            Self::Cleanup => "artwork.cleanup",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "fetch" => Ok(Self::Fetch),
            "resize" => Ok(Self::Resize),
            "preview" => Ok(Self::Preview),
            "cleanup" => Ok(Self::Cleanup),
            _ => Err(crate::TaruError::Database {
                message: format!("unknown artwork task kind stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArtworkTaskQueueOptions {
    pub max_concurrent_fetches: usize,
    pub max_concurrent_resizes: usize,
    pub max_concurrent_previews: usize,
    pub max_concurrent_cleanups: usize,
    pub max_attempts: u32,
}

impl Default for ArtworkTaskQueueOptions {
    fn default() -> Self {
        Self {
            max_concurrent_fetches: 4,
            max_concurrent_resizes: 2,
            max_concurrent_previews: 1,
            max_concurrent_cleanups: 1,
            max_attempts: 3,
        }
    }
}

impl ArtworkTaskQueueOptions {
    #[must_use]
    pub const fn limit_for(&self, kind: ArtworkTaskKind) -> usize {
        match kind {
            ArtworkTaskKind::Fetch => self.max_concurrent_fetches,
            ArtworkTaskKind::Resize => self.max_concurrent_resizes,
            ArtworkTaskKind::Preview => self.max_concurrent_previews,
            ArtworkTaskKind::Cleanup => self.max_concurrent_cleanups,
        }
    }
}
