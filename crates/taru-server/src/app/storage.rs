use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicI64, AtomicU64, Ordering},
    },
};

use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

use crate::config::{
    LocalLibraryConfig, PlaybackConfig, TaruServerConfig, WebDavLibraryConfig,
    configured_library_config_for, libraries_from_config,
};
use taru_api::{
    StorageBackendDiagnostic, StorageBackendDiagnosticsResponse, StorageBackendHealthDiagnostic,
    StorageBackendKind, StorageBackendRegistryDiagnostic, StorageBackendRuntimeStateScope,
    StorageBackendStatus,
};
use taru_core::{Library, LibraryId, MediaSource, Result, TaruError};
use taru_db::SqliteStore;
use taru_vfs::{LocalFsBackend, StorageBackend, StorageUri};

use super::current_time_ms;

#[derive(Clone, Debug)]
pub(crate) struct StorageDiagnosticsAppService {
    registry: StorageBackendRegistry,
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

    #[cfg(test)]
    pub(super) async fn backend_for_library_root(
        &self,
        library: &Library,
    ) -> Result<Arc<LibraryStorageBackend>> {
        self.registry.backend_for_library_root(library).await
    }
}

#[derive(Clone, Debug)]
pub(super) struct StorageBackendRegistry {
    config: TaruServerConfig,
    store: SqliteStore,
    playback: PlaybackConfig,
    backends: Arc<Mutex<HashMap<LibraryId, Arc<LibraryStorageBackend>>>>,
}

