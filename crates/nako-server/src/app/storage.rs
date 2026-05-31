use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicI64, AtomicU8, AtomicU64, Ordering},
    },
};

use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

use crate::config::{
    LocalLibraryConfig, NakoServerConfig, PlaybackConfig, WebDavLibraryConfig,
    configured_library_config_for,
};
use nako_api::admin::{
    StorageBackendDiagnostic, StorageBackendDiagnosticsResponse, StorageBackendHealthDiagnostic,
    StorageBackendKind, StorageBackendRegistryDiagnostic, StorageBackendRuntimeStateScope,
    StorageBackendStatus,
};
use nako_core::{
    Library, LibraryId, LibraryRepository, MediaSource, NakoError, PageRequest, Result,
    StagingManifestRecord, StagingManifestRepository, StagingPurpose, StagingState,
    StorageBackendHealthListFilter, StorageBackendHealthRecord, StorageBackendHealthRepository,
    StorageBackendHealthStatus, StorageCircuitBreakerState, StorageFailureClass,
    VfsCacheRepository, VfsCacheSummary,
};
use nako_db::NakoDatabase;
use nako_vfs::{LocalFsBackend, StorageBackend, StorageUri};

use super::current_time_ms;

#[derive(Clone, Debug)]
pub(crate) struct StorageDiagnosticsAppService {
    registry: StorageBackendRegistry,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct StagingCleanupPressureSummary {
    pub(crate) cleanup_candidate_records: usize,
    pub(crate) cleanup_candidate_bytes: u64,
}

impl StorageDiagnosticsAppService {
    pub(super) fn new(registry: StorageBackendRegistry) -> Self {
        Self { registry }
    }

    pub(crate) async fn list_storage_backend_diagnostics(
        &self,
    ) -> StorageBackendDiagnosticsResponse {
        self.registry.diagnostics().await
    }

    pub(crate) async fn list_storage_backend_health(
        &self,
        page: PageRequest,
    ) -> Result<Vec<StorageBackendHealthRecord>> {
        self.registry
            .store
            .list_storage_backend_health(StorageBackendHealthListFilter::default(), page)
            .await
    }

    pub(crate) async fn reset_storage_backend_health(
        &self,
        backend_key: &str,
        reset_at_ms: i64,
    ) -> Result<Option<StorageBackendHealthRecord>> {
        self.registry
            .store
            .clear_storage_backend_health(backend_key, reset_at_ms)
            .await
    }

    pub(crate) async fn list_staging_manifest_records(
        &self,
        purpose: Option<StagingPurpose>,
        state: Option<StagingState>,
        page: PageRequest,
    ) -> Result<Vec<StagingManifestRecord>> {
        self.registry
            .store
            .list_staging_manifest_records(purpose, state, page)
            .await
    }

    pub(crate) async fn sum_staging_manifest_bytes(&self) -> Result<u64> {
        self.registry.store.sum_staging_manifest_bytes().await
    }

    pub(crate) async fn process_cached_backend_count(&self) -> usize {
        self.registry.backends.lock().await.len()
    }

    pub(crate) async fn summarize_vfs_cache(&self, now_ms: i64) -> Result<VfsCacheSummary> {
        self.registry.store.summarize_vfs_cache(now_ms).await
    }

    pub(crate) async fn summarize_staging_cleanup_pressure(
        &self,
        now_ms: i64,
    ) -> Result<StagingCleanupPressureSummary> {
        let mut summary = StagingCleanupPressureSummary::default();
        let mut offset = 0;

        loop {
            let page = PageRequest::new(PageRequest::MAX_LIMIT, offset);
            let records = self
                .registry
                .store
                .list_staging_cleanup_candidates(now_ms, page)
                .await?;
            let returned = records.len();

            for record in &records {
                summary.cleanup_candidate_records =
                    summary.cleanup_candidate_records.saturating_add(1);
                summary.cleanup_candidate_bytes = summary
                    .cleanup_candidate_bytes
                    .saturating_add(record.size_bytes.unwrap_or(0));
            }

            if returned < PageRequest::MAX_LIMIT as usize {
                return Ok(summary);
            }

            offset =
                offset
                    .checked_add(returned as u64)
                    .ok_or_else(|| NakoError::InvalidInput {
                        message: "storage staging cleanup diagnostics pagination offset overflowed"
                            .to_owned(),
                    })?;
        }
    }

    #[cfg(test)]
    pub(super) async fn backend_for_library_root(
        &self,
        library: &Library,
    ) -> Result<Arc<LibraryStorageBackend>> {
        self.registry.backend_for_library_root(library).await
    }

