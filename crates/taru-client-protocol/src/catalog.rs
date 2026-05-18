use serde::{Deserialize, Serialize};

use crate::PageInfo;

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
    pub images: Vec<ImageAssetDto>,
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
    pub images: Vec<ImageAssetDto>,
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
    pub images: Vec<ImageRefDto>,
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
pub struct ImageRefDto {
    pub kind: ClientImageKind,
    pub uri: String,
    pub provider: ClientExternalProvider,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub language: Option<String>,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientPlaybackMode {
    DirectPlay,
    Remux,
    Transcode,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientTranscodeSessionKind {
    Remux,
    HlsTranscode,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientTranscodeSessionState {
    Planned,
    Starting,
    Running,
    CancelRequested,
    Cancelled,
    Failed,
    Finished,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientTranscodeFailureCategory {
    InvalidRequest,
    Runner,
    Timeout,
    Storage,
    Stale,
    Cancelled,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientOutputContainer {
    Hls,
    Mp4,
    Mkv,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientHardwareAcceleration {
    None,
    Vaapi,
    Nvenc,
    QuickSync,
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
pub struct ImageAssetDto {
    pub id: String,
    pub owner: ClientImageOwner,
    pub kind: ClientImageKind,
    pub source_uri: String,
    pub provider: ClientExternalProvider,
    pub cache_uri: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub language: Option<String>,
    pub selected: bool,
    pub content_hash: Option<String>,
    pub etag: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientMediaKind {
    Movie,
    Series,
    Season,
    Episode,
    Collection,
    Extra,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientMediaDomain {
    Video,
    Audio,
    Image,
    Document,
    Mixed,
    Online,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientLibraryPreset {
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientNamingStrategy {
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

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientLocalMetadataReader {
    Nfo,
    Embedded,
    Sidecar,
    Other(String),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientMetadataRefreshMode {
    None,
    ValidationOnly,
    Default,
    MissingOnly,
    FullRefresh,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientLocalMetadataPolicy {
    Disabled,
    ReadOnly,
    LocalFirst,
    RemoteFirst,
    WriteSidecar,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientExternalProvider {
    Tmdb,
    Douban,
    Bangumi,
    Imdb,
    Local,
    Other(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientMetadataSource {
    Local,
    Nfo,
    Provider(ClientExternalProvider),
    User,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientImageKind {
    Poster,
    Backdrop,
    Logo,
    Thumbnail,
    Banner,
    Other(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientCreditRole {
    Actor,
    Director,
    Writer,
    Producer,
    Creator,
    Other(String),
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientImageOwner {
    Item(String),
    Person(String),
    Collection(String),
    Studio(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientMediaStreamKind {
    Video,
    Audio,
    Subtitle,
    Data,
    Attachment,
    Other(String),
}
