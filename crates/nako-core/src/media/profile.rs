use serde::{Deserialize, Serialize};

use super::{item::MediaKind, library::LibraryPreset, provider::ExternalProvider};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProfile {
    pub item_kinds: Vec<MediaKind>,
    pub local_readers: Vec<LocalMetadataReader>,
    pub metadata_providers: Vec<ExternalProvider>,
    pub image_providers: Vec<ExternalProvider>,
    pub language: Option<String>,
    pub country: Option<String>,
    pub refresh_mode: MetadataRefreshMode,
    pub local_metadata_policy: LocalMetadataPolicy,
}

impl MetadataProfile {
    #[must_use]
    pub fn from_preset(preset: LibraryPreset) -> Self {
        match preset {
            LibraryPreset::Movies => Self {
                item_kinds: vec![MediaKind::Movie, MediaKind::Collection, MediaKind::Extra],
                local_readers: vec![LocalMetadataReader::Nfo],
                metadata_providers: vec![ExternalProvider::Tmdb, ExternalProvider::Douban],
                image_providers: vec![ExternalProvider::Tmdb],
                language: Some("en-US".to_owned()),
                country: None,
                refresh_mode: MetadataRefreshMode::Default,
                local_metadata_policy: LocalMetadataPolicy::LocalFirst,
            },
            LibraryPreset::Tv => Self {
                item_kinds: vec![MediaKind::Series, MediaKind::Season, MediaKind::Episode],
                local_readers: vec![LocalMetadataReader::Nfo],
                metadata_providers: vec![ExternalProvider::Tmdb],
                image_providers: vec![ExternalProvider::Tmdb],
                language: Some("en-US".to_owned()),
                country: None,
                refresh_mode: MetadataRefreshMode::Default,
                local_metadata_policy: LocalMetadataPolicy::LocalFirst,
            },
            LibraryPreset::Anime => Self {
                item_kinds: vec![
                    MediaKind::Movie,
                    MediaKind::Series,
                    MediaKind::Season,
                    MediaKind::Episode,
                    MediaKind::Extra,
                ],
                local_readers: vec![LocalMetadataReader::Nfo],
                metadata_providers: vec![
                    ExternalProvider::Bangumi,
                    ExternalProvider::Tmdb,
                    ExternalProvider::Douban,
                ],
                image_providers: vec![ExternalProvider::Bangumi, ExternalProvider::Tmdb],
                language: Some("zh-CN".to_owned()),
                country: Some("CN".to_owned()),
                refresh_mode: MetadataRefreshMode::Default,
                local_metadata_policy: LocalMetadataPolicy::LocalFirst,
            },
            LibraryPreset::Music => Self {
                item_kinds: vec![MediaKind::Unknown],
                local_readers: vec![LocalMetadataReader::Embedded],
                metadata_providers: Vec::new(),
                image_providers: Vec::new(),
                language: None,
                country: None,
                refresh_mode: MetadataRefreshMode::Default,
                local_metadata_policy: LocalMetadataPolicy::LocalFirst,
            },
            LibraryPreset::Podcast => Self {
                item_kinds: vec![MediaKind::Unknown],
                local_readers: vec![LocalMetadataReader::Sidecar],
                metadata_providers: Vec::new(),
                image_providers: Vec::new(),
                language: None,
                country: None,
                refresh_mode: MetadataRefreshMode::Default,
                local_metadata_policy: LocalMetadataPolicy::ReadOnly,
            },
            LibraryPreset::Photos | LibraryPreset::HomeVideo => Self {
                item_kinds: vec![MediaKind::Unknown],
                local_readers: vec![LocalMetadataReader::Sidecar],
                metadata_providers: Vec::new(),
                image_providers: Vec::new(),
                language: None,
                country: None,
                refresh_mode: MetadataRefreshMode::MissingOnly,
                local_metadata_policy: LocalMetadataPolicy::LocalFirst,
            },
            LibraryPreset::MixedVideo | LibraryPreset::Custom => Self {
                item_kinds: vec![
                    MediaKind::Movie,
                    MediaKind::Series,
                    MediaKind::Season,
                    MediaKind::Episode,
                    MediaKind::Extra,
                    MediaKind::Unknown,
                ],
                local_readers: vec![LocalMetadataReader::Nfo],
                metadata_providers: vec![ExternalProvider::Tmdb],
                image_providers: vec![ExternalProvider::Tmdb],
                language: Some("en-US".to_owned()),
                country: None,
                refresh_mode: MetadataRefreshMode::Default,
                local_metadata_policy: LocalMetadataPolicy::LocalFirst,
            },
            LibraryPreset::OnlineCatalog => Self {
                item_kinds: vec![MediaKind::Unknown],
                local_readers: Vec::new(),
                metadata_providers: Vec::new(),
                image_providers: Vec::new(),
                language: None,
                country: None,
                refresh_mode: MetadataRefreshMode::ValidationOnly,
                local_metadata_policy: LocalMetadataPolicy::Disabled,
            },
        }
    }
}

impl Default for MetadataProfile {
    fn default() -> Self {
        Self::from_preset(LibraryPreset::MixedVideo)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalMetadataReader {
    Nfo,
    Embedded,
    Sidecar,
    Other(String),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataRefreshMode {
    None,
    ValidationOnly,
    Default,
    MissingOnly,
    FullRefresh,
}

impl Default for MetadataRefreshMode {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalMetadataPolicy {
    Disabled,
    ReadOnly,
    LocalFirst,
    RemoteFirst,
    WriteSidecar,
}

impl Default for LocalMetadataPolicy {
    fn default() -> Self {
        Self::LocalFirst
    }
}