    #[cfg(test)]
    pub(crate) async fn replace_backend_for_test(
        &self,
        config: LocalLibraryConfig,
        backend: Arc<dyn StorageBackend>,
    ) {
        self.registry
            .replace_backend_for_test(config, backend)
            .await;
    }
}

#[derive(Clone, Debug)]
pub(super) struct StorageBackendRegistry {
    config: NakoServerConfig,
    store: NakoDatabase,
    playback: PlaybackConfig,
    backends: Arc<Mutex<HashMap<LibraryId, Arc<LibraryStorageBackend>>>>,
}

impl StorageBackendRegistry {
    pub(super) fn new(config: &NakoServerConfig, store: NakoDatabase) -> Self {
        Self {
            config: config.clone(),
            store,
            playback: config.playback,
            backends: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(super) async fn backend_for_library_root(
        &self,
        library: &Library,
    ) -> Result<Arc<LibraryStorageBackend>> {
        let config = configured_library_config_for(&self.config, library.id)?;
        self.backend_for_library_config(config).await
    }

    pub(super) async fn backend_for_media_source(
        &self,
        source: &MediaSource,
    ) -> Result<(StorageUri, Arc<LibraryStorageBackend>)> {
        let uri = StorageUri::parse(&source.locator)?;
        let library_config = configured_library_config_for(&self.config, source.library_id)?;
        let backend = self.backend_for_library_config(library_config).await?;

        Ok((uri, backend))
    }

    async fn backend_for_library_config(
        &self,
        config: LocalLibraryConfig,
    ) -> Result<Arc<LibraryStorageBackend>> {
        let mut backends = self.backends.lock().await;
        if let Some(backend) = backends.get(&config.id) {
            return Ok(backend.clone());
        }

        let backend = Arc::new(LibraryStorageBackend::new(
            config.clone(),
            self.build_backend(&config)?,
            self.playback,
            self.store.clone(),
        ));
        backends.insert(config.id, backend.clone());

        Ok(backend)
    }

    pub(super) async fn diagnostics(&self) -> StorageBackendDiagnosticsResponse {
        let libraries = match self.list_all_libraries().await {
            Ok(libraries) => libraries,
            Err(err) => {
                return StorageBackendDiagnosticsResponse {
                    backends: vec![unavailable_registry_diagnostic(err)],
                };
            }
        };
        let mut backends = Vec::with_capacity(libraries.len());

        for library in libraries {
            let config = match configured_library_config_for(&self.config, library.id) {
                Ok(config) => config,
                Err(err) => {
                    let backend_kind = library_backend_kind(&library);
                    backends.push(unavailable_backend_diagnostic(
                        &library,
                        None,
                        Some(backend_kind),
                        safe_unavailable_reason(&err, backend_kind),
                    ));
                    continue;
                }
            };
            let root_uri = library
                .roots
                .first()
                .cloned()
                .unwrap_or_else(|| "local:///".to_owned());
            let backend_kind = backend_kind(&config);

            match self.backend_for_library_config(config).await {
                Ok(backend) => {
                    backends.push(backend.diagnostic(
                        library.id,
                        library.name,
                        root_uri,
                        backend_kind,
                    ));
                }
                Err(err) => {
                    let reason = safe_unavailable_reason(&err, backend_kind);
                    backends.push(unavailable_backend_diagnostic(
                        &library,
                        Some(root_uri),
                        Some(backend_kind),
                        reason,
                    ));
                }
            }
        }

        backends.sort_by(|left, right| left.library_name.cmp(&right.library_name));
        StorageBackendDiagnosticsResponse { backends }
    }

    async fn list_all_libraries(&self) -> Result<Vec<Library>> {
        let mut libraries = Vec::new();
        let mut offset = 0;

        loop {
            let page = PageRequest::new(PageRequest::MAX_LIMIT, offset);
            let mut batch = self.store.list_libraries(page).await?;
            let returned = batch.len();
            libraries.append(&mut batch);

            if returned < PageRequest::MAX_LIMIT as usize {
                return Ok(libraries);
            }

            offset =
                offset
                    .checked_add(returned as u64)
                    .ok_or_else(|| NakoError::InvalidInput {
                        message: "storage diagnostics library pagination offset overflowed"
                            .to_owned(),
                    })?;
        }
    }

    fn build_backend(&self, config: &LocalLibraryConfig) -> Result<Arc<dyn StorageBackend>> {
        match config.webdav.as_ref() {
            Some(webdav) => self.webdav_storage_backend(webdav),
            None => Ok(Arc::new(LocalFsBackend::new(&config.root)?)),
        }
    }

    fn webdav_storage_backend(
        &self,
        config: &WebDavLibraryConfig,
    ) -> Result<Arc<dyn StorageBackend>> {
        let backend = nako_vfs::WebDavBackend::new(webdav_backend_config(config))?;
        Ok(Arc::new(nako_vfs::CachedStorageBackend::new(
            backend,
            self.store.clone(),
        )))
    }

    #[cfg(test)]
    async fn replace_backend_for_test(
        &self,
        config: LocalLibraryConfig,
        backend: Arc<dyn StorageBackend>,
    ) {
        self.backends.lock().await.insert(
            config.id,
            Arc::new(LibraryStorageBackend::new(
                config,
                backend,
                self.playback,
                self.store.clone(),
            )),
        );
    }
}

pub(super) struct LibraryStorageBackend {
    library_id: LibraryId,
    backend_key: String,
    scheme: String,
    store: NakoDatabase,
    inner: Arc<dyn StorageBackend>,
    stream_permits: Arc<Semaphore>,
    stream_permits_max: usize,
    stage_permits: Arc<Semaphore>,
    stage_permits_max: usize,
    health: Arc<StorageBackendHealth>,
    health_update_lock: Arc<Mutex<()>>,
}

impl LibraryStorageBackend {
    fn new(
        config: LocalLibraryConfig,
        inner: Arc<dyn StorageBackend>,
        playback: PlaybackConfig,
        store: NakoDatabase,
    ) -> Self {
        let stream_permits_max = playback.remote_stream_concurrency.max(1);
        let stage_permits_max = playback.remote_stage_concurrency.max(1);
        let scheme = inner.scheme().to_owned();

        Self {
            library_id: config.id,
            backend_key: storage_backend_key(config.id, &scheme),
            scheme,
            store,
            inner,
            stream_permits: Arc::new(Semaphore::new(stream_permits_max)),
            stream_permits_max,
            stage_permits: Arc::new(Semaphore::new(stage_permits_max)),
            stage_permits_max,
            health: Arc::new(StorageBackendHealth::new()),
            health_update_lock: Arc::new(Mutex::new(())),
        }
    }

    #[must_use]
    pub(super) fn library_id(&self) -> LibraryId {
        self.library_id
    }

    #[must_use]
    pub(super) fn stage_permits(&self) -> Arc<Semaphore> {
        self.stage_permits.clone()
    }

    #[must_use]
    pub(super) fn clone_backend(&self) -> Arc<dyn StorageBackend> {
        self.inner.clone()
    }

    #[must_use]
    pub(super) fn health(&self) -> Arc<StorageBackendHealth> {
        self.health.clone()
    }

    pub(super) async fn acquire_stream_permit(&self) -> Result<OwnedSemaphorePermit> {
        self.stream_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| {
                NakoError::storage_resource_budget_closed(
                    format!("library:{}", self.library_id),
                    format!("remote stream resource budget was closed: {err}"),
                )
            })
    }

    #[cfg(test)]
    pub(super) fn available_stream_permits(&self) -> usize {
        self.stream_permits.available_permits()
    }

    fn diagnostic(
        &self,
        library_id: LibraryId,
        library_name: String,
        root_uri: String,
        backend_kind: StorageBackendKind,
    ) -> StorageBackendDiagnostic {
        let health = self.health.diagnostic();
        let status = if health.consecutive_errors == 0 {
            StorageBackendStatus::Ready
        } else {
            StorageBackendStatus::Degraded
        };

        StorageBackendDiagnostic {
            library_id,
            library_name,
            root_uri,
            backend_kind,
            scheme: self.scheme.clone(),
            status,
            reason: None,
            registry: StorageBackendRegistryDiagnostic {
                cached: true,
                stream_permits_available: self.stream_permits.available_permits(),
                stream_permits_max: self.stream_permits_max,
                stage_permits_available: self.stage_permits.available_permits(),
                stage_permits_max: self.stage_permits_max,
                state_scope: StorageBackendRuntimeStateScope::ProcessLocal,
            },
            health,
        }
    }

    async fn record_success(&self) {
        let _guard = self.health_update_lock.lock().await;
        let now_ms = current_time_ms().unwrap_or_default();
        self.health.record_success(now_ms);
        let record = self.health_record(
            StorageBackendHealthStatus::Healthy,
            StorageCircuitBreakerState::Closed,
            0,
            Some(now_ms),
            None,
            None,
            None,
            None,
            None,
            now_ms,
        );
        let _ = self.store.upsert_storage_backend_health(record).await;
    }

    async fn record_error(&self, err: &NakoError) {
        let _guard = self.health_update_lock.lock().await;
        let now_ms = current_time_ms().unwrap_or_default();
        let class = err
            .storage_failure_class()
            .unwrap_or(StorageFailureClass::Unknown);
        self.health.record_error(err, now_ms);
        let diagnostic = self.health.diagnostic();
        let circuit_breaker_state = if class.is_retryable() && diagnostic.backoff_until_ms.is_some()
        {
            StorageCircuitBreakerState::Open
        } else {
            StorageCircuitBreakerState::Closed
        };
        let record = self.health_record(
            StorageBackendHealthStatus::Unavailable,
            circuit_breaker_state,
            u64_to_u32_saturating(diagnostic.consecutive_errors),
            diagnostic.last_success_at_ms,
            Some(now_ms),
            Some(class),
            Some(class.safe_message().to_owned()),
            (circuit_breaker_state == StorageCircuitBreakerState::Open).then_some(now_ms),
            diagnostic.backoff_until_ms,
            now_ms,
        );
        let _ = self.store.upsert_storage_backend_health(record).await;
    }

    fn health_record(
        &self,
        status: StorageBackendHealthStatus,
        circuit_breaker_state: StorageCircuitBreakerState,
        consecutive_failures: u32,
        last_success_at_ms: Option<i64>,
        last_failure_at_ms: Option<i64>,
        last_failure_class: Option<StorageFailureClass>,
        last_failure_safe_message: Option<String>,
        circuit_opened_at_ms: Option<i64>,
        backoff_until_ms: Option<i64>,
        updated_at_ms: i64,
    ) -> StorageBackendHealthRecord {
        StorageBackendHealthRecord {
            backend_key: self.backend_key.clone(),
            library_id: Some(self.library_id),
            scheme: self.scheme.clone(),
            status,
            circuit_breaker_state,
            consecutive_failures,
            last_success_at_ms,
            last_failure_at_ms,
            last_failure_class,
            last_failure_safe_message,
            circuit_opened_at_ms,
            backoff_until_ms,
            updated_at_ms,
        }
    }
}

#[derive(Debug)]
pub(super) struct StorageBackendHealth {
    last_success_at_ms: AtomicI64,
    last_error_at_ms: AtomicI64,
    last_error_class: AtomicU8,
    consecutive_errors: AtomicU64,
    backoff_until_ms: AtomicI64,
}

impl StorageBackendHealth {
    fn new() -> Self {
        Self {
            last_success_at_ms: AtomicI64::new(0),
            last_error_at_ms: AtomicI64::new(0),
            last_error_class: AtomicU8::new(0),
            consecutive_errors: AtomicU64::new(0),
            backoff_until_ms: AtomicI64::new(0),
        }
    }

