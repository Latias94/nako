use async_trait::async_trait;

use crate::{
    AddonId, AddonRegistrationRecord, AddonStatus, ArtworkTask, ArtworkTaskId,
    AutomationArtifactId, AutomationArtifactRecord, AutomationArtifactStatus,
    AutomationProviderConfigRecord, AutomationProviderId, Collection, CollectionId, CollectionItem,
    DirectorySnapshot, DomainEventKind, Genre, GenreId, ImageAsset, ImageAssetId, ItemCredit,
    ItemGenre, ItemStudio, ItemTag, Job, JobId, Library, LibraryId, MediaItem, MediaItemId,
    MediaProbeResult, MediaSource, MediaSourceId, MetadataFieldLock, NewAddonRegistration,
    NewAutomationArtifact, NewAutomationProviderConfig, NewJob, NewOutboxEvent,
    NewTranscodeSession, NewWebhookDeliveryAttempt, NewWebhookEndpoint, OutboxEventRecord, Person,
    PersonId, ProviderRawResponse, Result, ScanSnapshot, ScanSnapshotId, SourceState, Studio,
    StudioId, Tag, TagId, TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionKind,
    TranscodeSessionRecord, TranscodeSessionState, WebhookDeliveryAttemptId,
    WebhookDeliveryAttemptRecord, WebhookDeliveryStatus, WebhookEndpointId, WebhookEndpointRecord,
};

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn migrate(&self) -> Result<()>;
}

#[async_trait]
pub trait LibraryRepository: Send + Sync {
    async fn upsert_library(&self, library: &Library) -> Result<()>;

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>>;

    async fn list_libraries(&self, page: PageRequest) -> Result<Vec<Library>>;
}

