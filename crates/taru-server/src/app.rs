use std::{
    collections::HashSet,
    env,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use taru_addon_protocol::{ensure_scope_grant, validate_manifest};
use taru_api::{
    AddonRegistrationResponse, AddonRegistrationsResponse, AutomationArtifactsResponse,
    AutomationProviderResponse, AutomationProvidersResponse, EnqueueAutomationJobRequest,
    GenreItemsResponse, GenreListResponse, ImagesResponse, ItemCreditsResponse, ItemDetailResponse,
    ItemsResponse, LibraryListResponse, LibrarySourceResponse, LibrarySourcesResponse, PageInfo,
    PeopleResponse, PersonItemsResponse, RegisterAddonRequest, SearchItemHit, SearchResponse,
    TagItemsResponse, TagsResponse, UpsertAutomationProviderRequest, UpsertWebhookEndpointRequest,
    WebhookDeliveryAttemptsResponse, WebhookDispatchResponse, WebhookEndpointResponse,
    WebhookEndpointsResponse,
};
use taru_automation::AutomationJobService;
use taru_core::{
    AddonId, AddonRegistrationRecord, AddonRepository, AddonStatus, AutomationCapability,
    AutomationProviderId, AutomationRepository, CatalogRepository, DomainEventKind, EventId,
    EventOutboxRepository, GenreId, Job, JobId, Library, LibraryId, LibraryRepository, MediaItemId,
    MediaProbeRepository, MediaRepository, MediaSource, MediaSourceId, NewAddonRegistration,
    NewAutomationProviderConfig, NewOutboxEvent, NewWebhookEndpoint, OutboxEventRecord,
    PageRequest, PersonId, Result, TagId, TaruError, TransactionManager, TranscodeFailureCategory,
    TranscodeSessionRepository, WebhookDeliveryStatus, WebhookEndpointId, WebhookEndpointRecord,
    WebhookRepository,
};
use taru_db::SqliteStore;
use taru_events::{ReqwestWebhookTransport, WebhookDeliveryService, endpoint_subscribes_to};
use taru_search::{SearchIndex, SearchQuery};
use taru_vfs::StorageUri;
use tokio::sync::Semaphore;
use tracing::warn;

use crate::config::{TaruServerConfig, default_library_from_config, libraries_from_config};

mod jobs;
mod metadata;
mod nfo;
pub(crate) mod playback;
mod staging;
mod storage;

#[cfg(test)]
pub(crate) use playback::DirectPlayStreamBody;
pub(crate) use playback::{
    DirectPlaySourceBody, HlsSourceRequest, RemuxSourceDisposition, RemuxSourceRequest,
};
use playback::{HlsAppService, RemuxAppService};
use staging::{ManifestRecordingStorageBackend, cleanup_expired_staging_inputs};
use storage::{LibraryStorageBackend, StorageBackendRegistry, remote_probe_staging_root};

#[cfg(test)]
use playback::plan_direct_play_with_backend;

#[derive(Clone, Debug)]
pub struct TaruApp {
    inner: Arc<TaruAppInner>,
}

#[derive(Debug)]
struct TaruAppInner {
    config: TaruServerConfig,
    store: SqliteStore,
    scan_permits: Arc<Semaphore>,
    metadata_permits: Arc<Semaphore>,
    nfo_permits: Arc<Semaphore>,
    webhook_permits: Arc<Semaphore>,
    storage_backends: StorageBackendRegistry,
    remux: RemuxAppService,
    hls: HlsAppService,
}

#[derive(Clone, Debug)]
struct UnavailableAutomationProvider;

#[async_trait]
impl taru_automation::AutomationProvider for UnavailableAutomationProvider {
    fn descriptor(&self) -> taru_automation::AutomationProviderDescriptor {
        taru_automation::AutomationProviderDescriptor {
            id: AutomationProviderId::new(),
            name: "unavailable".to_owned(),
            capabilities: vec![
                AutomationCapability::Recommendation,
                AutomationCapability::MetadataCleanup,
                AutomationCapability::Summary,
                AutomationCapability::TitleMatch,
            ],
        }
    }

    async fn run(
        &self,
        _request: taru_automation::AutomationRequest,
    ) -> Result<taru_automation::AutomationOutcome> {
        Err(TaruError::Provider {
            provider: "automation".to_owned(),
            message: "no concrete automation provider runner is configured".to_owned(),
        })
    }
}

fn resolve_webhook_secret(endpoint: &WebhookEndpointRecord) -> Result<Option<String>> {
    let Some(name) = endpoint.secret_env.as_deref() else {
        return Ok(None);
    };

    env::var(name).map(Some).map_err(|err| TaruError::InvalidInput {
        message: format!(
            "webhook endpoint {} references unavailable secret environment variable {name}: {err}",
            endpoint.id
        ),
    })
}

impl TaruApp {
    pub async fn new(config: TaruServerConfig) -> Result<Self> {
        let store = SqliteStore::connect(&config.database_url).await?;
        Self::new_with_store(config, store).await
    }

    pub async fn new_with_store(config: TaruServerConfig, store: SqliteStore) -> Result<Self> {
        store.migrate().await?;
        let recovered_sessions = store
            .fail_stale_transcode_sessions(
                TranscodeFailureCategory::Stale,
                "session was active during server startup".to_owned(),
            )
            .await?;
        if recovered_sessions > 0 {
            warn!(
                recovered_sessions,
                "marked stale transcode sessions failed during startup"
            );
        }
        if config.staging.cleanup_on_startup {
            let cleanup = cleanup_expired_staging_inputs(&store, current_time_ms()?).await?;
            if cleanup.deleted_records > 0 || cleanup.deleted_files > 0 {
                warn!(
                    deleted_records = cleanup.deleted_records,
                    deleted_files = cleanup.deleted_files,
                    "cleaned expired staged inputs during startup"
                );
            }
        }

        let app = Self {
            inner: Arc::new(TaruAppInner {
                scan_permits: Arc::new(Semaphore::new(config.scan_concurrency.max(1))),
                metadata_permits: Arc::new(Semaphore::new(config.metadata_concurrency.max(1))),
                nfo_permits: Arc::new(Semaphore::new(config.metadata_concurrency.max(1))),
                webhook_permits: Arc::new(Semaphore::new(config.webhook_concurrency.max(1))),
                storage_backends: StorageBackendRegistry::new(&config, store.clone()),
                remux: RemuxAppService::new(&config),
                hls: HlsAppService::new(&config),
                config,
                store,
            }),
        };

        app.ensure_configured_libraries().await?;
        Ok(app)
    }

    #[must_use]
    pub fn config(&self) -> &TaruServerConfig {
        &self.inner.config
    }

    pub async fn list_libraries(&self, page: PageRequest) -> Result<LibraryListResponse> {
        let page = page.clamped();
        let libraries = self.inner.store.list_libraries(page).await?;

        Ok(LibraryListResponse {
            page: PageInfo::new(page, libraries.len()),
            libraries,
        })
    }

    pub async fn list_library_sources(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<LibrarySourcesResponse> {
        let page = page.clamped();
        let library = self.get_library_or_not_found(library_id).await?;
        let sources = self
            .inner
            .store
            .list_media_sources(library.id, page)
            .await?;
        let mut output_sources = Vec::with_capacity(sources.len());

        for source in sources {
            let item = self.inner.store.get_media_item(source.item_id).await?;
            let probe = self.inner.store.get_media_probe(source.id).await?;
            output_sources.push(LibrarySourceResponse {
                source,
                item,
                probe,
            });
        }

        Ok(LibrarySourcesResponse {
            library,
            page: PageInfo::new(page, output_sources.len()),
            sources: output_sources,
        })
    }

    pub async fn list_items(&self, page: PageRequest) -> Result<ItemsResponse> {
        let page = page.clamped();
        let items = self.inner.store.list_media_items(page).await?;

        Ok(ItemsResponse {
            page: PageInfo::new(page, items.len()),
            items,
        })
    }

    pub async fn get_item(&self, item_id: MediaItemId) -> Result<ItemDetailResponse> {
        let item = self
            .inner
            .store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let sources = self
            .inner
            .store
            .list_item_sources(item.id, PageRequest::first_page())
            .await?;
        let credits = self.inner.store.list_item_credits(item.id).await?;
        let genres = self.inner.store.list_item_genres(item.id).await?;
        let tags = self.inner.store.list_item_tags(item.id).await?;
        let collections = self.inner.store.list_item_collections(item.id).await?;
        let studios = self.inner.store.list_item_studios(item.id).await?;
        let images = self.inner.store.list_item_images(item.id).await?;

        Ok(ItemDetailResponse {
            item,
            sources,
            credits,
            genres,
            tags,
            collections,
            studios,
            images,
        })
    }

    pub async fn list_item_credits(&self, item_id: MediaItemId) -> Result<ItemCreditsResponse> {
        let item = self
            .inner
            .store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let credits = self.inner.store.list_item_credits(item.id).await?;
        let mut people = Vec::with_capacity(credits.len());

        for credit in &credits {
            if let Some(person) = self.inner.store.get_person(credit.person_id).await? {
                people.push(person);
            }
        }

        Ok(ItemCreditsResponse {
            item_id: item.id,
            credits,
            people,
        })
    }

    pub async fn list_item_images(&self, item_id: MediaItemId) -> Result<ImagesResponse> {
        self.inner
            .store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let images = self.inner.store.list_item_images(item_id).await?;

        Ok(ImagesResponse { item_id, images })
    }

    async fn storage_backend_for_library_root(
        &self,
        library: &taru_core::Library,
    ) -> Result<Arc<LibraryStorageBackend>> {
        self.inner
            .storage_backends
            .backend_for_library_root(library)
            .await
    }

    async fn storage_backend_for_media_source(
        &self,
        source: &MediaSource,
    ) -> Result<(StorageUri, Arc<LibraryStorageBackend>)> {
        self.inner
            .storage_backends
            .backend_for_media_source(source)
            .await
    }

    pub async fn list_people(&self, page: PageRequest) -> Result<PeopleResponse> {
        let page = page.clamped();
        let people = self.inner.store.list_people(page).await?;

        Ok(PeopleResponse {
            page: PageInfo::new(page, people.len()),
            people,
        })
    }

    pub async fn get_person(&self, person_id: PersonId) -> Result<taru_core::Person> {
        self.inner
            .store
            .get_person(person_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "person",
                id: person_id.to_string(),
            })
    }

    pub async fn list_person_items(
        &self,
        person_id: PersonId,
        page: PageRequest,
    ) -> Result<PersonItemsResponse> {
        let page = page.clamped();
        let person = self.get_person(person_id).await?;
        let items = self.inner.store.list_person_items(person.id, page).await?;

        Ok(PersonItemsResponse {
            person,
            page: PageInfo::new(page, items.len()),
            items,
        })
    }

    pub async fn list_tags(&self, page: PageRequest) -> Result<TagsResponse> {
        let page = page.clamped();
        let tags = self.inner.store.list_tags(page).await?;

        Ok(TagsResponse {
            page: PageInfo::new(page, tags.len()),
            tags,
        })
    }

    pub async fn list_tag_items(
        &self,
        tag_id: TagId,
        page: PageRequest,
    ) -> Result<TagItemsResponse> {
        let page = page.clamped();
        let tag = self
            .inner
            .store
            .get_tag(tag_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "tag",
                id: tag_id.to_string(),
            })?;
        let items = self.inner.store.list_tag_items(tag.id, page).await?;

        Ok(TagItemsResponse {
            tag,
            page: PageInfo::new(page, items.len()),
            items,
        })
    }

    pub async fn list_genres(&self, page: PageRequest) -> Result<GenreListResponse> {
        let page = page.clamped();
        let genres = self.inner.store.list_genres(page).await?;

        Ok(GenreListResponse {
            page: PageInfo::new(page, genres.len()),
            genres,
        })
    }

    pub async fn list_genre_items(
        &self,
        genre_id: GenreId,
        page: PageRequest,
    ) -> Result<GenreItemsResponse> {
        let page = page.clamped();
        let genre =
            self.inner
                .store
                .get_genre(genre_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "genre",
                    id: genre_id.to_string(),
                })?;
        let items = self.inner.store.list_genre_items(genre.id, page).await?;

        Ok(GenreItemsResponse {
            genre,
            page: PageInfo::new(page, items.len()),
            items,
        })
    }

    pub async fn search_items(
        &self,
        query: String,
        facets: Vec<String>,
        page: PageRequest,
    ) -> Result<SearchResponse> {
        let page = page.clamped();
        let hits = self
            .inner
            .store
            .search(SearchQuery {
                query,
                facets,
                limit: page.limit,
                offset: u32::try_from(page.offset).map_err(|err| TaruError::InvalidInput {
                    message: format!("search offset is too large: {err}"),
                })?,
            })
            .await?;
        let mut output_hits = Vec::with_capacity(hits.len());

        for hit in hits {
            let item = self
                .inner
                .store
                .get_media_item(hit.item_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "media_item",
                    id: hit.item_id.to_string(),
                })?;
            output_hits.push(SearchItemHit {
                item,
                score: hit.score,
            });
        }

        Ok(SearchResponse {
            page: PageInfo::new(page, output_hits.len()),
            hits: output_hits,
        })
    }

    pub async fn get_source_probe(
        &self,
        source_id: MediaSourceId,
    ) -> Result<taru_api::SourceProbeResponse> {
        let probe = self
            .inner
            .store
            .get_media_probe(source_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_source_probe",
                id: source_id.to_string(),
            })?;

        Ok(taru_api::SourceProbeResponse { source_id, probe })
    }

    async fn get_source_or_not_found(&self, source_id: MediaSourceId) -> Result<MediaSource> {
        self.inner
            .store
            .get_media_source(source_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_source",
                id: source_id.to_string(),
            })
    }

    async fn get_outbox_event_or_not_found(&self, event_id: EventId) -> Result<OutboxEventRecord> {
        self.inner
            .store
            .get_outbox_event(event_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "outbox_event",
                id: event_id.to_string(),
            })
    }

    fn normalize_webhook_endpoint(
        &self,
        request: UpsertWebhookEndpointRequest,
    ) -> Result<NewWebhookEndpoint> {
        let name = request.name.trim().to_owned();
        if name.is_empty() {
            return Err(TaruError::InvalidInput {
                message: "webhook endpoint name cannot be empty".to_owned(),
            });
        }

        let url = request.url.trim().to_owned();
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            return Err(TaruError::InvalidInput {
                message: "webhook endpoint URL must use http or https".to_owned(),
            });
        }

        let mut seen = HashSet::new();
        let mut subscribed_event_kinds = Vec::new();
        for value in request.subscribed_event_kinds {
            let value = value.trim().to_owned();
            if value.is_empty() || !seen.insert(value.clone()) {
                continue;
            }
            if value != "*" && DomainEventKind::parse(&value).is_err() {
                return Err(TaruError::InvalidInput {
                    message: format!("unsupported webhook event kind: {value}"),
                });
            }
            subscribed_event_kinds.push(value);
        }
        if subscribed_event_kinds.is_empty() {
            return Err(TaruError::InvalidInput {
                message: "webhook endpoint must subscribe to at least one event kind".to_owned(),
            });
        }

        let timeout_ms = request.timeout_ms.unwrap_or(10_000);
        if !(100..=60_000).contains(&timeout_ms) {
            return Err(TaruError::InvalidInput {
                message: "webhook timeout_ms must be between 100 and 60000".to_owned(),
            });
        }

        let max_attempts = request.max_attempts.unwrap_or(3);
        if !(1..=10).contains(&max_attempts) {
            return Err(TaruError::InvalidInput {
                message: "webhook max_attempts must be between 1 and 10".to_owned(),
            });
        }

        let secret_env = request.secret_env.and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        });

        Ok(NewWebhookEndpoint {
            id: request.id.unwrap_or_else(WebhookEndpointId::new),
            name,
            url,
            secret_env,
            subscribed_event_kinds,
            timeout_ms,
            max_attempts,
            status: request.status,
        })
    }

    fn normalize_automation_provider(
        &self,
        request: UpsertAutomationProviderRequest,
    ) -> Result<NewAutomationProviderConfig> {
        let name = request.name.trim().to_owned();
        if name.is_empty() {
            return Err(TaruError::InvalidInput {
                message: "automation provider name cannot be empty".to_owned(),
            });
        }

        let base_url = request.base_url.trim().to_owned();
        if !(base_url.starts_with("https://") || base_url.starts_with("http://")) {
            return Err(TaruError::InvalidInput {
                message: "automation provider base_url must use http or https".to_owned(),
            });
        }

        let mut seen = HashSet::new();
        let capabilities = request
            .capabilities
            .into_iter()
            .filter(|capability| seen.insert(*capability))
            .collect::<Vec<_>>();
        if capabilities.is_empty() {
            return Err(TaruError::InvalidInput {
                message: "automation provider must declare at least one capability".to_owned(),
            });
        }

        let timeout_ms = request.timeout_ms.unwrap_or(30_000);
        if !(100..=120_000).contains(&timeout_ms) {
            return Err(TaruError::InvalidInput {
                message: "automation provider timeout_ms must be between 100 and 120000".to_owned(),
            });
        }

        let max_attempts = request.max_attempts.unwrap_or(2);
        if !(1..=5).contains(&max_attempts) {
            return Err(TaruError::InvalidInput {
                message: "automation provider max_attempts must be between 1 and 5".to_owned(),
            });
        }

        let secret_env = request.secret_env.and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        });

        Ok(NewAutomationProviderConfig {
            id: request.id.unwrap_or_else(AutomationProviderId::new),
            name,
            base_url,
            secret_env,
            capabilities,
            timeout_ms,
            max_attempts,
            status: request.status,
        })
    }

    fn normalize_addon_registration(
        &self,
        request: RegisterAddonRequest,
    ) -> Result<NewAddonRegistration> {
        validate_manifest(&request.manifest).map_err(|err| TaruError::InvalidInput {
            message: err.to_string(),
        })?;

        let mut seen = HashSet::new();
        let granted_scopes = request
            .granted_scopes
            .into_iter()
            .filter(|scope| seen.insert(*scope))
            .collect::<Vec<_>>();

        for resource in &request.manifest.resources {
            ensure_scope_grant(&request.manifest, resource.kind, &granted_scopes).map_err(
                |err| TaruError::InvalidInput {
                    message: err.to_string(),
                },
            )?;
        }

        let manifest_json =
            serde_json::to_string(&request.manifest).map_err(|err| TaruError::InvalidInput {
                message: format!("failed to serialize addon manifest: {err}"),
            })?;
        let granted_scopes = granted_scopes
            .into_iter()
            .map(|scope| scope.as_str().to_owned())
            .collect();

        Ok(NewAddonRegistration {
            id: request.id.unwrap_or_else(AddonId::new),
            manifest_id: request.manifest.id,
            name: request.manifest.name,
            version: request.manifest.version,
            protocol_version: request.manifest.protocol_version,
            base_url: request.manifest.base_url,
            manifest_json,
            granted_scopes,
            status: request.status.unwrap_or(AddonStatus::Disabled),
        })
    }

    pub async fn upsert_webhook_endpoint(
        &self,
        request: UpsertWebhookEndpointRequest,
    ) -> Result<WebhookEndpointResponse> {
        let endpoint = self.normalize_webhook_endpoint(request)?;
        let endpoint = self.inner.store.upsert_webhook_endpoint(endpoint).await?;

        Ok(WebhookEndpointResponse { endpoint })
    }

    pub async fn get_webhook_endpoint(
        &self,
        endpoint_id: WebhookEndpointId,
    ) -> Result<WebhookEndpointResponse> {
        let endpoint = self
            .inner
            .store
            .get_webhook_endpoint(endpoint_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "webhook_endpoint",
                id: endpoint_id.to_string(),
            })?;

        Ok(WebhookEndpointResponse { endpoint })
    }

    pub async fn list_enabled_webhook_endpoints(&self) -> Result<WebhookEndpointsResponse> {
        let endpoints = self.inner.store.list_enabled_webhook_endpoints().await?;

        Ok(WebhookEndpointsResponse { endpoints })
    }

    pub async fn list_webhook_delivery_attempts(
        &self,
        event_id: EventId,
    ) -> Result<WebhookDeliveryAttemptsResponse> {
        self.get_outbox_event_or_not_found(event_id).await?;
        let attempts = self
            .inner
            .store
            .list_webhook_delivery_attempts(event_id)
            .await?;

        Ok(WebhookDeliveryAttemptsResponse { event_id, attempts })
    }

    pub async fn deliver_webhooks_for_event(
        &self,
        event_id: EventId,
    ) -> Result<WebhookDispatchResponse> {
        let event = self.get_outbox_event_or_not_found(event_id).await?;
        let endpoints = self.inner.store.list_enabled_webhook_endpoints().await?;
        let service = WebhookDeliveryService::new(ReqwestWebhookTransport::default());
        let mut workers = tokio::task::JoinSet::new();
        let mut attempted_endpoints = 0_u32;
        let mut delivered = 0_u32;
        let mut failed = 0_u32;
        let mut skipped_endpoints = 0_u32;
        let mut attempts = Vec::new();
        let mut errors = Vec::new();

        for endpoint in endpoints {
            if !endpoint_subscribes_to(&endpoint, event.kind) {
                skipped_endpoints += 1;
                continue;
            }

            let permit = self
                .inner
                .webhook_permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|err| TaruError::Provider {
                    provider: "webhook".to_owned(),
                    message: format!("webhook resource budget was closed: {err}"),
                })?;
            attempted_endpoints += 1;
            let endpoint_id = endpoint.id;
            let event = event.clone();
            let service = service.clone();
            let store = self.inner.store.clone();

            workers.spawn(async move {
                let _permit = permit;
                let secret = resolve_webhook_secret(&endpoint).map_err(|err| (endpoint_id, err))?;
                service
                    .deliver_once(&store, &event, &endpoint, secret.as_deref())
                    .await
                    .map(|attempt| (endpoint_id, attempt))
                    .map_err(|err| (endpoint_id, err))
            });
        }

        while let Some(result) = workers.join_next().await {
            match result {
                Ok(Ok((_, attempt))) => {
                    match attempt.status {
                        WebhookDeliveryStatus::Succeeded => delivered += 1,
                        WebhookDeliveryStatus::Failed => failed += 1,
                        WebhookDeliveryStatus::Pending | WebhookDeliveryStatus::Running => {}
                    }
                    attempts.push(attempt);
                }
                Ok(Err((endpoint_id, err))) => {
                    failed += 1;
                    warn!(
                        endpoint_id = %endpoint_id,
                        event_id = %event.id,
                        error = %err,
                        "webhook delivery failed before attempt completion"
                    );
                    errors.push(format!("endpoint {endpoint_id}: {err}"));
                }
                Err(err) => {
                    failed += 1;
                    warn!(
                        event_id = %event.id,
                        error = %err,
                        "webhook delivery worker join failed"
                    );
                    errors.push(format!("webhook delivery worker join failed: {err}"));
                }
            }
        }
        attempts.sort_by_key(|attempt| (attempt.endpoint_id, attempt.attempt_number));

        Ok(WebhookDispatchResponse {
            event,
            attempted_endpoints,
            delivered,
            failed,
            skipped_endpoints,
            attempts,
            errors,
        })
    }

    pub async fn upsert_automation_provider(
        &self,
        request: UpsertAutomationProviderRequest,
    ) -> Result<AutomationProviderResponse> {
        let provider = self.normalize_automation_provider(request)?;
        let provider = self
            .inner
            .store
            .upsert_automation_provider(provider)
            .await?;

        Ok(AutomationProviderResponse { provider })
    }

    pub async fn get_automation_provider(
        &self,
        provider_id: AutomationProviderId,
    ) -> Result<AutomationProviderResponse> {
        let provider = self
            .inner
            .store
            .get_automation_provider(provider_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "automation_provider",
                id: provider_id.to_string(),
            })?;

        Ok(AutomationProviderResponse { provider })
    }

    pub async fn list_enabled_automation_providers(&self) -> Result<AutomationProvidersResponse> {
        let providers = self.inner.store.list_enabled_automation_providers().await?;

        Ok(AutomationProvidersResponse { providers })
    }

    pub async fn register_addon(
        &self,
        request: RegisterAddonRequest,
    ) -> Result<AddonRegistrationResponse> {
        let addon = self.normalize_addon_registration(request)?;
        let addon = self.inner.store.upsert_addon_registration(addon).await?;

        Ok(AddonRegistrationResponse { addon })
    }

    pub async fn get_addon_registration(
        &self,
        addon_id: AddonId,
    ) -> Result<AddonRegistrationResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;

        Ok(AddonRegistrationResponse { addon })
    }

    pub async fn list_addon_registrations(
        &self,
        status: Option<AddonStatus>,
    ) -> Result<AddonRegistrationsResponse> {
        let addons = self.inner.store.list_addon_registrations(status).await?;

        Ok(AddonRegistrationsResponse { addons })
    }

    async fn get_addon_registration_or_not_found(
        &self,
        addon_id: AddonId,
    ) -> Result<AddonRegistrationRecord> {
        self.inner
            .store
            .get_addon_registration(addon_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "addon_registration",
                id: addon_id.to_string(),
            })
    }

    pub async fn enqueue_automation_job(
        &self,
        request: EnqueueAutomationJobRequest,
    ) -> Result<Job> {
        let input = request
            .into_job_input()
            .map_err(|err| TaruError::InvalidInput {
                message: format!("failed to serialize automation prompt: {err}"),
            })?;
        let service = AutomationJobService::new(UnavailableAutomationProvider);

        service.enqueue_job(&self.inner.store, input).await
    }

    pub async fn list_automation_artifacts_for_job(
        &self,
        job_id: JobId,
    ) -> Result<AutomationArtifactsResponse> {
        self.get_job(job_id).await?;
        let artifacts = self
            .inner
            .store
            .list_automation_artifacts_for_job(job_id)
            .await?;

        Ok(AutomationArtifactsResponse { artifacts })
    }

    pub async fn list_automation_artifacts_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<AutomationArtifactsResponse> {
        self.inner
            .store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let artifacts = self
            .inner
            .store
            .list_automation_artifacts_for_item(item_id, page)
            .await?;

        Ok(AutomationArtifactsResponse { artifacts })
    }

    async fn ensure_configured_libraries(&self) -> Result<()> {
        let libraries = libraries_from_config(self.config());
        if libraries.is_empty() {
            return Err(TaruError::InvalidInput {
                message: "server config must include at least one library".to_owned(),
            });
        }

        let mut seen = HashSet::new();
        for library in &libraries {
            if !seen.insert(library.id) {
                return Err(TaruError::InvalidInput {
                    message: format!("duplicate configured library id: {}", library.id),
                });
            }
        }

        for library in libraries {
            self.inner.store.upsert_library(&library).await?;
        }
        Ok(())
    }

    async fn record_outbox_event(&self, event: NewOutboxEvent) {
        let kind = event.kind.as_str();
        let idempotency_key = event.idempotency_key.clone();
        if let Err(err) = self.inner.store.enqueue_outbox_event(event).await {
            warn!(
                kind,
                idempotency_key,
                error = %err,
                "failed to persist outbox event"
            );
        }
    }

    async fn library_for_item(&self, item_id: MediaItemId) -> Result<Library> {
        for configured in libraries_from_config(self.config()) {
            let mut offset = 0;

            loop {
                let sources = self
                    .inner
                    .store
                    .list_media_sources(
                        configured.id,
                        PageRequest {
                            limit: PageRequest::MAX_LIMIT,
                            offset,
                        },
                    )
                    .await?;

                if sources.iter().any(|source| source.item_id == item_id) {
                    return Ok(configured);
                }

                if sources.len() < PageRequest::MAX_LIMIT as usize {
                    break;
                }

                offset += u64::from(PageRequest::MAX_LIMIT);
            }
        }

        default_library_from_config(self.config())
    }

    fn configured_library_for(&self, library_id: LibraryId) -> Result<Library> {
        libraries_from_config(self.config())
            .into_iter()
            .find(|library| library.id == library_id)
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
    }

    async fn get_library_or_not_found(&self, library_id: LibraryId) -> Result<Library> {
        self.inner
            .store
            .get_library(library_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
    }
}

fn current_time_ms() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| TaruError::InvalidInput {
            message: format!("system time is before UNIX epoch: {err}"),
        })?;

    i64::try_from(duration.as_millis()).map_err(|err| TaruError::InvalidInput {
        message: format!("current timestamp does not fit i64 milliseconds: {err}"),
    })
}

#[cfg(test)]
mod tests;