    fn record_success(&self, now_ms: i64) {
        self.last_success_at_ms.store(now_ms, Ordering::Relaxed);
        self.consecutive_errors.store(0, Ordering::Relaxed);
        self.backoff_until_ms.store(0, Ordering::Relaxed);
    }

    fn record_error(&self, err: &NakoError, now_ms: i64) {
        let class = err
            .storage_failure_class()
            .unwrap_or(StorageFailureClass::Unknown);
        self.last_error_at_ms.store(now_ms, Ordering::Relaxed);
        self.last_error_class
            .store(encode_storage_failure_class(class), Ordering::Relaxed);
        let consecutive_errors = self.consecutive_errors.fetch_add(1, Ordering::Relaxed) + 1;
        if class.is_retryable() {
            let backoff_until_ms = now_ms.saturating_add(storage_backoff_ms(consecutive_errors));
            self.backoff_until_ms
                .store(backoff_until_ms, Ordering::Relaxed);
        } else {
            self.backoff_until_ms.store(0, Ordering::Relaxed);
        }
    }

    #[cfg(test)]
    pub(super) fn consecutive_errors(&self) -> u64 {
        self.consecutive_errors.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(super) fn backoff_until_ms(&self) -> Option<i64> {
        timestamp_diagnostic(self.backoff_until_ms.load(Ordering::Relaxed))
    }

    fn backoff_error(&self, library_id: LibraryId) -> Option<NakoError> {
        let backoff_until_ms = self.backoff_until_ms.load(Ordering::Relaxed);
        if backoff_until_ms <= current_time_ms().unwrap_or_default() {
            return None;
        }

        Some(NakoError::storage_rate_limited(
            format!("library:{library_id}"),
            "storage backend is in process-local backoff",
        ))
    }

    fn diagnostic(&self) -> StorageBackendHealthDiagnostic {
        StorageBackendHealthDiagnostic {
            consecutive_errors: self.consecutive_errors.load(Ordering::Relaxed),
            last_success_at_ms: timestamp_diagnostic(
                self.last_success_at_ms.load(Ordering::Relaxed),
            ),
            last_error_at_ms: timestamp_diagnostic(self.last_error_at_ms.load(Ordering::Relaxed)),
            last_error_class: decode_storage_failure_class(
                self.last_error_class.load(Ordering::Relaxed),
            ),
            backoff_until_ms: timestamp_diagnostic(self.backoff_until_ms.load(Ordering::Relaxed)),
        }
    }
}

#[async_trait::async_trait]
impl StorageBackend for LibraryStorageBackend {
    fn scheme(&self) -> &'static str {
        self.inner.scheme()
    }