#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn upsert_media_item(&self, item: &MediaItem) -> Result<()>;

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>>;

    async fn list_media_items(&self, page: PageRequest) -> Result<Vec<MediaItem>>;

    async fn upsert_media_source(&self, library_id: LibraryId, source: &MediaSource) -> Result<()>;

    async fn get_media_source(&self, id: MediaSourceId) -> Result<Option<MediaSource>>;

    async fn get_media_source_by_locator(&self, locator: &str) -> Result<Option<MediaSource>>;

    async fn list_item_sources(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<MediaSource>>;

    async fn list_media_sources(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<MediaSource>>;
}

#[async_trait]
pub trait MediaProbeRepository: Send + Sync {
    async fn upsert_media_probe(
        &self,
        source_id: MediaSourceId,
        result: &MediaProbeResult,
    ) -> Result<()>;

    async fn get_media_probe(&self, source_id: MediaSourceId) -> Result<Option<MediaProbeResult>>;
}

#[async_trait]
pub trait CatalogRepository: Send + Sync {
    async fn upsert_person(&self, person: &Person) -> Result<()>;

    async fn get_person(&self, id: PersonId) -> Result<Option<Person>>;

    async fn find_person_by_external_id(
        &self,
        external_id: &crate::ExternalId,
    ) -> Result<Option<Person>>;

    async fn find_person_by_name(&self, name: &str) -> Result<Option<Person>>;

    async fn list_people(&self, page: PageRequest) -> Result<Vec<Person>>;

    async fn upsert_item_credit(&self, credit: &ItemCredit) -> Result<()>;

    async fn clear_item_credits(&self, item_id: MediaItemId) -> Result<()>;

    async fn list_item_credits(&self, item_id: MediaItemId) -> Result<Vec<ItemCredit>>;

    async fn list_person_credits(&self, person_id: PersonId) -> Result<Vec<ItemCredit>>;

    async fn list_person_items(
        &self,
        person_id: PersonId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>>;

    async fn upsert_genre(&self, genre: &Genre) -> Result<()>;

    async fn get_genre(&self, id: GenreId) -> Result<Option<Genre>>;

    async fn find_genre_by_name_source(
        &self,
        name: &str,
        source: &crate::MetadataSource,
    ) -> Result<Option<Genre>>;

    async fn list_genres(&self, page: PageRequest) -> Result<Vec<Genre>>;

    async fn upsert_item_genre(&self, item_genre: &ItemGenre) -> Result<()>;

    async fn clear_item_genres(&self, item_id: MediaItemId) -> Result<()>;

    async fn list_item_genres(&self, item_id: MediaItemId) -> Result<Vec<ItemGenre>>;

    async fn list_genre_items(
        &self,
        genre_id: GenreId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>>;

    async fn upsert_tag(&self, tag: &Tag) -> Result<()>;

    async fn get_tag(&self, id: TagId) -> Result<Option<Tag>>;

    async fn find_tag_by_name_source(
        &self,
        name: &str,
        source: &crate::MetadataSource,
    ) -> Result<Option<Tag>>;

    async fn list_tags(&self, page: PageRequest) -> Result<Vec<Tag>>;

    async fn upsert_item_tag(&self, item_tag: &ItemTag) -> Result<()>;

    async fn clear_item_tags(&self, item_id: MediaItemId) -> Result<()>;

    async fn list_item_tags(&self, item_id: MediaItemId) -> Result<Vec<ItemTag>>;

    async fn list_tag_items(&self, tag_id: TagId, page: PageRequest) -> Result<Vec<MediaItem>>;

    async fn upsert_collection(&self, collection: &Collection) -> Result<()>;

    async fn get_collection(&self, id: CollectionId) -> Result<Option<Collection>>;

    async fn find_collection_by_external_id(
        &self,
        external_id: &crate::ExternalId,
    ) -> Result<Option<Collection>>;

    async fn find_collection_by_name_source(
        &self,
        name: &str,
        source: &crate::MetadataSource,
    ) -> Result<Option<Collection>>;

    async fn list_collections(&self, page: PageRequest) -> Result<Vec<Collection>>;

    async fn upsert_collection_item(&self, item: &CollectionItem) -> Result<()>;

    async fn clear_item_collections(&self, item_id: MediaItemId) -> Result<()>;

    async fn list_item_collections(&self, item_id: MediaItemId) -> Result<Vec<CollectionItem>>;

    async fn list_collection_items(
        &self,
        collection_id: CollectionId,
    ) -> Result<Vec<CollectionItem>>;

    async fn upsert_studio(&self, studio: &Studio) -> Result<()>;

    async fn get_studio(&self, id: StudioId) -> Result<Option<Studio>>;

    async fn find_studio_by_external_id(
        &self,
        external_id: &crate::ExternalId,
    ) -> Result<Option<Studio>>;

    async fn find_studio_by_name_source(
        &self,
        name: &str,
        source: &crate::MetadataSource,
    ) -> Result<Option<Studio>>;

    async fn list_studios(&self, page: PageRequest) -> Result<Vec<Studio>>;

    async fn upsert_item_studio(&self, item_studio: &ItemStudio) -> Result<()>;

    async fn clear_item_studios(&self, item_id: MediaItemId) -> Result<()>;

    async fn list_item_studios(&self, item_id: MediaItemId) -> Result<Vec<ItemStudio>>;

    async fn upsert_image_asset(&self, image: &ImageAsset) -> Result<()>;

    async fn get_image_asset(&self, id: ImageAssetId) -> Result<Option<ImageAsset>>;

    async fn find_image_asset_by_source(
        &self,
        owner: &crate::ImageOwner,
        kind: &crate::ImageKind,
        source_uri: &str,
    ) -> Result<Option<ImageAsset>>;

    async fn list_item_images(&self, item_id: MediaItemId) -> Result<Vec<ImageAsset>>;
}

#[async_trait]
pub trait ScanRepository: Send + Sync {
    async fn begin_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<ScanSnapshot>;

    async fn complete_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        status: crate::ScanStatus,
        error: Option<String>,
    ) -> Result<ScanSnapshot>;

    async fn get_scan_snapshot(&self, id: ScanSnapshotId) -> Result<Option<ScanSnapshot>>;

    async fn upsert_directory_snapshot(&self, snapshot: &DirectorySnapshot) -> Result<()>;

    async fn list_directory_snapshots(
        &self,
        scan_id: ScanSnapshotId,
    ) -> Result<Vec<DirectorySnapshot>>;

    async fn upsert_source_state(&self, state: &SourceState) -> Result<()>;

    async fn get_source_state(
        &self,
        library_id: LibraryId,
        uri: &str,
    ) -> Result<Option<SourceState>>;

    async fn list_source_states(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<SourceState>>;
}

#[async_trait]
pub trait ArtworkTaskRepository: Send + Sync {
    async fn enqueue_artwork_task(&self, task: &ArtworkTask) -> Result<()>;

    async fn get_artwork_task(&self, id: ArtworkTaskId) -> Result<Option<ArtworkTask>>;

    async fn list_artwork_tasks(&self, page: PageRequest) -> Result<Vec<ArtworkTask>>;
}

#[async_trait]
pub trait MetadataRepository: Send + Sync {
    async fn upsert_field_lock(&self, lock: &MetadataFieldLock) -> Result<()>;

    async fn list_field_locks(&self, item_id: MediaItemId) -> Result<Vec<MetadataFieldLock>>;

    async fn upsert_provider_raw_response(&self, response: &ProviderRawResponse) -> Result<()>;

    async fn get_provider_raw_response(
        &self,
        item_id: MediaItemId,
        provider: &crate::ExternalProvider,
        provider_key: &str,
    ) -> Result<Option<ProviderRawResponse>>;
}

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn enqueue_job(&self, job: NewJob) -> Result<Job>;

    async fn start_job(&self, id: JobId) -> Result<Job>;

    async fn succeed_job(&self, id: JobId, summary_json: Option<String>) -> Result<Job>;

    async fn fail_job(&self, id: JobId, error: String) -> Result<Job>;

    async fn get_job(&self, id: JobId) -> Result<Option<Job>>;
}

#[async_trait]
pub trait EventOutboxRepository: Send + Sync {
    async fn enqueue_outbox_event(&self, event: NewOutboxEvent) -> Result<OutboxEventRecord>;

    async fn get_outbox_event(&self, id: crate::EventId) -> Result<Option<OutboxEventRecord>>;

    async fn find_outbox_event_by_idempotency_key(
        &self,
        kind: DomainEventKind,
        idempotency_key: &str,
    ) -> Result<Option<OutboxEventRecord>>;

    async fn list_outbox_events(&self, page: PageRequest) -> Result<Vec<OutboxEventRecord>>;
}

#[async_trait]
pub trait AutomationRepository: Send + Sync {
    async fn upsert_automation_provider(
        &self,
        provider: NewAutomationProviderConfig,
    ) -> Result<AutomationProviderConfigRecord>;

    async fn get_automation_provider(
        &self,
        id: AutomationProviderId,
    ) -> Result<Option<AutomationProviderConfigRecord>>;

    async fn list_enabled_automation_providers(
        &self,
    ) -> Result<Vec<AutomationProviderConfigRecord>>;

    async fn create_automation_artifact(
        &self,
        artifact: NewAutomationArtifact,
    ) -> Result<AutomationArtifactRecord>;

    async fn set_automation_artifact_status(
        &self,
        id: AutomationArtifactId,
        status: AutomationArtifactStatus,
    ) -> Result<AutomationArtifactRecord>;

    async fn list_automation_artifacts_for_job(
        &self,
        job_id: JobId,
    ) -> Result<Vec<AutomationArtifactRecord>>;

    async fn list_automation_artifacts_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<AutomationArtifactRecord>>;
}

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn upsert_webhook_endpoint(
        &self,
        endpoint: NewWebhookEndpoint,
    ) -> Result<WebhookEndpointRecord>;

    async fn get_webhook_endpoint(
        &self,
        id: WebhookEndpointId,
    ) -> Result<Option<WebhookEndpointRecord>>;

    async fn list_enabled_webhook_endpoints(&self) -> Result<Vec<WebhookEndpointRecord>>;

    async fn create_webhook_delivery_attempt(
        &self,
        attempt: NewWebhookDeliveryAttempt,
    ) -> Result<WebhookDeliveryAttemptRecord>;

    async fn set_webhook_delivery_attempt_result(
        &self,
        id: WebhookDeliveryAttemptId,
        status: WebhookDeliveryStatus,
        http_status: Option<u16>,
        error: Option<String>,
        next_retry_at: Option<String>,
    ) -> Result<WebhookDeliveryAttemptRecord>;

    async fn list_webhook_delivery_attempts(
        &self,
        event_id: crate::EventId,
    ) -> Result<Vec<WebhookDeliveryAttemptRecord>>;
}

#[async_trait]
pub trait AddonRepository: Send + Sync {
    async fn upsert_addon_registration(
        &self,
        addon: NewAddonRegistration,
    ) -> Result<AddonRegistrationRecord>;

    async fn get_addon_registration(&self, id: AddonId) -> Result<Option<AddonRegistrationRecord>>;

    async fn find_addon_registration_by_manifest_id(
        &self,
        manifest_id: &str,
    ) -> Result<Option<AddonRegistrationRecord>>;

    async fn list_addon_registrations(
        &self,
        status: Option<AddonStatus>,
    ) -> Result<Vec<AddonRegistrationRecord>>;
}

#[async_trait]
pub trait TranscodeSessionRepository: Send + Sync {
    async fn create_transcode_session(
        &self,
        session: NewTranscodeSession,
    ) -> Result<TranscodeSessionRecord>;

    async fn get_transcode_session(
        &self,
        id: TranscodeSessionId,
    ) -> Result<Option<TranscodeSessionRecord>>;

    async fn find_latest_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>>;

    async fn find_active_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>>;

    async fn set_transcode_session_state(
        &self,
        id: TranscodeSessionId,
        state: TranscodeSessionState,
        failure_category: Option<TranscodeFailureCategory>,
        failure_message: Option<String>,
    ) -> Result<TranscodeSessionRecord>;

    async fn fail_stale_transcode_sessions(
        &self,
        failure_category: TranscodeFailureCategory,
        failure_message: String,
    ) -> Result<u64>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageRequest {
    pub limit: u32,
    pub offset: u64,
}

impl PageRequest {
    pub const DEFAULT_LIMIT: u32 = 50;
    pub const MAX_LIMIT: u32 = 500;

    #[must_use]
    pub const fn new(limit: u32, offset: u64) -> Self {
        Self { limit, offset }
    }

    #[must_use]
    pub const fn first_page() -> Self {
        Self {
            limit: Self::DEFAULT_LIMIT,
            offset: 0,
        }
    }

    #[must_use]
    pub fn clamped(self) -> Self {
        let limit = if self.limit == 0 {
            Self::DEFAULT_LIMIT
        } else {
            self.limit.min(Self::MAX_LIMIT)
        };

        Self {
            limit,
            offset: self.offset,
        }
    }
}
