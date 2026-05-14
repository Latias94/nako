use serde::{Deserialize, Serialize};

use crate::{LibraryId, MediaItemId, MediaSourceId};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Library {
    pub id: LibraryId,
    pub name: String,
    pub roots: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Movie,
    Series,
    Season,
    Episode,
    Collection,
    Extra,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaItem {
    pub id: MediaItemId,
    pub kind: MediaKind,
    pub parent_id: Option<MediaItemId>,
    pub metadata: CanonicalMetadata,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CanonicalMetadata {
    pub title: String,
    pub original_title: Option<String>,
    pub sort_title: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaSource {
    pub id: MediaSourceId,
    pub item_id: MediaItemId,
    pub locator: String,
    pub file_name: String,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalProvider {
    Tmdb,
    Douban,
    Bangumi,
    Imdb,
    Local,
    Other(String),
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ExternalId {
    pub provider: ExternalProvider,
    pub value: String,
}