    async fn stat(&self, uri: &StorageUri) -> Result<nako_vfs::ObjectMetadata> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.stat(uri).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn list(&self, uri: &StorageUri) -> Result<Vec<nako_vfs::ObjectMetadata>> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.list(uri).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn list_with_status(&self, uri: &StorageUri) -> Result<nako_vfs::ObjectListing> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.list_with_status(uri).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn open_range(
        &self,
        uri: &StorageUri,
        range: Option<nako_vfs::ByteRange>,
    ) -> Result<nako_vfs::VirtualFile> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.open_range(uri, range).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn read_range(
        &self,
        uri: &StorageUri,
        range: Option<nako_vfs::ByteRange>,
    ) -> Result<nako_vfs::ReadRange> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.read_range(uri, range).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn stream_range(
        &self,
        uri: &StorageUri,
        range: Option<nako_vfs::ByteRange>,
    ) -> Result<nako_vfs::ReadStream> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.stream_range(uri, range).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.read_to_string(uri).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn write_string(&self, uri: &StorageUri, content: &str) -> Result<()> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.write_string(uri, content).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn write(
        &self,
        request: nako_vfs::StorageWriteRequest,
    ) -> Result<nako_vfs::StorageWriteReport> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.write(request).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn stage(&self, request: nako_vfs::StageRequest) -> Result<nako_vfs::StagedFile> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.stage(request).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn plan_link(
        &self,
        request: nako_vfs::StorageLinkPlanRequest,
    ) -> Result<nako_vfs::StorageLinkPlan> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.plan_link(request).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn apply(
        &self,
        request: nako_vfs::StorageApplyRequest,
    ) -> Result<nako_vfs::StorageApplyReport> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.apply(request).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn cleanup(
        &self,
        request: nako_vfs::StorageCleanupRequest,
    ) -> Result<nako_vfs::StorageCleanupReport> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.cleanup(request).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn restore(
        &self,
        request: nako_vfs::StorageRestoreRequest,
    ) -> Result<nako_vfs::StorageRestoreReport> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.restore(request).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }
}

impl LibraryStorageBackend {
    async fn record_result(&self, err: Option<NakoError>) {
        match err {
            None => self.record_success().await,
            Some(err) => self.record_error(&err).await,
        }
    }