impl StorageBackendRegistry {
    pub(super) fn new(config: &TaruServerConfig, store: SqliteStore) -> Self {
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
        ));
        backends.insert(config.id, backend.clone());

        Ok(backend)
    }

    pub(super) async fn diagnostics(&self) -> StorageBackendDiagnosticsResponse {
        let libraries = libraries_from_config(&self.config);
        let mut backends = Vec::with_capacity(libraries.len());

        for library in libraries {
            let config = match configured_library_config_for(&self.config, library.id) {
                Ok(config) => config,
                Err(_) => {
                    backends.push(unavailable_backend_diagnostic(
                        &library,
                        None,
                        None,
                        "configured library backend is unavailable".to_owned(),
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
        let backend = taru_vfs::WebDavBackend::new(webdav_backend_config(config))?;
        Ok(Arc::new(taru_vfs::CachedStorageBackend::new(
            backend,
            self.store.clone(),
        )))
    }
}

pub(super) struct LibraryStorageBackend {
    library_id: LibraryId,
    inner: Arc<dyn StorageBackend>,
    stream_permits: Arc<Semaphore>,
    stream_permits_max: usize,
    stage_permits: Arc<Semaphore>,
    stage_permits_max: usize,
    health: Arc<StorageBackendHealth>,
}

impl LibraryStorageBackend {
    fn new(
        config: LocalLibraryConfig,
        inner: Arc<dyn StorageBackend>,
        playback: PlaybackConfig,
    ) -> Self {
        let stream_permits_max = playback.remote_stream_concurrency.max(1);
        let stage_permits_max = playback.remote_stage_concurrency.max(1);

        Self {
            library_id: config.id,
            inner,
            stream_permits: Arc::new(Semaphore::new(stream_permits_max)),
            stream_permits_max,
            stage_permits: Arc::new(Semaphore::new(stage_permits_max)),
            stage_permits_max,
            health: Arc::new(StorageBackendHealth::new()),
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
    pub(super) fn health(&self) -> Arc<StorageBackendHealth> {
        self.health.clone()
    }

    pub(super) async fn acquire_stream_permit(&self) -> Result<OwnedSemaphorePermit> {
        self.stream_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| TaruError::Storage {
                uri: format!("library:{}", self.library_id),
                message: format!("remote stream resource budget was closed: {err}"),
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
            scheme: self.inner.scheme().to_owned(),
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

    fn record_success(&self) {
        self.health.record_success();
    }

    fn record_error(&self) {
        self.health.record_error();
    }
}

#[derive(Debug)]
pub(super) struct StorageBackendHealth {
    last_success_at_ms: AtomicI64,
    last_error_at_ms: AtomicI64,
    consecutive_errors: AtomicU64,
}

impl StorageBackendHealth {
    fn new() -> Self {
        Self {
            last_success_at_ms: AtomicI64::new(0),
            last_error_at_ms: AtomicI64::new(0),
            consecutive_errors: AtomicU64::new(0),
        }
    }

    fn record_success(&self) {
        let now_ms = current_time_ms().unwrap_or_default();
        self.last_success_at_ms.store(now_ms, Ordering::Relaxed);
        self.consecutive_errors.store(0, Ordering::Relaxed);
    }

    fn record_error(&self) {
        let now_ms = current_time_ms().unwrap_or_default();
        self.last_error_at_ms.store(now_ms, Ordering::Relaxed);
        self.consecutive_errors.fetch_add(1, Ordering::Relaxed);
    }

    #[cfg(test)]
    pub(super) fn consecutive_errors(&self) -> u64 {
        self.consecutive_errors.load(Ordering::Relaxed)
    }

    fn diagnostic(&self) -> StorageBackendHealthDiagnostic {
        StorageBackendHealthDiagnostic {
            consecutive_errors: self.consecutive_errors.load(Ordering::Relaxed),
            last_success_at_ms: timestamp_diagnostic(
                self.last_success_at_ms.load(Ordering::Relaxed),
            ),
            last_error_at_ms: timestamp_diagnostic(self.last_error_at_ms.load(Ordering::Relaxed)),
        }
    }
}

#[async_trait::async_trait]
impl StorageBackend for LibraryStorageBackend {
    fn scheme(&self) -> &'static str {
        self.inner.scheme()
    }

    async fn stat(&self, uri: &StorageUri) -> Result<taru_vfs::ObjectMetadata> {
        let result = self.inner.stat(uri).await;
        self.record_result(&result);
        result
    }

    async fn list(&self, uri: &StorageUri) -> Result<Vec<taru_vfs::ObjectMetadata>> {
        let result = self.inner.list(uri).await;
        self.record_result(&result);
        result
    }

    async fn list_with_status(&self, uri: &StorageUri) -> Result<taru_vfs::ObjectListing> {
        let result = self.inner.list_with_status(uri).await;
        self.record_result(&result);
        result
    }

    async fn open_range(
        &self,
        uri: &StorageUri,
        range: Option<taru_vfs::ByteRange>,
    ) -> Result<taru_vfs::VirtualFile> {
        let result = self.inner.open_range(uri, range).await;
        self.record_result(&result);
        result
    }

    async fn read_range(
        &self,
        uri: &StorageUri,
        range: Option<taru_vfs::ByteRange>,
    ) -> Result<taru_vfs::ReadRange> {
        let result = self.inner.read_range(uri, range).await;
        self.record_result(&result);
        result
    }

    async fn stream_range(
        &self,
        uri: &StorageUri,
        range: Option<taru_vfs::ByteRange>,
    ) -> Result<taru_vfs::ReadStream> {
        let result = self.inner.stream_range(uri, range).await;
        self.record_result(&result);
        result
    }

    async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
        let result = self.inner.read_to_string(uri).await;
        self.record_result(&result);
        result
    }

    async fn write_string(&self, uri: &StorageUri, content: &str) -> Result<()> {
        let result = self.inner.write_string(uri, content).await;
        self.record_result(&result);
        result
    }

    async fn stage(&self, request: taru_vfs::StageRequest) -> Result<taru_vfs::StagedFile> {
        let result = self.inner.stage(request).await;
        self.record_result(&result);
        result
    }
}

impl LibraryStorageBackend {
    fn record_result<T>(&self, result: &Result<T>) {
        match result {
            Ok(_) => self.record_success(),
            Err(_) => self.record_error(),
        }
    }
}

impl std::fmt::Debug for LibraryStorageBackend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LibraryStorageBackend")
            .field("library_id", &self.library_id)
            .field("scheme", &self.inner.scheme())
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

pub(super) fn webdav_backend_config(config: &WebDavLibraryConfig) -> taru_vfs::WebDavBackendConfig {
    taru_vfs::WebDavBackendConfig {
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
        },
    }
}

fn safe_unavailable_reason(err: &TaruError, backend_kind: StorageBackendKind) -> String {
    match err {
        TaruError::InvalidInput { .. } => match backend_kind {
            StorageBackendKind::Local => "local storage backend configuration is invalid",
            StorageBackendKind::WebDav => "WebDAV storage backend configuration is invalid",
        },
        TaruError::Storage { .. } => match backend_kind {
            StorageBackendKind::Local => "local storage backend could not be initialized",
            StorageBackendKind::WebDav => "WebDAV storage backend could not be initialized",
        },
        TaruError::NotFound { .. } => "configured library backend was not found",
        _ => "storage backend is unavailable",
    }
    .to_owned()
}

fn timestamp_diagnostic(value: i64) -> Option<i64> {
    (value > 0).then_some(value)
}

pub(super) fn remote_probe_staging_root(
    library: &Library,
    config: &TaruServerConfig,
) -> Option<PathBuf> {
    library
        .roots
        .iter()
        .any(|root| root.starts_with("webdav://"))
        .then(|| config.remux_staging_root.join("probe-inputs"))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use taru_core::{LibraryPreset, MediaItemId, MediaSourceId, Result, TaruError};
    use taru_vfs::{
        ByteRange, ObjectMetadata, ReadRange, ReadStream, StageRequest, StagedFile, StorageBackend,
        StorageUri, VirtualFile,
    };
    use tempfile::tempdir;

    use super::*;
    use crate::config::{
        LocalLibraryConfig, PlaybackConfig, StagingConfig, TaruServerConfig,
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
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 1,
            remux_timeout_ms: 1,
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: Default::default(),
            transcode: Default::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            libraries: vec![library_config.clone()],
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
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
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 1,
            remux_timeout_ms: 1,
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: Default::default(),
            transcode: Default::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            libraries: vec![library_config],
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
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
            TaruError::NotFound {
                entity: "library",
                ..
            }
        ));
    }

    #[tokio::test]
    async fn library_backend_records_health_failures() {
        let temp = tempdir().unwrap();
        let config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: LibraryPreset::Movies,
            webdav: None,
        };
        let backend =
            LibraryStorageBackend::new(config, Arc::new(FailingBackend), PlaybackConfig::default());
        let uri = StorageUri::parse("local:///demo.mkv").unwrap();

        assert!(backend.stat(&uri).await.is_err());
        assert!(backend.stat(&uri).await.is_err());
        assert_eq!(backend.health().consecutive_errors(), 2);
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
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 1,
            remux_timeout_ms: 1,
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: Default::default(),
            transcode: Default::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            libraries: vec![library_config],
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
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
            Err(TaruError::Storage {
                uri: uri.to_string(),
                message: "intentional failure".to_owned(),
            })
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Err(TaruError::Storage {
                uri: uri.to_string(),
                message: "intentional failure".to_owned(),
            })
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<VirtualFile> {
            Err(TaruError::Storage {
                uri: uri.to_string(),
                message: "intentional failure".to_owned(),
            })
        }

        async fn read_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<ReadRange> {
            Err(TaruError::Storage {
                uri: uri.to_string(),
                message: "intentional failure".to_owned(),
            })
        }

        async fn stream_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<ReadStream> {
            Err(TaruError::Storage {
                uri: uri.to_string(),
                message: "intentional failure".to_owned(),
            })
        }

        async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
            Err(TaruError::Storage {
                uri: uri.to_string(),
                message: "intentional failure".to_owned(),
            })
        }

        async fn write_string(&self, uri: &StorageUri, _content: &str) -> Result<()> {
            Err(TaruError::Storage {
                uri: uri.to_string(),
                message: "intentional failure".to_owned(),
            })
        }

        async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
            Err(TaruError::Storage {
                uri: request.uri.to_string(),
                message: "intentional failure".to_owned(),
            })
        }
    }
}
