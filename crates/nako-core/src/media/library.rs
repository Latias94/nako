use serde::{Deserialize, Serialize};

use crate::{LibraryId, MediaItemId};

use super::profile::MetadataProfile;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Library {
    pub id: LibraryId,
    pub name: String,
    pub roots: Vec<String>,
    pub options: LibraryOptions,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryItemState {
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub provisional: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryOptions {
    pub domain: MediaDomain,
    pub preset: LibraryPreset,
    pub scan: LibraryScanOptions,
    pub naming_strategy: NamingStrategy,
    pub metadata_profile: MetadataProfile,
}

impl LibraryOptions {
    #[must_use]
    pub fn from_preset(preset: LibraryPreset) -> Self {
        let domain = preset.default_domain();
        Self {
            domain,
            preset,
            scan: LibraryScanOptions::default(),
            naming_strategy: preset.default_naming_strategy(),
            metadata_profile: MetadataProfile::from_preset(preset),
        }
    }
}

impl Default for LibraryOptions {
    fn default() -> Self {
        Self::from_preset(LibraryPreset::MixedVideo)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaDomain {
    Video,
    Audio,
    Image,
    Document,
    Mixed,
    Online,
}

impl MediaDomain {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Image => "image",
            Self::Document => "document",
            Self::Mixed => "mixed",
            Self::Online => "online",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "video" => Some(Self::Video),
            "audio" => Some(Self::Audio),
            "image" => Some(Self::Image),
            "document" => Some(Self::Document),
            "mixed" => Some(Self::Mixed),
            "online" => Some(Self::Online),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryPreset {
    Movies,
    Tv,
    Anime,
    Music,
    Podcast,
    Photos,
    HomeVideo,
    MixedVideo,
    OnlineCatalog,
    Custom,
}

impl LibraryPreset {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Movies => "movies",
            Self::Tv => "tv",
            Self::Anime => "anime",
            Self::Music => "music",
            Self::Podcast => "podcast",
            Self::Photos => "photos",
            Self::HomeVideo => "home_video",
            Self::MixedVideo => "mixed_video",
            Self::OnlineCatalog => "online_catalog",
            Self::Custom => "custom",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "movies" => Some(Self::Movies),
            "tv" => Some(Self::Tv),
            "anime" => Some(Self::Anime),
            "music" => Some(Self::Music),
            "podcast" => Some(Self::Podcast),
            "photos" => Some(Self::Photos),
            "home_video" => Some(Self::HomeVideo),
            "mixed_video" => Some(Self::MixedVideo),
            "online_catalog" => Some(Self::OnlineCatalog),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    #[must_use]
    pub const fn default_domain(self) -> MediaDomain {
        match self {
            Self::Movies | Self::Tv | Self::Anime | Self::HomeVideo => MediaDomain::Video,
            Self::Music | Self::Podcast => MediaDomain::Audio,
            Self::Photos => MediaDomain::Image,
            Self::MixedVideo | Self::Custom => MediaDomain::Mixed,
            Self::OnlineCatalog => MediaDomain::Online,
        }
    }

    #[must_use]
    pub const fn default_naming_strategy(self) -> NamingStrategy {
        match self {
            Self::Movies => NamingStrategy::Movie,
            Self::Tv => NamingStrategy::Series,
            Self::Anime => NamingStrategy::Anime,
            Self::Music => NamingStrategy::Music,
            Self::Podcast => NamingStrategy::Podcast,
            Self::Photos => NamingStrategy::Photo,
            Self::HomeVideo => NamingStrategy::HomeVideo,
            Self::OnlineCatalog => NamingStrategy::OnlineCatalog,
            Self::MixedVideo | Self::Custom => NamingStrategy::Mixed,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryScanOptions {
    pub realtime_monitor: bool,
    pub max_depth: Option<usize>,
}

impl Default for LibraryScanOptions {
    fn default() -> Self {
        Self {
            realtime_monitor: false,
            max_depth: Some(32),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NamingStrategy {
    Movie,
    Series,
    Anime,
    Music,
    Podcast,
    Photo,
    HomeVideo,
    Mixed,
    OnlineCatalog,
}