    async fn reject_if_backing_off<T>(&self) -> Option<Result<T>> {
        match self.durable_backoff_error().await {
            Ok(Some(err)) => return Some(Err(err)),
            Ok(None) => {}
            Err(err) => return Some(Err(err)),
        }

        let err = self.health.backoff_error(self.library_id)?;
        Some(Err(err))
    }

    async fn durable_backoff_error(&self) -> Result<Option<NakoError>> {
        let Some(record) = self
            .store
            .get_storage_backend_health(&self.backend_key)
            .await?
        else {
            return Ok(None);
        };
        if record.circuit_breaker_state != StorageCircuitBreakerState::Open {
            return Ok(None);
        }
        let Some(backoff_until_ms) = record.backoff_until_ms else {
            return Ok(None);
        };
        if backoff_until_ms <= current_time_ms().unwrap_or_default() {
            return Ok(None);
        }

        Ok(Some(NakoError::storage_rate_limited(
            format!("library:{}", self.library_id),
            "storage circuit breaker is open",
        )))
    }
}

impl std::fmt::Debug for LibraryStorageBackend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LibraryStorageBackend")
            .field("library_id", &self.library_id)
            .field("scheme", &self.scheme)
            .field(
                "available_stream_permits",
                &self.stream_permits.available_permits(),
            )
            .field(
                "available_stage_permits",
                &self.stage_permits.available_permits(),
            )
            .field("stream_permits_max", &self.stream_permits_max)
            .field("stage_permits_max", &self.stage_permits_max)
            .field("health", &self.health)
            .finish()
    }
}

