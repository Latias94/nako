use std::{
    collections::HashSet,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use taru_core::{
    EventOutboxRepository, Library, LibraryId, LibraryRepository, MediaItemId, MediaRepository,
    MediaSource, MediaSourceId, NewOutboxEvent, PageRequest, Result, TaruError, TransactionManager,
    TranscodeFailureCategory, TranscodeSessionRepository,
};
use taru_db::SqliteStore;
use taru_metadata::MetadataProviderRegistry;
use taru_vfs::StorageUri;
use tokio::sync::Semaphore;
use tracing::warn;

use crate::config::{TaruServerConfig, default_library_from_config, libraries_from_config};

mod addons;
mod automation;
mod catalog;
mod jobs;
mod library;
mod metadata;
mod nfo;
pub(crate) mod playback;
mod staging;
mod storage;
mod webhooks;

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
    metadata_providers: MetadataProviderRegistry,
    remux: RemuxAppService,
    hls: HlsAppService,
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
                metadata_providers: metadata::build_metadata_provider_registry(&config),
                remux: RemuxAppService::new(&config),
                hls: HlsAppService::new(&config)?,
                config,
                store,
            }),
        };

        app.ensure_configured_libraries().await?;
        app.cleanup_metadata_raw_cache_on_startup().await?;
        app.start_metadata_lifecycle_tasks();
        Ok(app)
    }

    #[must_use]
    pub fn config(&self) -> &TaruServerConfig {
        &self.inner.config
    }

    pub async fn list_storage_backend_diagnostics(
        &self,
    ) -> taru_api::StorageBackendDiagnosticsResponse {
        self.inner.storage_backends.diagnostics().await
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
