use serde::{Deserialize, Serialize};

use crate::{
    AddonId, AutomationProviderId, MediaItemId, MetadataCandidateReviewId, NakoError,
    ProviderSubjectId, Result,
};

use super::{
    CanonicalMetadata, CollectionRef, ContentRating, Credit, ExternalId, ExternalProvider,
    ImageRef, MediaKind, MetadataSource, ProviderSubject, ProviderSubjectKind, StudioRef,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateGraph {
    pub root: MetadataCandidateNode,
    #[serde(default)]
    pub related: Vec<MetadataCandidateNode>,
    #[serde(default)]
    pub relationships: Vec<MetadataCandidateRelationship>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateNode {
    pub source: MetadataCandidateSource,
    pub kind: MediaKind,
    pub subject: Option<MetadataCandidateSubject>,
    pub metadata: MetadataCandidateRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateSubject {
    pub provider: ExternalProvider,
    pub subject_kind: ProviderSubjectKind,
    pub subject_key: String,
    pub title: Option<String>,
    pub release_year: Option<i32>,
    pub locale: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateRelationship {
    pub parent_subject: MetadataCandidateSubject,
    pub child_subject: MetadataCandidateSubject,
    pub kind: MetadataCandidateRelationshipKind,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataCandidateRelationshipKind {
    Contains,
    BelongsTo,
    Related,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataCandidateSource {
    Local,
    Nfo,
    Provider(ExternalProvider),
    Addon(AddonId),
    Automation(AutomationProviderId),
    User,
    Other(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewPlan {
    pub root: MetadataCandidateReviewNode,
    #[serde(default)]
    pub related: Vec<MetadataCandidateReviewNode>,
    #[serde(default)]
    pub relationships: Vec<MetadataCandidateReviewRelationship>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewNode {
    pub source: MetadataCandidateSource,
    pub kind: MediaKind,
    pub subject: Option<MetadataCandidateSubject>,
    pub metadata: MetadataCandidateRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewRelationship {
    pub parent_subject: MetadataCandidateSubject,
    pub child_subject: MetadataCandidateSubject,
    pub kind: MetadataCandidateRelationshipKind,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataCandidateReviewStatus {
    Pending,
    Accepted,
    Rejected,
    Superseded,
    Expired,
}

impl MetadataCandidateReviewStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Expired => "expired",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "superseded" => Ok(Self::Superseded),
            "expired" => Ok(Self::Expired),
            _ => Err(NakoError::Database {
                message: format!(
                    "unknown metadata candidate review status stored in database: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewMetadataCandidateReview {
    pub id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub source: MetadataCandidateSource,
    pub source_key: String,
    pub plan: MetadataCandidateReviewPlan,
    pub expires_at_ms: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewRecord {
    pub id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub source: MetadataCandidateSource,
    pub source_key: String,
    pub status: MetadataCandidateReviewStatus,
    pub plan: MetadataCandidateReviewPlan,
    pub expires_at_ms: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateRecord {
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub sort_title: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub runtime_minutes: Option<u32>,
    pub tagline: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub ratings: Vec<ContentRating>,
    #[serde(default)]
    pub images: Vec<ImageRef>,
    #[serde(default)]
    pub credits: Vec<Credit>,
    #[serde(default)]
    pub collections: Vec<CollectionRef>,
    #[serde(default)]
    pub studios: Vec<StudioRef>,
    #[serde(default)]
    pub external_ids: Vec<ExternalId>,
}

impl MetadataCandidateGraph {
    #[must_use]
    pub fn new(
        source: MetadataCandidateSource,
        kind: MediaKind,
        metadata: MetadataCandidateRecord,
    ) -> Self {
        Self {
            root: MetadataCandidateNode {
                source,
                kind,
                subject: None,
                metadata,
            },
            related: Vec::new(),
            relationships: Vec::new(),
        }
    }

    #[must_use]
    pub fn for_provider(
        provider: ExternalProvider,
        kind: MediaKind,
        subject_kind: ProviderSubjectKind,
        subject_key: impl Into<String>,
        metadata: MetadataCandidateRecord,
    ) -> Self {
        let subject_key = subject_key.into();
        let title = metadata.title.clone();
        let release_year = release_year(metadata.release_date.as_deref());

        Self {
            root: MetadataCandidateNode {
                source: MetadataCandidateSource::Provider(provider.clone()),
                kind,
                subject: Some(MetadataCandidateSubject {
                    provider,
                    subject_kind,
                    subject_key,
                    title,
                    release_year,
                    locale: None,
                }),
                metadata,
            },
            related: Vec::new(),
            relationships: Vec::new(),
        }
    }

    #[must_use]
    pub fn for_nfo(kind: MediaKind, metadata: MetadataCandidateRecord) -> Self {
        Self::new(MetadataCandidateSource::Nfo, kind, metadata)
    }

    #[must_use]
    pub fn from_canonical(
        source: MetadataCandidateSource,
        kind: MediaKind,
        metadata: CanonicalMetadata,
    ) -> Self {
        Self::new(source, kind, MetadataCandidateRecord::from(metadata))
    }

    #[must_use]
    pub fn canonical_metadata(&self) -> CanonicalMetadata {
        self.root.metadata.to_canonical_metadata()
    }

    #[must_use]
    pub fn root_provider_subject(&self) -> Option<&MetadataCandidateSubject> {
        self.root.subject.as_ref()
    }
}

impl MetadataCandidateReviewPlan {
    #[must_use]
    pub fn from_graph(graph: &MetadataCandidateGraph) -> Self {
        Self {
            root: MetadataCandidateReviewNode::from(&graph.root),
            related: graph
                .related
                .iter()
                .map(MetadataCandidateReviewNode::from)
                .collect(),
            relationships: graph
                .relationships
                .iter()
                .map(MetadataCandidateReviewRelationship::from)
                .collect(),
        }
    }
}

impl From<&MetadataCandidateGraph> for MetadataCandidateReviewPlan {
    fn from(value: &MetadataCandidateGraph) -> Self {
        Self::from_graph(value)
    }
}

impl From<&MetadataCandidateNode> for MetadataCandidateReviewNode {
    fn from(value: &MetadataCandidateNode) -> Self {
        Self {
            source: value.source.clone(),
            kind: value.kind,
            subject: value.subject.clone(),
            metadata: value.metadata.clone(),
        }
    }
}

impl From<&MetadataCandidateRelationship> for MetadataCandidateReviewRelationship {
    fn from(value: &MetadataCandidateRelationship) -> Self {
        Self {
            parent_subject: value.parent_subject.clone(),
            child_subject: value.child_subject.clone(),
            kind: value.kind,
        }
    }
}

impl MetadataCandidateRecord {
    #[must_use]
    pub fn to_canonical_metadata(&self) -> CanonicalMetadata {
        CanonicalMetadata {
            title: self.title.clone().unwrap_or_default(),
            original_title: self.original_title.clone(),
            sort_title: self.sort_title.clone(),
            overview: self.overview.clone(),
            release_date: self.release_date.clone(),
            runtime_minutes: self.runtime_minutes,
            tagline: self.tagline.clone(),
            genres: self.genres.clone(),
            tags: self.tags.clone(),
            ratings: self.ratings.clone(),
            images: self.images.clone(),
            credits: self.credits.clone(),
            collections: self.collections.clone(),
            studios: self.studios.clone(),
            external_ids: self.external_ids.clone(),
        }
    }
}

impl MetadataCandidateSubject {
    #[must_use]
    pub fn into_provider_subject(self, id: ProviderSubjectId) -> ProviderSubject {
        ProviderSubject {
            id,
            provider: self.provider,
            subject_kind: self.subject_kind,
            subject_key: self.subject_key,
            title: self.title,
            release_year: self.release_year,
            locale: self.locale,
        }
    }
}

impl From<CanonicalMetadata> for MetadataCandidateRecord {
    fn from(value: CanonicalMetadata) -> Self {
        Self {
            title: non_empty(value.title),
            original_title: value.original_title,
            sort_title: value.sort_title,
            overview: value.overview,
            release_date: value.release_date,
            runtime_minutes: value.runtime_minutes,
            tagline: value.tagline,
            genres: value.genres,
            tags: value.tags,
            ratings: value.ratings,
            images: value.images,
            credits: value.credits,
            collections: value.collections,
            studios: value.studios,
            external_ids: value.external_ids,
        }
    }
}

impl From<MetadataSource> for MetadataCandidateSource {
    fn from(value: MetadataSource) -> Self {
        match value {
            MetadataSource::Local => Self::Local,
            MetadataSource::Nfo => Self::Nfo,
            MetadataSource::Provider(provider) => Self::Provider(provider),
            MetadataSource::User => Self::User,
            MetadataSource::Addon(addon_id) => Self::Addon(addon_id),
        }
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

fn release_year(value: Option<&str>) -> Option<i32> {
    let value = value?;
    let year = value.get(0..4)?;
    if year.chars().all(|character| character.is_ascii_digit()) {
        year.parse().ok()
    } else {
        None
    }
}