pub(super) fn webdav_backend_config(config: &WebDavLibraryConfig) -> nako_vfs::WebDavBackendConfig {
    nako_vfs::WebDavBackendConfig {
        base_url: config.base_url.clone(),
        username: config.username.clone(),
        password_env: config.password_env.clone(),
        timeout_ms: config.timeout_ms,
        max_attempts: config.max_attempts,
    }
}

fn backend_kind(config: &LocalLibraryConfig) -> StorageBackendKind {
    if config.webdav.is_some() {
        StorageBackendKind::WebDav
    } else {
        StorageBackendKind::Local
    }
}

fn unavailable_backend_diagnostic(
    library: &Library,
    root_uri: Option<String>,
    backend_kind: Option<StorageBackendKind>,
    reason: String,
) -> StorageBackendDiagnostic {
    StorageBackendDiagnostic {
        library_id: library.id,
        library_name: library.name.clone(),
        root_uri: root_uri.unwrap_or_else(|| {
            library
                .roots
                .first()
                .cloned()
                .unwrap_or_else(|| "local:///".to_owned())
        }),
        backend_kind: backend_kind.unwrap_or(StorageBackendKind::Local),
        scheme: library
            .roots
            .first()
            .and_then(|root| StorageUri::parse(root).ok())
            .map(|uri| uri.scheme().to_owned())
            .unwrap_or_else(|| "unknown".to_owned()),
        status: StorageBackendStatus::Unavailable,
        reason: Some(reason),
        registry: StorageBackendRegistryDiagnostic {
            cached: false,
            stream_permits_available: 0,
            stream_permits_max: 0,
            stage_permits_available: 0,
            stage_permits_max: 0,
            state_scope: StorageBackendRuntimeStateScope::ProcessLocal,
        },
        health: StorageBackendHealthDiagnostic {
            consecutive_errors: 0,
            last_success_at_ms: None,
            last_error_at_ms: None,
            last_error_class: None,
            backoff_until_ms: None,
        },
    }
}

fn library_backend_kind(library: &Library) -> StorageBackendKind {
    library
        .roots
        .first()
        .and_then(|root| StorageUri::parse(root).ok())
        .map(|uri| {
            if uri.scheme() == "webdav" {
                StorageBackendKind::WebDav
            } else {
                StorageBackendKind::Local
            }
        })
        .unwrap_or(StorageBackendKind::Local)
}

fn unavailable_registry_diagnostic(err: NakoError) -> StorageBackendDiagnostic {
    StorageBackendDiagnostic {
        library_id: LibraryId::new(),
        library_name: "Library registry".to_owned(),
        root_uri: "unknown:///".to_owned(),
        backend_kind: StorageBackendKind::Local,
        scheme: "unknown".to_owned(),
        status: StorageBackendStatus::Unavailable,
        reason: Some(safe_unavailable_reason(&err, StorageBackendKind::Local)),
        registry: StorageBackendRegistryDiagnostic {
            cached: false,
            stream_permits_available: 0,
            stream_permits_max: 0,
            stage_permits_available: 0,
            stage_permits_max: 0,
            state_scope: StorageBackendRuntimeStateScope::ProcessLocal,
        },
        health: StorageBackendHealthDiagnostic {
            consecutive_errors: 0,
            last_success_at_ms: None,
            last_error_at_ms: None,
            last_error_class: None,
            backoff_until_ms: None,
        },
    }
}

fn safe_unavailable_reason(err: &NakoError, backend_kind: StorageBackendKind) -> String {
    match err {
        NakoError::InvalidInput { .. } => match backend_kind {
            StorageBackendKind::Local => "local storage backend configuration is invalid",
            StorageBackendKind::WebDav => "WebDAV storage backend configuration is invalid",
        },
        NakoError::Storage { .. } => match backend_kind {
            StorageBackendKind::Local => "local storage backend could not be initialized",
            StorageBackendKind::WebDav => "WebDAV storage backend could not be initialized",
        },
        NakoError::NotFound { .. } => "configured library backend was not found",
        _ => "storage backend is unavailable",
    }
    .to_owned()
}

fn timestamp_diagnostic(value: i64) -> Option<i64> {
    (value > 0).then_some(value)
}

