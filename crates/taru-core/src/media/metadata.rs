use serde::{Deserialize, Serialize};

use crate::{
    AddonId, LibraryId, MediaItem, MediaItemId, MetadataProviderAttemptId, ProviderMappingId,
    ProviderSubjectId,
};

use super::provider::{ExternalProvider, ProviderSubject};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataField {
    Title,
    OriginalTitle,
    SortTitle,
    Overview,
    ReleaseDate,
    RuntimeMinutes,
    Tagline,
    Genres,
    Tags,
    Ratings,
    Images,
    Credits,
    Collections,
    Studios,
    ExternalIds,
}

impl MetadataField {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::OriginalTitle => "original_title",
            Self::SortTitle => "sort_title",
            Self::Overview => "overview",
            Self::ReleaseDate => "release_date",
            Self::RuntimeMinutes => "runtime_minutes",
            Self::Tagline => "tagline",
            Self::Genres => "genres",
            Self::Tags => "tags",
            Self::Ratings => "ratings",
            Self::Images => "images",
            Self::Credits => "credits",
            Self::Collections => "collections",
            Self::Studios => "studios",
            Self::ExternalIds => "external_ids",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataFieldLock {
    pub item_id: MediaItemId,
    pub field: MetadataField,
    pub locked: bool,
    pub source: MetadataSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataSource {
    Local,
    Nfo,
    Provider(ExternalProvider),
    User,
    Addon(AddonId),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderRawResponse {
    pub item_id: MediaItemId,
    pub provider: ExternalProvider,
    pub provider_key: String,
    pub fetched_at: String,
    pub body_json: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRefreshProviderMappingCommit {
    pub id: Option<ProviderMappingId>,
    pub subject: ProviderSubject,
    pub confidence_milli: Option<u16>,
    pub source: MetadataSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRefreshPersistenceCommit {
    pub item: MediaItem,
    pub raw_response: ProviderRawResponse,
    pub provider_mapping: MetadataRefreshProviderMappingCommit,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRefreshPersistenceSummary {
    pub item_id: MediaItemId,
    pub provider_subject_id: ProviderSubjectId,
    pub provider_mapping_id: ProviderMappingId,
    pub confirmed_libraries: Vec<LibraryId>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataAttemptFilter {
    pub provider: Option<ExternalProvider>,
    pub status: Option<MetadataProviderAttemptStatus>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderRawResponseFilter {
    pub provider: Option<ExternalProvider>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderRawResponseCleanup {
    pub provider: Option<ExternalProvider>,
    pub fetched_before: String,
    pub deleted: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewMetadataProviderAttempt {
    pub id: MetadataProviderAttemptId,
    pub job_id: crate::JobId,
    pub item_id: MediaItemId,
    pub provider: ExternalProvider,
    pub status: MetadataProviderAttemptStatus,
    pub provider_key: Option<String>,
    pub matched_by: Option<MetadataMatchKind>,
    pub started_at: String,
    pub finished_at: String,
    pub error_class: Option<MetadataProviderErrorClass>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderAttemptRecord {
    pub id: MetadataProviderAttemptId,
    pub job_id: crate::JobId,
    pub item_id: MediaItemId,
    pub provider: ExternalProvider,
    pub status: MetadataProviderAttemptStatus,
    pub provider_key: Option<String>,
    pub matched_by: Option<MetadataMatchKind>,
    pub started_at: String,
    pub finished_at: String,
    pub error_class: Option<MetadataProviderErrorClass>,
    pub message: Option<String>,
}

impl MetadataProviderAttemptRecord {
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        self.status.is_retryable()
            || self
                .error_class
                .is_some_and(MetadataProviderErrorClass::is_retryable)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataProviderAttemptStatus {
    Succeeded,
    SkippedDisabled,
    SkippedUnavailable,
    NotImplemented,
    NoMatch,
    RateLimited,
    Failed,
}

impl MetadataProviderAttemptStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::SkippedDisabled => "skipped_disabled",
            Self::SkippedUnavailable => "skipped_unavailable",
            Self::NotImplemented => "not_implemented",
            Self::NoMatch => "no_match",
            Self::RateLimited => "rate_limited",
            Self::Failed => "failed",
        }
    }

    #[must_use]
    pub const fn is_retryable(self) -> bool {
        matches!(self, Self::SkippedUnavailable | Self::RateLimited)
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "succeeded" => Ok(Self::Succeeded),
            "skipped_disabled" => Ok(Self::SkippedDisabled),
            "skipped_unavailable" => Ok(Self::SkippedUnavailable),
            "not_implemented" => Ok(Self::NotImplemented),
            "no_match" => Ok(Self::NoMatch),
            "rate_limited" => Ok(Self::RateLimited),
            "failed" => Ok(Self::Failed),
            _ => Err(crate::TaruError::InvalidInput {
                message: format!("unknown metadata provider attempt status: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataMatchKind {
    ExternalId,
    Search,
}

impl MetadataMatchKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExternalId => "external_id",
            Self::Search => "search",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "external_id" => Ok(Self::ExternalId),
            "search" => Ok(Self::Search),
            _ => Err(crate::TaruError::InvalidInput {
                message: format!("unknown metadata match kind: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataProviderErrorClass {
    Timeout,
    RateLimited,
    Network,
    HttpStatus,
    Parse,
    Auth,
    Unsupported,
    NoMatch,
    Unavailable,
    Unknown,
}

impl MetadataProviderErrorClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::RateLimited => "rate_limited",
            Self::Network => "network",
            Self::HttpStatus => "http_status",
            Self::Parse => "parse",
            Self::Auth => "auth",
            Self::Unsupported => "unsupported",
            Self::NoMatch => "no_match",
            Self::Unavailable => "unavailable",
            Self::Unknown => "unknown",
        }
    }

    #[must_use]
    pub const fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::Timeout
                | Self::RateLimited
                | Self::Network
                | Self::HttpStatus
                | Self::Unavailable
                | Self::Unknown
        )
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "timeout" => Ok(Self::Timeout),
            "rate_limited" => Ok(Self::RateLimited),
            "network" => Ok(Self::Network),
            "http_status" => Ok(Self::HttpStatus),
            "parse" => Ok(Self::Parse),
            "auth" => Ok(Self::Auth),
            "unsupported" => Ok(Self::Unsupported),
            "no_match" => Ok(Self::NoMatch),
            "unavailable" => Ok(Self::Unavailable),
            "unknown" => Ok(Self::Unknown),
            _ => Err(crate::TaruError::InvalidInput {
                message: format!("unknown metadata provider error class: {value}"),
            }),
        }
    }
}
