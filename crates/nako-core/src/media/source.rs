use serde::{Deserialize, Serialize};

use crate::{
    LibraryId, LocalInferenceEvidenceId, MediaItemId, MediaSourceId, SourceDuplicateRelationshipId,
};

use super::item::MediaKind;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaSource {
    pub id: MediaSourceId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub locator: String,
    pub file_name: String,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceDuplicateEvidenceKind {
    StrongFingerprint,
    SizeAndEtag,
    PathEvidence,
    FilesystemLink,
    Manual,
    Other(String),
}

impl SourceDuplicateEvidenceKind {
    #[must_use]
    pub fn as_parts(&self) -> (&'static str, &str) {
        match self {
            Self::StrongFingerprint => ("strong_fingerprint", ""),
            Self::SizeAndEtag => ("size_and_etag", ""),
            Self::PathEvidence => ("path_evidence", ""),
            Self::FilesystemLink => ("filesystem_link", ""),
            Self::Manual => ("manual", ""),
            Self::Other(value) => ("other", value.as_str()),
        }
    }

    #[must_use]
    pub fn from_parts(kind: &str, kind_key: String) -> Self {
        match kind {
            "strong_fingerprint" => Self::StrongFingerprint,
            "size_and_etag" => Self::SizeAndEtag,
            "path_evidence" => Self::PathEvidence,
            "filesystem_link" => Self::FilesystemLink,
            "manual" => Self::Manual,
            "other" => Self::Other(kind_key),
            _ => Self::Other(kind.to_owned()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceDuplicateRelationshipStatus {
    Suggested,
    Confirmed,
    Rejected,
}

impl SourceDuplicateRelationshipStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::Confirmed => "confirmed",
            Self::Rejected => "rejected",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "suggested" => Ok(Self::Suggested),
            "confirmed" => Ok(Self::Confirmed),
            "rejected" => Ok(Self::Rejected),
            _ => Err(crate::NakoError::Database {
                message: format!(
                    "unknown source duplicate relationship status stored in database: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceDuplicateRelationship {
    pub id: SourceDuplicateRelationshipId,
    pub source_id: MediaSourceId,
    pub duplicate_source_id: MediaSourceId,
    pub evidence_kind: SourceDuplicateEvidenceKind,
    pub evidence_value: Option<String>,
    pub status: SourceDuplicateRelationshipStatus,
    pub confidence_milli: Option<u16>,
}

impl SourceDuplicateRelationship {
    #[must_use]
    pub fn canonicalized(&self) -> Self {
        let mut relationship = self.clone();
        if relationship.source_id > relationship.duplicate_source_id {
            std::mem::swap(
                &mut relationship.source_id,
                &mut relationship.duplicate_source_id,
            );
        }
        relationship
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalInferenceEvidenceSource {
    Path,
    FileName,
    Directory,
    NearbyFile,
    MediaProbe,
    Other(String),
}

impl LocalInferenceEvidenceSource {
    #[must_use]
    pub fn as_parts(&self) -> (&'static str, &str) {
        match self {
            Self::Path => ("path", ""),
            Self::FileName => ("file_name", ""),
            Self::Directory => ("directory", ""),
            Self::NearbyFile => ("nearby_file", ""),
            Self::MediaProbe => ("media_probe", ""),
            Self::Other(value) => ("other", value.as_str()),
        }
    }

    #[must_use]
    pub fn from_parts(source: &str, source_key: String) -> Self {
        match source {
            "path" => Self::Path,
            "file_name" => Self::FileName,
            "directory" => Self::Directory,
            "nearby_file" => Self::NearbyFile,
            "media_probe" => Self::MediaProbe,
            "other" => Self::Other(source_key),
            _ => Self::Other(source.to_owned()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LocalInferenceEvidence {
    pub id: LocalInferenceEvidenceId,
    pub source_id: MediaSourceId,
    pub inferred_kind: MediaKind,
    pub inferred_title: Option<String>,
    pub inferred_year: Option<i32>,
    pub inferred_season: Option<u32>,
    pub inferred_episode: Option<u32>,
    pub confidence_milli: Option<u16>,
    pub evidence_source: LocalInferenceEvidenceSource,
    pub evidence_value: String,
    pub inference_version: String,
}
