use serde::{Deserialize, Serialize};

use crate::{
    CollectionId, GenreId, ImageAssetId, MediaItemId, NakoError, PersonId, Result, StudioId, TagId,
};

use super::{
    item::{CreditRole, ImageKind},
    metadata::MetadataSource,
    provider::{ExternalId, ExternalProvider},
};

pub const CATALOG_SEARCH_PROJECTION_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Person {
    pub id: PersonId,
    pub name: String,
    pub sort_name: Option<String>,
    pub overview: Option<String>,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemCredit {
    pub item_id: MediaItemId,
    pub person_id: PersonId,
    pub role: CreditRole,
    pub character: Option<String>,
    pub sort_order: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Genre {
    pub id: GenreId,
    pub name: String,
    pub source: MetadataSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemGenre {
    pub item_id: MediaItemId,
    pub genre_id: GenreId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub source: MetadataSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemTag {
    pub item_id: MediaItemId,
    pub tag_id: TagId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Collection {
    pub id: CollectionId,
    pub name: String,
    pub overview: Option<String>,
    pub source: MetadataSource,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollectionItem {
    pub collection_id: CollectionId,
    pub item_id: MediaItemId,
    pub sort_order: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Studio {
    pub id: StudioId,
    pub name: String,
    pub source: MetadataSource,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemStudio {
    pub item_id: MediaItemId,
    pub studio_id: StudioId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImageAsset {
    pub id: ImageAssetId,
    pub owner: ImageOwner,
    pub kind: ImageKind,
    pub source_uri: String,
    pub provider: ExternalProvider,
    pub cache_uri: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub language: Option<String>,
    pub selected: bool,
    pub content_hash: Option<String>,
    pub etag: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CatalogItemGraphReplacement {
    pub people: Vec<Person>,
    pub credits: Vec<ItemCredit>,
    pub genres: Vec<Genre>,
    pub item_genres: Vec<ItemGenre>,
    pub tags: Vec<Tag>,
    pub item_tags: Vec<ItemTag>,
    pub collections: Vec<Collection>,
    pub collection_items: Vec<CollectionItem>,
    pub studios: Vec<Studio>,
    pub item_studios: Vec<ItemStudio>,
    pub images: Vec<ImageAsset>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CatalogSearchProjection {
    pub item_id: MediaItemId,
    pub projection_version: u16,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub browse_facets: Vec<BrowseFacet>,
    #[serde(default)]
    pub sort_keys: Vec<SortKey>,
    #[serde(default)]
    pub provider_identifiers: Vec<ExternalId>,
}

impl CatalogSearchProjection {
    #[must_use]
    pub fn new(item_id: MediaItemId, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            item_id,
            projection_version: CATALOG_SEARCH_PROJECTION_VERSION,
            title: title.into(),
            body: body.into(),
            aliases: Vec::new(),
            browse_facets: Vec::new(),
            sort_keys: Vec::new(),
            provider_identifiers: Vec::new(),
        }
    }

    pub fn try_from_facet_labels(
        item_id: MediaItemId,
        title: impl Into<String>,
        body: impl Into<String>,
        facets: Vec<String>,
    ) -> Result<Self> {
        let mut projection = Self::new(item_id, title, body);
        projection.browse_facets = facets
            .into_iter()
            .map(|facet| BrowseFacet::parse_label(&facet))
            .collect::<Result<Vec<_>>>()?;
        Ok(projection)
    }

    #[must_use]
    pub fn facet_labels(&self) -> Vec<String> {
        self.browse_facets.iter().map(BrowseFacet::label).collect()
    }

    #[must_use]
    pub fn searchable_text(&self) -> String {
        let mut parts = Vec::new();
        push_search_text_part(&mut parts, &self.title);
        push_search_text_part(&mut parts, &self.body);
        for alias in &self.aliases {
            push_search_text_part(&mut parts, alias);
        }
        for identifier in &self.provider_identifiers {
            push_search_text_part(&mut parts, &identifier.value);
        }
        parts.join(" ")
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CatalogItemProjectionCommit {
    pub graph: CatalogItemGraphReplacement,
    pub search: CatalogSearchProjection,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct BrowseFacet {
    pub kind: BrowseFacetKind,
    pub value: String,
}

impl BrowseFacet {
    #[must_use]
    pub fn new(kind: BrowseFacetKind, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.into(),
        }
    }

    #[must_use]
    pub fn label(&self) -> String {
        let (kind, kind_key) = self.kind.as_parts();
        if kind_key.is_empty() {
            format!("{kind}:{}", self.value)
        } else {
            format!("{kind}:{kind_key}:{}", self.value)
        }
    }

    pub fn parse_label(value: &str) -> Result<Self> {
        let Some((kind, remainder)) = value.split_once(':') else {
            return Err(NakoError::InvalidInput {
                message: format!("browse facet '{value}' must contain kind:value"),
            });
        };
        if kind.trim().is_empty() {
            return Err(NakoError::InvalidInput {
                message: "browse facet must start with a non-empty kind".to_owned(),
            });
        }
        if remainder.trim().is_empty() {
            return Err(NakoError::InvalidInput {
                message: format!("browse facet '{value}' has an empty value"),
            });
        };

        let (kind, facet_value) = if browse_facet_kind_requires_key(kind) {
            let Some((kind_key, facet_value)) = remainder.split_once(':') else {
                return Err(NakoError::InvalidInput {
                    message: format!("browse facet '{value}' must contain kind:key:value"),
                });
            };
            if kind_key.trim().is_empty() || facet_value.trim().is_empty() {
                return Err(NakoError::InvalidInput {
                    message: format!("browse facet '{value}' must contain non-empty key and value"),
                });
            }
            (
                BrowseFacetKind::from_parts(kind, kind_key.to_owned()),
                facet_value,
            )
        } else {
            (BrowseFacetKind::from_parts(kind, String::new()), remainder)
        };

        if facet_value.trim().is_empty() {
            return Err(NakoError::InvalidInput {
                message: format!("browse facet '{value}' has an empty value"),
            });
        }

        Ok(Self::new(kind, facet_value))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowseFacetKind {
    Kind,
    Source,
    Genre,
    Tag,
    Collection,
    Studio,
    Credit,
    Actor,
    Director,
    Writer,
    Producer,
    Creator,
    Provider,
    ReleaseYear,
    ExternalId(String),
    CreditRole(String),
    Other(String),
}

impl BrowseFacetKind {
    #[must_use]
    pub fn as_parts(&self) -> (&'static str, &str) {
        match self {
            Self::Kind => ("kind", ""),
            Self::Source => ("source", ""),
            Self::Genre => ("genre", ""),
            Self::Tag => ("tag", ""),
            Self::Collection => ("collection", ""),
            Self::Studio => ("studio", ""),
            Self::Credit => ("credit", ""),
            Self::Actor => ("actor", ""),
            Self::Director => ("director", ""),
            Self::Writer => ("writer", ""),
            Self::Producer => ("producer", ""),
            Self::Creator => ("creator", ""),
            Self::Provider => ("provider", ""),
            Self::ReleaseYear => ("release_year", ""),
            Self::ExternalId(value) => ("external_id", value.as_str()),
            Self::CreditRole(value) => ("credit_role", value.as_str()),
            Self::Other(value) => ("other", value.as_str()),
        }
    }

    #[must_use]
    pub fn from_parts(kind: &str, kind_key: String) -> Self {
        match kind {
            "kind" => Self::Kind,
            "source" => Self::Source,
            "genre" => Self::Genre,
            "tag" => Self::Tag,
            "collection" => Self::Collection,
            "studio" => Self::Studio,
            "credit" => Self::Credit,
            "actor" => Self::Actor,
            "director" => Self::Director,
            "writer" => Self::Writer,
            "producer" => Self::Producer,
            "creator" => Self::Creator,
            "provider" => Self::Provider,
            "release_year" => Self::ReleaseYear,
            "external_id" => Self::ExternalId(kind_key),
            "credit_role" => Self::CreditRole(kind_key),
            "other" => Self::Other(kind_key),
            _ => Self::Other(kind.to_owned()),
        }
    }
}

fn browse_facet_kind_requires_key(kind: &str) -> bool {
    matches!(kind, "external_id" | "credit_role" | "other")
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SortKey {
    pub kind: SortKeyKind,
    pub value: String,
}

impl SortKey {
    #[must_use]
    pub fn new(kind: SortKeyKind, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortKeyKind {
    Title,
    SortTitle,
    ReleaseDate,
    Other(String),
}

fn push_search_text_part(parts: &mut Vec<String>, value: &str) {
    let value = value.trim();
    if !value.is_empty() {
        parts.push(value.to_owned());
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageOwner {
    Item(MediaItemId),
    Person(PersonId),
    Collection(CollectionId),
    Studio(StudioId),
}
