use serde::{Deserialize, Serialize};

use crate::PageInfo;

macro_rules! public_string_value {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($variant:ident => $wire:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub enum $name {
            $($variant,)+
            Other(String),
        }

        impl $name {
            #[must_use]
            pub fn from_wire_value(value: &str) -> Self {
                match value {
                    $($wire => Self::$variant,)+
                    other => Self::Other(other.to_owned()),
                }
            }

            #[must_use]
            pub fn wire_value(&self) -> &str {
                match self {
                    $(Self::$variant => $wire,)+
                    Self::Other(value) => value,
                }
            }

            #[must_use]
            pub const fn is_known(&self) -> bool {
                !matches!(self, Self::Other(_))
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.wire_value())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Ok(Self::from_wire_value(&value))
            }
        }
    };
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryListResponse {
    pub libraries: Vec<LibraryDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryResponse {
    pub library: LibraryDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibrarySourcesResponse {
    pub library: LibraryDto,
    pub sources: Vec<LibrarySourceResponse>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibrarySourceResponse {
    pub source: MediaSourceDto,
    pub item: Option<MediaItemDto>,
    pub probe: Option<MediaProbeDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryDto {
    pub id: String,
    pub name: String,
    pub roots: Vec<String>,
    pub options: LibraryOptionsDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryOptionsDto {
    pub domain: ClientMediaDomain,
    pub preset: ClientLibraryPreset,
    pub scan: LibraryScanOptionsDto,
    pub naming_strategy: ClientNamingStrategy,
    pub metadata_profile: MetadataProfileDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryScanOptionsDto {
    pub realtime_monitor: bool,
    pub max_depth: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProfileDto {
    pub item_kinds: Vec<ClientMediaKind>,
    pub local_readers: Vec<ClientLocalMetadataReader>,
    pub metadata_providers: Vec<ClientExternalProvider>,
    pub image_providers: Vec<ClientExternalProvider>,
    pub language: Option<String>,
    pub country: Option<String>,
    pub refresh_mode: ClientMetadataRefreshMode,
    pub local_metadata_policy: ClientLocalMetadataPolicy,
    pub scan: MetadataScanPolicyDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataScanPolicyDto {
    pub enabled: bool,
    pub addon_scrape: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemsResponse {
    pub items: Vec<MediaItemDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemDetailResponse {
    pub item: MediaItemDto,
    pub sources: Vec<MediaSourceDto>,
    pub credits: Vec<ItemCreditDto>,
    pub genres: Vec<ItemGenreDto>,
    pub tags: Vec<ItemTagDto>,
    pub collections: Vec<CollectionItemDto>,
    pub studios: Vec<ItemStudioDto>,
    pub images: Vec<PublicImageRefDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemCreditsResponse {
    pub item_id: String,
    pub credits: Vec<ItemCreditDto>,
    pub people: Vec<PersonDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImagesResponse {
    pub item_id: String,
    pub images: Vec<PublicImageRefDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PeopleResponse {
    pub people: Vec<PersonDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PersonResponse {
    pub person: PersonDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PersonItemsResponse {
    pub person: PersonDto,
    pub items: Vec<MediaItemDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TagsResponse {
    pub tags: Vec<TagDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TagItemsResponse {
    pub tag: TagDto,
    pub items: Vec<MediaItemDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenreListResponse {
    pub genres: Vec<GenreDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenreItemsResponse {
    pub genre: GenreDto,
    pub items: Vec<MediaItemDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchItemHit>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SearchItemHit {
    pub item: MediaItemDto,
    pub score: f32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceProbeResponse {
    pub source_id: String,
    pub probe: MediaProbeDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaItemDto {
    pub id: String,
    pub kind: ClientMediaKind,
    pub parent_id: Option<String>,
    pub metadata: CanonicalMetadataDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CanonicalMetadataDto {
    pub title: String,
    pub original_title: Option<String>,
    pub sort_title: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub runtime_minutes: Option<u32>,
    pub tagline: Option<String>,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub ratings: Vec<ContentRatingDto>,
    pub credits: Vec<CreditDto>,
    pub collections: Vec<CollectionRefDto>,
    pub studios: Vec<StudioRefDto>,
    pub external_ids: Vec<ExternalIdDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContentRatingDto {
    pub source: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CreditDto {
    pub name: String,
    pub role: ClientCreditRole,
    pub character: Option<String>,
    pub order: Option<u32>,
    pub external_ids: Vec<ExternalIdDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollectionRefDto {
    pub name: String,
    pub overview: Option<String>,
    pub sort_order: Option<u32>,
    pub external_ids: Vec<ExternalIdDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StudioRefDto {
    pub name: String,
    pub external_ids: Vec<ExternalIdDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ExternalIdDto {
    pub provider: ClientExternalProvider,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaSourceDto {
    pub id: String,
    pub library_id: String,
    pub item_id: String,
    pub file_name: String,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaProbeDto {
    pub duration_ms: Option<u64>,
    pub container: Option<String>,
    pub bit_rate: Option<u64>,
    pub streams: Vec<MediaStreamDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackDecisionResponse {
    pub source: MediaSourceDto,
    pub probe: Option<MediaProbeDto>,
    pub decision: ClientPlaybackDecision,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientPlaybackDecision {
    pub mode: ClientPlaybackMode,
    pub reason: String,
    pub direct_play: Option<ClientDirectPlayPlan>,
    pub transcode_plan: Option<ClientTranscodePlan>,
}

public_string_value! {
    pub enum ClientPlaybackMode {
        DirectPlay => "direct_play",
        Remux => "remux",
        Transcode => "transcode",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientDirectPlayPlan {
    pub source_id: String,
    pub content_type: String,
    pub supports_range_requests: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientTranscodePlan {
    pub output_container: ClientOutputContainer,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub hardware_acceleration: ClientHardwareAcceleration,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeSessionResponse {
    pub session: TranscodeSessionDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeSessionDto {
    pub id: String,
    pub source_id: String,
    pub kind: ClientTranscodeSessionKind,
    pub request_key: String,
    pub state: ClientTranscodeSessionState,
    pub failure_category: Option<ClientTranscodeFailureCategory>,
    pub failure_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserPlaybackStateResponse {
    pub state: UserPlaybackStateDto,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContinueWatchingResponse {
    pub items: Vec<ContinueWatchingItemDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContinueWatchingItemDto {
    pub item: MediaItemDto,
    pub state: UserPlaybackStateDto,
    pub images: Vec<PublicImageRefDto>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserPlaybackStateDto {
    pub item_id: String,
    pub source_id: Option<String>,
    pub resume_position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub progress_percent: Option<f32>,
    pub watched: bool,
    pub watched_at: Option<String>,
    pub last_played_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UpdatePlaybackProgressRequest {
    pub source_id: Option<String>,
    pub position_ms: u64,
    pub duration_ms: Option<u64>,
    pub reported_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SetWatchedStateRequest {
    pub watched: bool,
    pub source_id: Option<String>,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub marked_at: Option<String>,
}

public_string_value! {
    pub enum ClientTranscodeSessionKind {
        Remux => "remux",
        HlsTranscode => "hls_transcode",
    }
}

public_string_value! {
    pub enum ClientTranscodeSessionState {
        Planned => "planned",
        Starting => "starting",
        Running => "running",
        CancelRequested => "cancel_requested",
        Cancelled => "cancelled",
        Failed => "failed",
        Finished => "finished",
    }
}

public_string_value! {
    pub enum ClientTranscodeFailureCategory {
        InvalidRequest => "invalid_request",
        Runner => "runner",
        Timeout => "timeout",
        Storage => "storage",
        Stale => "stale",
        Cancelled => "cancelled",
        Unknown => "unknown",
    }
}

public_string_value! {
    pub enum ClientOutputContainer {
        Hls => "hls",
        Mp4 => "mp4",
        Mkv => "mkv",
    }
}

public_string_value! {
    pub enum ClientHardwareAcceleration {
        None => "none",
        Vaapi => "vaapi",
        Nvenc => "nvenc",
        QuickSync => "quick_sync",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaStreamDto {
    pub index: u32,
    pub kind: ClientMediaStreamKind,
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
pub struct PersonDto {
    pub id: String,
    pub name: String,
    pub sort_name: Option<String>,
    pub overview: Option<String>,
    pub external_ids: Vec<ExternalIdDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemCreditDto {
    pub item_id: String,
    pub person_id: String,
    pub role: ClientCreditRole,
    pub character: Option<String>,
    pub sort_order: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenreDto {
    pub id: String,
    pub name: String,
    pub source: ClientMetadataSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemGenreDto {
    pub item_id: String,
    pub genre_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TagDto {
    pub id: String,
    pub name: String,
    pub source: ClientMetadataSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemTagDto {
    pub item_id: String,
    pub tag_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollectionItemDto {
    pub collection_id: String,
    pub item_id: String,
    pub sort_order: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemStudioDto {
    pub item_id: String,
    pub studio_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PublicImageRefDto {
    pub id: String,
    pub owner: ClientImageOwner,
    pub kind: ClientImageKind,
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub language: Option<String>,
    pub media_type: Option<String>,
    pub etag: Option<String>,
}

public_string_value! {
    pub enum ClientMediaKind {
        Movie => "movie",
        Series => "series",
        Season => "season",
        Episode => "episode",
        Collection => "collection",
        Extra => "extra",
        Unknown => "unknown",
    }
}

public_string_value! {
    pub enum ClientMediaDomain {
        Video => "video",
        Audio => "audio",
        Image => "image",
        Document => "document",
        Mixed => "mixed",
        Online => "online",
    }
}

public_string_value! {
    pub enum ClientLibraryPreset {
        Movies => "movies",
        Tv => "tv",
        Anime => "anime",
        Music => "music",
        Podcast => "podcast",
        Photos => "photos",
        HomeVideo => "home_video",
        MixedVideo => "mixed_video",
        OnlineCatalog => "online_catalog",
        Custom => "custom",
    }
}

public_string_value! {
    pub enum ClientNamingStrategy {
        Movie => "movie",
        Series => "series",
        Anime => "anime",
        Music => "music",
        Podcast => "podcast",
        Photo => "photo",
        HomeVideo => "home_video",
        Mixed => "mixed",
        OnlineCatalog => "online_catalog",
    }
}

public_string_value! {
    pub enum ClientLocalMetadataReader {
        Nfo => "nfo",
        Embedded => "embedded",
        Sidecar => "sidecar",
    }
}

public_string_value! {
    pub enum ClientMetadataRefreshMode {
        None => "none",
        ValidationOnly => "validation_only",
        Default => "default",
        MissingOnly => "missing_only",
        FullRefresh => "full_refresh",
    }
}

public_string_value! {
    pub enum ClientLocalMetadataPolicy {
        Disabled => "disabled",
        ReadOnly => "read_only",
        LocalFirst => "local_first",
        RemoteFirst => "remote_first",
        WriteSidecar => "write_sidecar",
    }
}

public_string_value! {
    pub enum ClientExternalProvider {
        Tmdb => "tmdb",
        Douban => "douban",
        Bangumi => "bangumi",
        Imdb => "imdb",
        Local => "local",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientMetadataSource {
    Local,
    Nfo,
    Provider(ClientExternalProvider),
    User,
    Addon(String),
}

public_string_value! {
    pub enum ClientImageKind {
        Poster => "poster",
        Backdrop => "backdrop",
        Logo => "logo",
        Thumbnail => "thumbnail",
        Banner => "banner",
    }
}

public_string_value! {
    pub enum ClientCreditRole {
        Actor => "actor",
        Director => "director",
        Writer => "writer",
        Producer => "producer",
        Creator => "creator",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientImageOwner {
    Item(String),
    Person(String),
    Collection(String),
    Studio(String),
}

public_string_value! {
    pub enum ClientMediaStreamKind {
        Video => "video",
        Audio => "audio",
        Subtitle => "subtitle",
        Data => "data",
        Attachment => "attachment",
    }
}
