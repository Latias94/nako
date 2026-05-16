use serde::{Deserialize, Serialize};
use taru_addon_protocol::{AddonManifest, AddonScope};
pub use taru_client_protocol::{
    CLIENT_PROTOCOL_VERSION as API_VERSION, ErrorResponse, HealthResponse, PageInfo,
};
use taru_core::{
    AddonId, AddonRegistrationRecord, AddonStatus, AutomationArtifactRecord, AutomationCapability,
    AutomationJobInput, AutomationProviderConfigRecord, AutomationProviderId,
    AutomationProviderStatus, CanonicalMetadata, CollectionItem, CollectionRef, ContentRating,
    Credit, CreditRole, EventId, ExternalId, ExternalProvider, Genre, GenreId, ImageAsset,
    ImageAssetId, ImageKind, ImageOwner, ImageRef, IngestionFailureClass, IngestionFailurePhase,
    IngestionFailureRecord, IngestionFailureStatus, ItemCredit, ItemGenre, ItemStudio, ItemTag,
    Job, JobId, JobKind, JobStatus, Library, LibraryId, LibraryOptions, LibraryPreset,
    LibraryScanOptions, LocalMetadataPolicy, LocalMetadataReader, MediaDomain, MediaItem,
    MediaItemId, MediaKind, MediaProbeResult, MediaSource, MediaSourceId, MediaStreamInfo,
    MediaStreamKind, MetadataProfile, MetadataProviderAttemptRecord, MetadataRefreshMode,
    MetadataSource, NamingStrategy, OutboxEventRecord, PageRequest, Person, PersonId,
    ProviderRawResponse, ProviderRawResponseCleanup, ScanSnapshotId, StudioId, StudioRef, Tag,
    TagId, TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionKind,
    TranscodeSessionRecord, TranscodeSessionState, WebhookDeliveryAttemptRecord, WebhookEndpointId,
    WebhookEndpointRecord, WebhookEndpointStatus,
};
use taru_streaming::PlaybackDecision;