fn storage_backoff_ms(consecutive_errors: u64) -> i64 {
    const BASE_MS: i64 = 250;
    const MAX_MS: i64 = 30_000;

    let exponent = consecutive_errors.saturating_sub(1).min(7) as u32;
    BASE_MS.saturating_mul(2_i64.pow(exponent)).min(MAX_MS)
}

fn storage_backend_key(library_id: LibraryId, scheme: &str) -> String {
    format!("library:{library_id}:{scheme}")
}

fn u64_to_u32_saturating(value: u64) -> u32 {
    value.min(u64::from(u32::MAX)) as u32
}

fn encode_storage_failure_class(class: StorageFailureClass) -> u8 {
    match class {
        StorageFailureClass::Timeout => 1,
        StorageFailureClass::Unavailable => 2,
        StorageFailureClass::Permission => 3,
        StorageFailureClass::RateLimited => 4,
        StorageFailureClass::StaleCache => 5,
        StorageFailureClass::PartialRead => 6,
        StorageFailureClass::Budget => 7,
        StorageFailureClass::Security => 8,
        StorageFailureClass::Unknown => 9,
    }
}

fn decode_storage_failure_class(value: u8) -> Option<StorageFailureClass> {
    match value {
        1 => Some(StorageFailureClass::Timeout),
        2 => Some(StorageFailureClass::Unavailable),
        3 => Some(StorageFailureClass::Permission),
        4 => Some(StorageFailureClass::RateLimited),
        5 => Some(StorageFailureClass::StaleCache),
        6 => Some(StorageFailureClass::PartialRead),
        7 => Some(StorageFailureClass::Budget),
        8 => Some(StorageFailureClass::Security),
        9 => Some(StorageFailureClass::Unknown),
        _ => None,
    }
}

