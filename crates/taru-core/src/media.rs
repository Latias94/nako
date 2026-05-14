use serde::{Deserialize, Serialize};

use crate::{
    ArtworkTaskId, CollectionId, GenreId, ImageAssetId, JobStatus, LibraryId, MediaItemId,
    MediaSourceId, PersonId, ScanSnapshotId, StudioId, TagId,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Library {
    pub id: LibraryId,
    pub name: String,
    pub roots: Vec<String>,
    pub options: LibraryOptions,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
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
    pub runtime_minutes: Option<u32>,
    pub tagline: Option<String>,
    pub genres: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub ratings: Vec<ContentRating>,
    pub images: Vec<ImageRef>,
    pub credits: Vec<Credit>,
    #[serde(default)]
    pub collections: Vec<CollectionRef>,
    #[serde(default)]
    pub studios: Vec<StudioRef>,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContentRating {
    pub source: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImageRef {
    pub kind: ImageKind,
    pub uri: String,
    pub provider: ExternalProvider,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageKind {
    Poster,
    Backdrop,
    Logo,
    Thumbnail,
    Banner,
    Other(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Credit {
    pub name: String,
    pub role: CreditRole,
    pub character: Option<String>,
    pub order: Option<u32>,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollectionRef {
    pub name: String,
    pub overview: Option<String>,
    pub sort_order: Option<u32>,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StudioRef {
    pub name: String,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditRole {
    Actor,
    Director,
    Writer,
    Producer,
    Creator,
    Other(String),
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

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageOwner {
    Item(MediaItemId),
    Person(PersonId),
    Collection(CollectionId),
    Studio(StudioId),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ScanSnapshot {
    pub id: ScanSnapshotId,
    pub library_id: LibraryId,
    pub root: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: ScanStatus,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    Running,
    Succeeded,
    Failed,
}

impl ScanStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            _ => Err(crate::TaruError::Database {
                message: format!("unknown scan status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DirectorySnapshot {
    pub scan_id: ScanSnapshotId,
    pub uri: String,
    pub etag: Option<String>,
    pub modified_at: Option<String>,
    pub child_count: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceState {
    pub library_id: LibraryId,
    pub source_id: Option<MediaSourceId>,
    pub uri: String,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<String>,
    pub etag: Option<String>,
    pub fingerprint: Option<String>,
    pub last_seen_scan_id: ScanSnapshotId,
    pub tombstoned: bool,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaProbeResult {
    pub duration_ms: Option<u64>,
    pub container: Option<String>,
    pub bit_rate: Option<u64>,
    pub streams: Vec<MediaStreamInfo>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaStreamInfo {
    pub index: u32,
    pub kind: MediaStreamKind,
    pub codec: Option<String>,
    pub language: Option<String>,
    pub duration_ms: Option<u64>,
    pub bit_rate: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub channels: Option<u32>,
    pub sample_rate: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaStreamKind {
    Video,
    Audio,
    Subtitle,
    Data,
    Attachment,
    Other(String),
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

impl MediaKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Movie => "movie",
            Self::Series => "series",
            Self::Season => "season",
            Self::Episode => "episode",
            Self::Collection => "collection",
            Self::Extra => "extra",
            Self::Unknown => "unknown",
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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderRawResponse {
    pub item_id: MediaItemId,
    pub provider: ExternalProvider,
    pub provider_key: String,
    pub fetched_at: String,
    pub body_json: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anime_preset_is_editable_configuration_not_media_kind() {
        let mut options = LibraryOptions::from_preset(LibraryPreset::Anime);

        assert_eq!(options.domain, MediaDomain::Video);
        assert_eq!(options.preset, LibraryPreset::Anime);
        assert_eq!(options.naming_strategy, NamingStrategy::Anime);
        assert!(
            options
                .metadata_profile
                .item_kinds
                .contains(&MediaKind::Movie)
        );
        assert!(
            options
                .metadata_profile
                .item_kinds
                .contains(&MediaKind::Episode)
        );
        assert_eq!(
            options.metadata_profile.metadata_providers,
            vec![
                ExternalProvider::Bangumi,
                ExternalProvider::Tmdb,
                ExternalProvider::Douban
            ]
        );

        options.metadata_profile.metadata_providers = vec![ExternalProvider::Tmdb];

        assert_eq!(
            options.metadata_profile.metadata_providers,
            vec![ExternalProvider::Tmdb]
        );
        assert!(!matches!(MediaKind::Movie, MediaKind::Unknown));
    }
}