#[must_use]
pub fn page_info_from_request(page: PageRequest, returned: usize) -> PageInfo {
    let page = page.clamped();

    PageInfo::new(
        page.limit,
        page.offset,
        u32::try_from(returned).unwrap_or(u32::MAX),
    )
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct JobResponse {
    pub id: JobId,
    pub kind: JobKind,
    pub status: JobStatus,
    pub resource_class: String,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub input: Option<serde_json::Value>,
    pub summary: Option<serde_json::Value>,
    pub error: Option<String>,
    pub queued_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl JobResponse {
    #[must_use]
    pub fn from_job(job: Job) -> Self {
        Self {
            id: job.id,
            kind: job.kind,
            status: job.status,
            resource_class: job.resource_class,
            library_id: job.library_id,
            source_id: job.source_id,
            input: job
                .input_json
                .and_then(|value| serde_json::from_str(&value).ok()),
            summary: job
                .summary_json
                .and_then(|value| serde_json::from_str(&value).ok()),
            error: job.error,
            queued_at: job.queued_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeSessionResponse {
    pub session: TranscodeSessionDto,
}

impl TranscodeSessionResponse {
    #[must_use]
    pub fn from_session(session: TranscodeSessionRecord) -> Self {
        Self {
            session: session.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeSessionDto {
    pub id: TranscodeSessionId,
    pub source_id: MediaSourceId,
    pub kind: TranscodeSessionKind,
    pub request_key: String,
    pub output_path: String,
    pub state: TranscodeSessionState,
    pub failure_category: Option<TranscodeFailureCategory>,
    pub failure_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl From<TranscodeSessionRecord> for TranscodeSessionDto {
    fn from(session: TranscodeSessionRecord) -> Self {
        Self {
            id: session.id,
            source_id: session.source_id,
            kind: session.kind,
            request_key: session.request_key,
            output_path: session.output_path.display().to_string(),
            state: session.state,
            failure_category: session.failure_category,
            failure_message: session.failure_message,
            created_at: session.created_at,
            updated_at: session.updated_at,
            started_at: session.started_at,
            completed_at: session.completed_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryListResponse {
    pub libraries: Vec<LibraryDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibrarySourcesResponse {
    pub library: LibraryDto,
    pub sources: Vec<LibrarySourceResponse>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryDto {
    pub id: LibraryId,
    pub name: String,
    pub roots: Vec<String>,
    pub options: LibraryOptionsDto,
}

impl From<Library> for LibraryDto {
    fn from(library: Library) -> Self {
        Self {
            id: library.id,
            name: library.name,
            roots: library.roots,
            options: library.options.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryOptionsDto {
    pub domain: MediaDomain,
    pub preset: LibraryPreset,
    pub scan: LibraryScanOptionsDto,
    pub naming_strategy: NamingStrategy,
    pub metadata_profile: MetadataProfileDto,
}

impl From<LibraryOptions> for LibraryOptionsDto {
    fn from(options: LibraryOptions) -> Self {
        Self {
            domain: options.domain,
            preset: options.preset,
            scan: options.scan.into(),
            naming_strategy: options.naming_strategy,
            metadata_profile: options.metadata_profile.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryScanOptionsDto {
    pub realtime_monitor: bool,
    pub max_depth: Option<usize>,
}

impl From<LibraryScanOptions> for LibraryScanOptionsDto {
    fn from(options: LibraryScanOptions) -> Self {
        Self {
            realtime_monitor: options.realtime_monitor,
            max_depth: options.max_depth,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProfileDto {
    pub item_kinds: Vec<MediaKind>,
    pub local_readers: Vec<LocalMetadataReader>,
    pub metadata_providers: Vec<ExternalProvider>,
    pub image_providers: Vec<ExternalProvider>,
    pub language: Option<String>,
    pub country: Option<String>,
    pub refresh_mode: MetadataRefreshMode,
    pub local_metadata_policy: LocalMetadataPolicy,
}

impl From<MetadataProfile> for MetadataProfileDto {
    fn from(profile: MetadataProfile) -> Self {
        Self {
            item_kinds: profile.item_kinds,
            local_readers: profile.local_readers,
            metadata_providers: profile.metadata_providers,
            image_providers: profile.image_providers,
            language: profile.language,
            country: profile.country,
            refresh_mode: profile.refresh_mode,
            local_metadata_policy: profile.local_metadata_policy,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IngestionFailuresResponse {
    pub library_id: LibraryId,
    pub failures: Vec<IngestionFailureDiagnostic>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IngestionFailureDiagnostic {
    #[serde(flatten)]
    pub failure: IngestionFailureDto,
    pub retryable_now: bool,
}

impl IngestionFailureDiagnostic {
    #[must_use]
    pub fn from_record(failure: IngestionFailureRecord) -> Self {
        let retryable_now = failure.status == IngestionFailureStatus::Open && failure.retryable;
        Self {
            failure: failure.into(),
            retryable_now,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IngestionFailureDto {
    pub library_id: LibraryId,
    pub job_id: Option<JobId>,
    pub scan_id: Option<ScanSnapshotId>,
    pub source_id: Option<MediaSourceId>,
    pub phase: IngestionFailurePhase,
    pub target_uri: String,
    pub target_kind: String,
    pub failure_class: IngestionFailureClass,
    pub status: IngestionFailureStatus,
    pub message: String,
    pub retryable: bool,
    pub attempts: u32,
    pub first_failed_at_ms: i64,
    pub last_failed_at_ms: i64,
    pub resolved_at_ms: Option<i64>,
    pub ignored_at_ms: Option<i64>,
}

impl From<IngestionFailureRecord> for IngestionFailureDto {
    fn from(failure: IngestionFailureRecord) -> Self {
        Self {
            library_id: failure.library_id,
            job_id: failure.job_id,
            scan_id: failure.scan_id,
            source_id: failure.source_id,
            phase: failure.phase,
            target_uri: failure.target_uri,
            target_kind: failure.target_kind,
            failure_class: failure.failure_class,
            status: failure.status,
            message: failure.message,
            retryable: failure.retryable,
            attempts: failure.attempts,
            first_failed_at_ms: failure.first_failed_at_ms,
            last_failed_at_ms: failure.last_failed_at_ms,
            resolved_at_ms: failure.resolved_at_ms,
            ignored_at_ms: failure.ignored_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IgnoreIngestionFailureRequest {
    pub phase: IngestionFailurePhase,
    pub target_uri: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibrarySourceResponse {
    pub source: MediaSourceDto,
    pub item: Option<MediaItemDto>,
    pub probe: Option<MediaProbeDto>,
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
    pub item_id: taru_core::MediaItemId,
    pub credits: Vec<ItemCreditDto>,
    pub people: Vec<PersonDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImagesResponse {
    pub item_id: taru_core::MediaItemId,
    pub images: Vec<ImageAssetDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackDecisionResponse {
    pub source: MediaSourceDto,
    pub probe: Option<MediaProbeDto>,
    pub decision: PlaybackDecision,
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
    pub source_id: MediaSourceId,
    pub probe: MediaProbeDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaItemDto {
    pub id: MediaItemId,
    pub kind: MediaKind,
    pub parent_id: Option<MediaItemId>,
    pub metadata: CanonicalMetadataDto,
}

impl From<MediaItem> for MediaItemDto {
    fn from(item: MediaItem) -> Self {
        Self {
            id: item.id,
            kind: item.kind,
            parent_id: item.parent_id,
            metadata: item.metadata.into(),
        }
    }
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
    pub ratings: Vec<ContentRating>,
    pub images: Vec<ImageRef>,
    pub credits: Vec<Credit>,
    pub collections: Vec<CollectionRef>,
    pub studios: Vec<StudioRef>,
    pub external_ids: Vec<ExternalId>,
}

impl From<CanonicalMetadata> for CanonicalMetadataDto {
    fn from(metadata: CanonicalMetadata) -> Self {
        Self {
            title: metadata.title,
            original_title: metadata.original_title,
            sort_title: metadata.sort_title,
            overview: metadata.overview,
            release_date: metadata.release_date,
            runtime_minutes: metadata.runtime_minutes,
            tagline: metadata.tagline,
            genres: metadata.genres,
            tags: metadata.tags,
            ratings: metadata.ratings,
            images: metadata.images,
            credits: metadata.credits,
            collections: metadata.collections,
            studios: metadata.studios,
            external_ids: metadata.external_ids,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaSourceDto {
    pub id: MediaSourceId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub locator: String,
    pub file_name: String,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
}

impl From<MediaSource> for MediaSourceDto {
    fn from(source: MediaSource) -> Self {
        Self {
            id: source.id,
            library_id: source.library_id,
            item_id: source.item_id,
            locator: source.locator,
            file_name: source.file_name,
            size_bytes: source.size_bytes,
            fingerprint: source.fingerprint,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaProbeDto {
    pub duration_ms: Option<u64>,
    pub container: Option<String>,
    pub bit_rate: Option<u64>,
    pub streams: Vec<MediaStreamDto>,
}

impl From<MediaProbeResult> for MediaProbeDto {
    fn from(probe: MediaProbeResult) -> Self {
        Self {
            duration_ms: probe.duration_ms,
            container: probe.container,
            bit_rate: probe.bit_rate,
            streams: probe.streams.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaStreamDto {
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

impl From<MediaStreamInfo> for MediaStreamDto {
    fn from(stream: MediaStreamInfo) -> Self {
        Self {
            index: stream.index,
            kind: stream.kind,
            codec: stream.codec,
            language: stream.language,
            duration_ms: stream.duration_ms,
            bit_rate: stream.bit_rate,
            width: stream.width,
            height: stream.height,
            channels: stream.channels,
            sample_rate: stream.sample_rate,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PersonDto {
    pub id: PersonId,
    pub name: String,
    pub sort_name: Option<String>,
    pub overview: Option<String>,
    pub external_ids: Vec<ExternalId>,
}

impl From<Person> for PersonDto {
    fn from(person: Person) -> Self {
        Self {
            id: person.id,
            name: person.name,
            sort_name: person.sort_name,
            overview: person.overview,
            external_ids: person.external_ids,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemCreditDto {
    pub item_id: MediaItemId,
    pub person_id: PersonId,
    pub role: CreditRole,
    pub character: Option<String>,
    pub sort_order: Option<u32>,
}

impl From<ItemCredit> for ItemCreditDto {
    fn from(credit: ItemCredit) -> Self {
        Self {
            item_id: credit.item_id,
            person_id: credit.person_id,
            role: credit.role,
            character: credit.character,
            sort_order: credit.sort_order,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenreDto {
    pub id: GenreId,
    pub name: String,
    pub source: MetadataSource,
}

impl From<Genre> for GenreDto {
    fn from(genre: Genre) -> Self {
        Self {
            id: genre.id,
            name: genre.name,
            source: genre.source,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemGenreDto {
    pub item_id: MediaItemId,
    pub genre_id: GenreId,
}

impl From<ItemGenre> for ItemGenreDto {
    fn from(genre: ItemGenre) -> Self {
        Self {
            item_id: genre.item_id,
            genre_id: genre.genre_id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TagDto {
    pub id: TagId,
    pub name: String,
    pub source: MetadataSource,
}

impl From<Tag> for TagDto {
    fn from(tag: Tag) -> Self {
        Self {
            id: tag.id,
            name: tag.name,
            source: tag.source,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemTagDto {
    pub item_id: MediaItemId,
    pub tag_id: TagId,
}

impl From<ItemTag> for ItemTagDto {
    fn from(tag: ItemTag) -> Self {
        Self {
            item_id: tag.item_id,
            tag_id: tag.tag_id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollectionItemDto {
    pub collection_id: taru_core::CollectionId,
    pub item_id: MediaItemId,
    pub sort_order: Option<u32>,
}

impl From<CollectionItem> for CollectionItemDto {
    fn from(collection: CollectionItem) -> Self {
        Self {
            collection_id: collection.collection_id,
            item_id: collection.item_id,
            sort_order: collection.sort_order,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemStudioDto {
    pub item_id: MediaItemId,
    pub studio_id: StudioId,
}

impl From<ItemStudio> for ItemStudioDto {
    fn from(studio: ItemStudio) -> Self {
        Self {
            item_id: studio.item_id,
            studio_id: studio.studio_id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImageAssetDto {
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

impl From<ImageAsset> for ImageAssetDto {
    fn from(image: ImageAsset) -> Self {
        Self {
            id: image.id,
            owner: image.owner,
            kind: image.kind,
            source_uri: image.source_uri,
            provider: image.provider,
            cache_uri: image.cache_uri,
            width: image.width,
            height: image.height,
            language: image.language,
            selected: image.selected,
            content_hash: image.content_hash,
            etag: image.etag,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderAttemptsResponse {
    pub item_id: MediaItemId,
    pub attempts: Vec<MetadataProviderAttemptDiagnostic>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderAttemptDiagnostic {
    #[serde(flatten)]
    pub attempt: MetadataProviderAttemptRecord,
    pub retryable: bool,
}

impl MetadataProviderAttemptDiagnostic {
    #[must_use]
    pub fn from_record(attempt: MetadataProviderAttemptRecord) -> Self {
        Self {
            retryable: attempt.is_retryable(),
            attempt,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRawResponsesResponse {
    pub item_id: MediaItemId,
    pub responses: Vec<ProviderRawResponse>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderDiagnosticsResponse {
    pub providers: Vec<MetadataProviderDiagnostic>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderDiagnostic {
    pub provider: ExternalProvider,
    pub status: MetadataProviderDiagnosticStatus,
    pub provider_name: Option<String>,
    pub reason: Option<String>,
    pub runtime: MetadataProviderRuntimeDiagnostic,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataProviderDiagnosticStatus {
    Available,
    Disabled,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderRuntimeDiagnostic {
    pub timeout_ms: u64,
    pub max_attempts: u32,
    pub min_interval_ms: u64,
    pub concurrency: usize,
    pub user_agent: String,
    pub proxy_configured: bool,
    pub circuit_breaker_failures: u32,
    pub circuit_breaker_backoff_ms: u64,
    pub circuit_open: bool,
    pub circuit_open_until_ms: Option<u64>,
    pub consecutive_failures: u64,
    pub last_error: Option<String>,
    pub last_rate_limit_wait_ms: u64,
    pub state_scope: MetadataProviderRuntimeStateScope,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataProviderRuntimeStateScope {
    ProcessLocal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendDiagnosticsResponse {
    pub backends: Vec<StorageBackendDiagnostic>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendDiagnostic {
    pub library_id: LibraryId,
    pub library_name: String,
    pub root_uri: String,
    pub backend_kind: StorageBackendKind,
    pub scheme: String,
    pub status: StorageBackendStatus,
    pub reason: Option<String>,
    pub registry: StorageBackendRegistryDiagnostic,
    pub health: StorageBackendHealthDiagnostic,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendKind {
    Local,
    WebDav,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendRegistryDiagnostic {
    pub cached: bool,
    pub stream_permits_available: usize,
    pub stream_permits_max: usize,
    pub stage_permits_available: usize,
    pub stage_permits_max: usize,
    pub state_scope: StorageBackendRuntimeStateScope,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendHealthDiagnostic {
    pub consecutive_errors: u64,
    pub last_success_at_ms: Option<i64>,
    pub last_error_at_ms: Option<i64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendRuntimeStateScope {
    ProcessLocal,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EnqueueMetadataMaintenanceRequest {
    pub library_id: Option<LibraryId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub item_ids: Vec<MediaItemId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Vec<ExternalProvider>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub item_kinds: Vec<MediaKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<MetadataProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_mode: Option<MetadataRefreshMode>,
    #[serde(default)]
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataMaintenancePlanResponse {
    pub request: EnqueueMetadataMaintenanceRequest,
    pub planned_items: u32,
    pub skipped_items: u32,
    pub items: Vec<MetadataMaintenancePlanItem>,
    pub errors: Vec<MetadataMaintenancePlanError>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataMaintenancePlanItem {
    pub item_id: MediaItemId,
    pub library_id: Option<LibraryId>,
    pub kind: MediaKind,
    pub title: String,
    pub providers: Vec<ExternalProvider>,
    pub language: Option<String>,
    pub refresh_mode: MetadataRefreshMode,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataMaintenancePlanError {
    pub item_id: MediaItemId,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRawCleanupResponse {
    pub cleanup: ProviderRawResponseCleanup,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UpsertWebhookEndpointRequest {
    pub id: Option<WebhookEndpointId>,
    pub name: String,
    pub url: String,
    pub secret_env: Option<String>,
    pub subscribed_event_kinds: Vec<String>,
    pub timeout_ms: Option<u64>,
    pub max_attempts: Option<u32>,
    pub status: WebhookEndpointStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebhookEndpointResponse {
    pub endpoint: WebhookEndpointRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebhookEndpointsResponse {
    pub endpoints: Vec<WebhookEndpointRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebhookDeliveryAttemptsResponse {
    pub event_id: EventId,
    pub attempts: Vec<WebhookDeliveryAttemptRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebhookDispatchResponse {
    pub event: OutboxEventRecord,
    pub attempted_endpoints: u32,
    pub delivered: u32,
    pub failed: u32,
    pub skipped_endpoints: u32,
    pub attempts: Vec<WebhookDeliveryAttemptRecord>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UpsertAutomationProviderRequest {
    pub id: Option<AutomationProviderId>,
    pub name: String,
    pub base_url: String,
    pub secret_env: Option<String>,
    pub capabilities: Vec<AutomationCapability>,
    pub timeout_ms: Option<u64>,
    pub max_attempts: Option<u32>,
    pub status: AutomationProviderStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationProviderResponse {
    pub provider: AutomationProviderConfigRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationProvidersResponse {
    pub providers: Vec<AutomationProviderConfigRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EnqueueAutomationJobRequest {
    pub provider_id: AutomationProviderId,
    pub capability: AutomationCapability,
    pub library_id: Option<LibraryId>,
    pub item_id: Option<MediaItemId>,
    pub source_id: Option<MediaSourceId>,
    pub prompt: serde_json::Value,
    pub idempotency_key: String,
}

impl EnqueueAutomationJobRequest {
    pub fn into_job_input(self) -> Result<AutomationJobInput, serde_json::Error> {
        Ok(AutomationJobInput {
            provider_id: self.provider_id,
            capability: self.capability,
            library_id: self.library_id,
            item_id: self.item_id,
            source_id: self.source_id,
            prompt_json: serde_json::to_string(&self.prompt)?,
            idempotency_key: self.idempotency_key,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationArtifactsResponse {
    pub artifacts: Vec<AutomationArtifactRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RegisterAddonRequest {
    pub id: Option<AddonId>,
    pub manifest: AddonManifest,
    #[serde(default)]
    pub granted_scopes: Vec<AddonScope>,
    #[serde(default)]
    pub status: Option<AddonStatus>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRegistrationResponse {
    pub addon: AddonRegistrationRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRegistrationsResponse {
    pub addons: Vec<AddonRegistrationRecord>,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use taru_core::{
        CanonicalMetadata, IngestionFailureClass, MediaItem, TranscodeSessionKind,
        TranscodeSessionState,
    };

    #[test]
    fn page_info_adapter_keeps_server_pagination_out_of_protocol_types() {
        let page = PageRequest {
            limit: 25,
            offset: 50,
        };

        let info = page_info_from_request(page, usize::MAX);

        assert_eq!(info.limit, 25);
        assert_eq!(info.offset, 50);
        assert_eq!(info.returned, u32::MAX);
    }

    #[test]
    fn media_item_dto_serializes_field_level_payload() {
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "DTO Demo".to_owned(),
                tags: vec!["favorite".to_owned()],
                ..CanonicalMetadata::default()
            },
        };

        let value = serde_json::to_value(MediaItemDto::from(item)).unwrap();

        assert_eq!(value["kind"], "movie");
        assert_eq!(value["metadata"]["title"], "DTO Demo");
        assert_eq!(value["metadata"]["tags"][0], "favorite");
        assert!(value.get("input_json").is_none());
    }

    #[test]
    fn ingestion_failure_diagnostic_serializes_explicit_dto_fields() {
        let record = IngestionFailureRecord {
            library_id: LibraryId::new(),
            job_id: Some(JobId::new()),
            scan_id: Some(ScanSnapshotId::new()),
            source_id: None,
            phase: IngestionFailurePhase::Scan,
            target_uri: "webdav:///Movies/Broken/".to_owned(),
            target_kind: "directory".to_owned(),
            failure_class: IngestionFailureClass::Storage,
            status: IngestionFailureStatus::Open,
            message: "failed to list directory".to_owned(),
            retryable: true,
            attempts: 2,
            first_failed_at_ms: 10,
            last_failed_at_ms: 20,
            resolved_at_ms: None,
            ignored_at_ms: None,
        };

        let diagnostic = IngestionFailureDiagnostic::from_record(record);
        let value = serde_json::to_value(&diagnostic).unwrap();

        assert_eq!(diagnostic.failure.attempts, 2);
        assert!(diagnostic.retryable_now);
        assert_eq!(value["phase"], "scan");
        assert_eq!(value["failure_class"], "storage");
        assert_eq!(value["status"], "open");
        assert!(value.get("failure").is_none());
    }

    #[test]
    fn transcode_session_response_serializes_path_as_string() {
        let session = TranscodeSessionRecord {
            id: TranscodeSessionId::new(),
            source_id: MediaSourceId::new(),
            kind: TranscodeSessionKind::Remux,
            request_key: "remux:mp4".to_owned(),
            output_path: PathBuf::from("cache/remux/output.mp4"),
            state: TranscodeSessionState::Finished,
            failure_category: None,
            failure_message: None,
            created_at: "2026-05-16T00:00:00Z".to_owned(),
            updated_at: "2026-05-16T00:01:00Z".to_owned(),
            started_at: Some("2026-05-16T00:00:01Z".to_owned()),
            completed_at: Some("2026-05-16T00:01:00Z".to_owned()),
        };

        let response = TranscodeSessionResponse::from_session(session);
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["session"]["kind"], "remux");
        assert!(
            value["session"]["output_path"]
                .as_str()
                .unwrap()
                .contains("output.mp4")
        );
    }
}
