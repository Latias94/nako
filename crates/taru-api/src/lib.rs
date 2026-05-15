use serde::{Deserialize, Serialize};
use taru_addon_protocol::{AddonManifest, AddonScope};
use taru_core::{
    AddonId, AddonRegistrationRecord, AddonStatus, AutomationArtifactRecord, AutomationCapability,
    AutomationJobInput, AutomationProviderConfigRecord, AutomationProviderId,
    AutomationProviderStatus, CollectionItem, EventId, ExternalProvider, Genre, ImageAsset,
    ItemCredit, ItemGenre, ItemStudio, ItemTag, Job, JobId, JobKind, JobStatus, Library, LibraryId,
    MediaItem, MediaItemId, MediaProbeResult, MediaSource, MediaSourceId,
    MetadataProviderAttemptRecord, OutboxEventRecord, PageRequest, Person, ProviderRawResponse,
    Tag, TranscodeSessionRecord, WebhookDeliveryAttemptRecord, WebhookEndpointId,
    WebhookEndpointRecord, WebhookEndpointStatus,
};
use taru_streaming::PlaybackDecision;

pub const API_VERSION: &str = "v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PageInfo {
    pub limit: u32,
    pub offset: u64,
    pub returned: u32,
}

impl PageInfo {
    #[must_use]
    pub fn new(page: PageRequest, returned: usize) -> Self {
        let page = page.clamped();

        Self {
            limit: page.limit,
            offset: page.offset,
            returned: u32::try_from(returned).unwrap_or(u32::MAX),
        }
    }
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
    pub session: TranscodeSessionRecord,
}

impl TranscodeSessionResponse {
    #[must_use]
    pub fn from_session(session: TranscodeSessionRecord) -> Self {
        Self { session }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryListResponse {
    pub libraries: Vec<Library>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibrarySourcesResponse {
    pub library: Library,
    pub sources: Vec<LibrarySourceResponse>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibrarySourceResponse {
    pub source: MediaSource,
    pub item: Option<MediaItem>,
    pub probe: Option<MediaProbeResult>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemsResponse {
    pub items: Vec<MediaItem>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemDetailResponse {
    pub item: MediaItem,
    pub sources: Vec<MediaSource>,
    pub credits: Vec<ItemCredit>,
    pub genres: Vec<ItemGenre>,
    pub tags: Vec<ItemTag>,
    pub collections: Vec<CollectionItem>,
    pub studios: Vec<ItemStudio>,
    pub images: Vec<ImageAsset>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemCreditsResponse {
    pub item_id: taru_core::MediaItemId,
    pub credits: Vec<ItemCredit>,
    pub people: Vec<Person>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImagesResponse {
    pub item_id: taru_core::MediaItemId,
    pub images: Vec<ImageAsset>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackDecisionResponse {
    pub source: MediaSource,
    pub probe: Option<MediaProbeResult>,
    pub decision: PlaybackDecision,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PeopleResponse {
    pub people: Vec<Person>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PersonItemsResponse {
    pub person: Person,
    pub items: Vec<MediaItem>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TagsResponse {
    pub tags: Vec<Tag>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TagItemsResponse {
    pub tag: Tag,
    pub items: Vec<MediaItem>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenreListResponse {
    pub genres: Vec<Genre>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenreItemsResponse {
    pub genre: Genre,
    pub items: Vec<MediaItem>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchItemHit>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SearchItemHit {
    pub item: MediaItem,
    pub score: f32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceProbeResponse {
    pub source_id: MediaSourceId,
    pub probe: MediaProbeResult,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderAttemptsResponse {
    pub item_id: MediaItemId,
    pub attempts: Vec<MetadataProviderAttemptRecord>,
    pub page: PageInfo,
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