pub(super) fn remote_probe_staging_root(
    library: &Library,
    config: &NakoServerConfig,
) -> Option<PathBuf> {
    library
        .roots
        .iter()
        .any(|root| root.starts_with("webdav://"))
        .then(|| config.remux_staging_root.join("probe-inputs"))
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };

    use async_trait::async_trait;
    use nako_core::{
        DatabaseLifecycle, LibraryOptions, LibraryPreset, MediaItemId, MediaSourceId, NakoError,
        Result, StorageErrorKind, StorageFailureClass,
    };
    use nako_vfs::{
        ByteRange, ObjectMetadata, ReadRange, ReadStream, StageRequest, StagedFile, StorageBackend,
        StorageUri, VirtualFile,
    };
    use tempfile::tempdir;

    use super::*;
    use crate::config::{
        LocalLibraryConfig, NakoServerConfig, PlaybackConfig, StagingConfig,
        library_from_library_config,
    };

    #[tokio::test]
    async fn registry_reuses_library_backend_instances() {
        let temp = tempdir().unwrap();
        let library_config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: LibraryPreset::Movies,
            webdav: None,
        };
        let config = NakoServerConfig {
            database_backend: Default::default(),
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            database_url_env: None,
            auth: crate::config::AuthConfig::disabled(),
            network: crate::config::NetworkAccessConfig::default(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 1,
            addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
            remux_timeout_ms: 1,
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: Default::default(),
            transcode: Default::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            artwork: crate::config::ArtworkConfig::default(),
            libraries: vec![library_config.clone()],
        };
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        let registry = StorageBackendRegistry::new(&config, store);
        let library = library_from_library_config(&library_config);

        let first = registry.backend_for_library_root(&library).await.unwrap();
        let second = registry.backend_for_library_root(&library).await.unwrap();

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first.library_id(), library.id);
    }

    #[tokio::test]
    async fn registry_resolves_media_sources_by_library_id_only() {
        let temp = tempdir().unwrap();
        let library_config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: LibraryPreset::Movies,
            webdav: None,
        };
        let config = NakoServerConfig {
            database_backend: Default::default(),
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            database_url_env: None,
            auth: crate::config::AuthConfig::disabled(),
            network: crate::config::NetworkAccessConfig::default(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 1,
            addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
            remux_timeout_ms: 1,
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: Default::default(),
            transcode: Default::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            artwork: crate::config::ArtworkConfig::default(),
            libraries: vec![library_config],
        };
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        let registry = StorageBackendRegistry::new(&config, store);
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id: LibraryId::new(),
            item_id: MediaItemId::new(),
            locator: "local:///demo.mkv".to_owned(),
            file_name: "demo.mkv".to_owned(),
            size_bytes: None,
            fingerprint: None,
        };

        let err = registry
            .backend_for_media_source(&source)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            NakoError::NotFound {
                entity: "library",
                ..
            }
        ));
    }

    #[tokio::test]
    async fn library_backend_records_health_failures() {
        let temp = tempdir().unwrap();
        let store = migrated_store().await;
        let config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: LibraryPreset::Movies,
            webdav: None,
        };
        let backend = LibraryStorageBackend::new(
            config,
            Arc::new(FailingBackend),
            PlaybackConfig::default(),
            store,
        );
        let uri = StorageUri::parse("local:///demo.mkv").unwrap();

        assert!(backend.stat(&uri).await.is_err());
        assert!(backend.stat(&uri).await.is_err());
        assert_eq!(backend.health().consecutive_errors(), 2);
        assert!(backend.health().backoff_until_ms().is_none());
    }

    #[tokio::test]
    async fn library_backend_applies_process_local_backoff_after_retryable_storage_failure() {
        let temp = tempdir().unwrap();
        let store = migrated_store().await;
        let config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: LibraryPreset::Movies,
            webdav: None,
        };
        let failing = Arc::new(CountingFailingBackend::new(StorageErrorKind::Timeout));
        let backend =
            LibraryStorageBackend::new(config, failing.clone(), PlaybackConfig::default(), store);
        let uri = StorageUri::parse("local:///demo.mkv").unwrap();

        let first = backend.stat(&uri).await.unwrap_err();
        let second = backend.stat(&uri).await.unwrap_err();

        assert_eq!(
            first.storage_failure_class(),
            Some(StorageFailureClass::Timeout)
        );
        assert_eq!(
            second.storage_failure_class(),
            Some(StorageFailureClass::RateLimited)
        );
        assert_eq!(failing.stat_calls.load(Ordering::SeqCst), 1);
        assert_eq!(backend.health().consecutive_errors(), 1);
        assert!(backend.health().backoff_until_ms().is_some());
    }

    async fn migrated_store() -> NakoDatabase {
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store
    }

    #[tokio::test]
    async fn registry_diagnostics_redacts_unavailable_backend_details() {
        let temp = tempdir().unwrap();
        let missing_root = temp.path().join("missing-root");
        let library_config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: missing_root.clone(),
            preset: LibraryPreset::Movies,
            webdav: None,
        };
        let config = NakoServerConfig {
            database_backend: Default::default(),
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            database_url_env: None,
            auth: crate::config::AuthConfig::disabled(),
            network: crate::config::NetworkAccessConfig::default(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 1,
            addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
            remux_timeout_ms: 1,
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: Default::default(),
            transcode: Default::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            artwork: crate::config::ArtworkConfig::default(),
            libraries: vec![library_config.clone()],
        };
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store
            .upsert_library(&Library {
                id: library_config.id,
                name: library_config.name.clone(),
                roots: vec!["local:///".to_owned()],
                options: LibraryOptions::from_preset(library_config.preset),
            })
            .await
            .unwrap();
        let registry = StorageBackendRegistry::new(&config, store);

        let diagnostics = registry.diagnostics().await;

        assert_eq!(diagnostics.backends.len(), 1);
        let backend = &diagnostics.backends[0];
        assert_eq!(backend.status, StorageBackendStatus::Unavailable);
        assert_eq!(
            backend.reason.as_deref(),
            Some("local storage backend could not be initialized")
        );
        let serialized = serde_json::to_string(backend).unwrap();
        assert!(!serialized.contains(&missing_root.display().to_string()));
    }

    struct FailingBackend;

    #[async_trait]
    impl StorageBackend for FailingBackend {
        fn scheme(&self) -> &'static str {
            "local"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<VirtualFile> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn read_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<ReadRange> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn stream_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<ReadStream> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn write_string(&self, uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
            Err(NakoError::storage_unknown(
                request.uri.to_string(),
                "intentional failure",
            ))
        }
    }

    struct CountingFailingBackend {
        kind: StorageErrorKind,
        stat_calls: AtomicU64,
    }

    impl CountingFailingBackend {
        fn new(kind: StorageErrorKind) -> Self {
            Self {
                kind,
                stat_calls: AtomicU64::new(0),
            }
        }
    }

    #[async_trait]
    impl StorageBackend for CountingFailingBackend {
        fn scheme(&self) -> &'static str {
            "local"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            self.stat_calls.fetch_add(1, Ordering::SeqCst);
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<VirtualFile> {
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn read_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<ReadRange> {
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn stream_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<ReadStream> {
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn write_string(&self, uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
            Err(NakoError::storage(
                request.uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }
    }
}
