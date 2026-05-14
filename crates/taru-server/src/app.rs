use std::{env, sync::Arc};

use serde::Serialize;
use taru_api::{
    ItemsResponse, LibraryListResponse, LibrarySourceResponse, LibrarySourcesResponse, PageInfo,
};
use taru_core::{
    ExternalProvider, Job, JobId, JobKind, JobRepository, Library, LibraryId, LibraryRepository,
    MediaItemId, MediaProbeRepository, MediaRepository, MediaSourceId, MetadataProfile, NewJob,
    PageRequest, Result, TaruError, TransactionManager,
};
use taru_db::SqliteStore;
use taru_library::{
    LibraryIndexRequest, LibraryIndexService, LibraryIndexSummary, LibraryProbeOptions,
    LibraryProbeRequest, LibraryProbeService, LibraryProbeSummary,
};
use taru_media_probe::FfprobeMediaProbe;
use taru_metadata::{
    MetadataProviderRegistry, MetadataRefreshJobInput, MetadataRefreshRequest,
    MetadataRefreshSummary, MetadataStrategyExecutor, TmdbMetadataProvider, TmdbProviderConfig,
};
use taru_nfo::{
    MovieNfoCodec, NfoExportRequest, NfoExportSummary, NfoImportRequest, NfoImportSummary,
    NfoJobInput, NfoService,
};
use taru_vfs::LocalFsBackend;
use tokio::sync::Semaphore;
use tracing::{Instrument, error, info, info_span, warn};

use crate::config::{TaruServerConfig, library_from_config};

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
}

#[derive(Clone, Debug, Serialize)]
pub struct ScanCommandOutput {
    pub job: Job,
    pub index: LibraryIndexSummary,
    pub probe: LibraryProbeSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct MetadataRefreshCommandOutput {
    pub job: Job,
    pub refresh: MetadataRefreshSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct NfoImportCommandOutput {
    pub job: Job,
    pub import: NfoImportSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct NfoExportCommandOutput {
    pub job: Job,
    pub export: NfoExportSummary,
}

impl TaruApp {
    pub async fn new(config: TaruServerConfig) -> Result<Self> {
        let store = SqliteStore::connect(&config.database_url).await?;
        Self::new_with_store(config, store).await
    }

    pub async fn new_with_store(config: TaruServerConfig, store: SqliteStore) -> Result<Self> {
        store.migrate().await?;

        let app = Self {
            inner: Arc::new(TaruAppInner {
                scan_permits: Arc::new(Semaphore::new(config.scan_concurrency.max(1))),
                metadata_permits: Arc::new(Semaphore::new(config.metadata_concurrency.max(1))),
                nfo_permits: Arc::new(Semaphore::new(config.metadata_concurrency.max(1))),
                config,
                store,
            }),
        };

        app.ensure_configured_library().await?;
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

    pub async fn get_job(&self, job_id: JobId) -> Result<Job> {
        self.inner
            .store
            .get_job(job_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "job",
                id: job_id.to_string(),
            })
    }

    pub async fn enqueue_library_scan(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_library_scan_job(library_id).await?;
        let job_id = job.id;
        let app = self.clone();

        tokio::spawn(
            async move {
                app.finish_library_scan_job(job_id, library_id).await;
            }
            .instrument(info_span!(
                "library_scan_background_job",
                job_id = %job_id,
                library_id = %library_id,
                resource_class = "disk.scan"
            )),
        );

        Ok(job)
    }

    pub async fn scan_configured_library(&self) -> Result<ScanCommandOutput> {
        let library_id = self.config().library.id;
        let job = self.create_library_scan_job(library_id).await?;
        self.execute_library_scan_job(job.id, library_id).await
    }

    pub async fn enqueue_metadata_refresh(&self, item_id: MediaItemId) -> Result<Job> {
        let job = self.create_metadata_refresh_job(item_id).await?;
        let job_id = job.id;
        let app = self.clone();

        tokio::spawn(
            async move {
                app.finish_metadata_refresh_job(job_id, item_id).await;
            }
            .instrument(info_span!(
                "metadata_refresh_background_job",
                job_id = %job_id,
                item_id = %item_id,
                resource_class = "metadata.tmdb"
            )),
        );

        Ok(job)
    }

    pub async fn refresh_item_metadata(
        &self,
        item_id: MediaItemId,
    ) -> Result<MetadataRefreshCommandOutput> {
        let job = self.create_metadata_refresh_job(item_id).await?;
        self.execute_metadata_refresh_job(job.id, item_id).await
    }

    pub async fn enqueue_nfo_import(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_nfo_import_job(library_id).await?;
        let job_id = job.id;
        let app = self.clone();

        tokio::spawn(
            async move {
                app.finish_nfo_import_job(job_id, library_id).await;
            }
            .instrument(info_span!(
                "nfo_import_background_job",
                job_id = %job_id,
                library_id = %library_id,
                resource_class = "metadata.nfo.import"
            )),
        );

        Ok(job)
    }

    pub async fn enqueue_nfo_export(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_nfo_export_job(library_id).await?;
        let job_id = job.id;
        let app = self.clone();

        tokio::spawn(
            async move {
                app.finish_nfo_export_job(job_id, library_id).await;
            }
            .instrument(info_span!(
                "nfo_export_background_job",
                job_id = %job_id,
                library_id = %library_id,
                resource_class = "metadata.nfo.export"
            )),
        );

        Ok(job)
    }

    pub async fn import_library_nfo(
        &self,
        library_id: LibraryId,
    ) -> Result<NfoImportCommandOutput> {
        let job = self.create_nfo_import_job(library_id).await?;
        self.execute_nfo_import_job(job.id, library_id).await
    }

    pub async fn export_library_nfo(
        &self,
        library_id: LibraryId,
    ) -> Result<NfoExportCommandOutput> {
        let job = self.create_nfo_export_job(library_id).await?;
        self.execute_nfo_export_job(job.id, library_id).await
    }

    async fn ensure_configured_library(&self) -> Result<()> {
        let library = library_from_config(self.config());
        self.inner.store.upsert_library(&library).await?;
        Ok(())
    }

    async fn create_library_scan_job(&self, library_id: LibraryId) -> Result<Job> {
        self.configured_library_for(library_id)?;
        let input = LibraryScanJobInput {
            library_id,
            force: false,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize job input: {err}"),
        })?;

        self.inner
            .store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::LibraryScan,
                resource_class: "disk.scan".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(input_json),
            })
            .await
    }

    async fn create_metadata_refresh_job(&self, item_id: MediaItemId) -> Result<Job> {
        let item = self
            .inner
            .store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let library = self.library_for_item(item_id).await?;
        let profile = self.effective_metadata_profile(&library, item.kind)?;
        let provider = self.first_metadata_provider(&profile)?;
        let input = MetadataRefreshJobInput {
            item_id,
            provider: Some(provider.clone()),
            force: false,
            language: profile.language.clone(),
            refresh_mode: profile.refresh_mode,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize metadata refresh job input: {err}"),
        })?;

        self.inner
            .store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::MetadataRefresh,
                resource_class: format!("metadata.{}", provider_resource_name(&provider)),
                library_id: Some(library.id),
                source_id: None,
                input_json: Some(input_json),
            })
            .await
    }

    async fn create_nfo_import_job(&self, library_id: LibraryId) -> Result<Job> {
        let library = self.configured_library_for(library_id)?;
        let input = NfoJobInput {
            library_id,
            policy: library.options.metadata_profile.local_metadata_policy,
            force: false,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize NFO import job input: {err}"),
        })?;

        self.inner
            .store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::NfoImport,
                resource_class: "metadata.nfo.import".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(input_json),
            })
            .await
    }

    async fn create_nfo_export_job(&self, library_id: LibraryId) -> Result<Job> {
        let library = self.configured_library_for(library_id)?;
        let input = NfoJobInput {
            library_id,
            policy: library.options.metadata_profile.local_metadata_policy,
            force: false,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize NFO export job input: {err}"),
        })?;

        self.inner
            .store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::NfoExport,
                resource_class: "metadata.nfo.export".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(input_json),
            })
            .await
    }

    async fn finish_library_scan_job(&self, job_id: JobId, library_id: LibraryId) {
        match self.execute_library_scan_job(job_id, library_id).await {
            Ok(output) => {
                info!(
                    job_id = %output.job.id,
                    library_id = %library_id,
                    status = ?output.job.status,
                    "library scan job completed"
                );
            }
            Err(err) => {
                error!(
                    job_id = %job_id,
                    library_id = %library_id,
                    error = %err,
                    "library scan job failed"
                );
            }
        }
    }

    async fn finish_metadata_refresh_job(&self, job_id: JobId, item_id: MediaItemId) {
        match self.execute_metadata_refresh_job(job_id, item_id).await {
            Ok(output) => {
                info!(
                    job_id = %output.job.id,
                    item_id = %item_id,
                    provider_key = %output.refresh.provider_key,
                    status = ?output.job.status,
                    "metadata refresh job completed"
                );
            }
            Err(err) => {
                error!(
                    job_id = %job_id,
                    item_id = %item_id,
                    error = %err,
                    "metadata refresh job failed"
                );
            }
        }
    }

    async fn finish_nfo_import_job(&self, job_id: JobId, library_id: LibraryId) {
        match self.execute_nfo_import_job(job_id, library_id).await {
            Ok(output) => {
                info!(
                    job_id = %output.job.id,
                    library_id = %library_id,
                    imported_items = output.import.imported_items,
                    status = ?output.job.status,
                    "NFO import job completed"
                );
            }
            Err(err) => {
                error!(
                    job_id = %job_id,
                    library_id = %library_id,
                    error = %err,
                    "NFO import job failed"
                );
            }
        }
    }

    async fn finish_nfo_export_job(&self, job_id: JobId, library_id: LibraryId) {
        match self.execute_nfo_export_job(job_id, library_id).await {
            Ok(output) => {
                info!(
                    job_id = %output.job.id,
                    library_id = %library_id,
                    exported_items = output.export.exported_items,
                    status = ?output.job.status,
                    "NFO export job completed"
                );
            }
            Err(err) => {
                error!(
                    job_id = %job_id,
                    library_id = %library_id,
                    error = %err,
                    "NFO export job failed"
                );
            }
        }
    }

    async fn execute_library_scan_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<ScanCommandOutput> {
        let permit = self
            .inner
            .scan_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| TaruError::InvalidInput {
                message: format!("scan concurrency limiter is unavailable: {err}"),
            })?;
        let _permit = permit;

        self.inner.store.start_job(job_id).await?;

        match self.run_library_scan(job_id, library_id).await {
            Ok((index, probe)) => {
                let output = ScanJobSummary {
                    index: index.clone(),
                    probe: probe.clone(),
                };
                let summary_json =
                    serde_json::to_string(&output).map_err(|err| TaruError::InvalidInput {
                        message: format!("failed to serialize job summary: {err}"),
                    })?;
                let job = self
                    .inner
                    .store
                    .succeed_job(job_id, Some(summary_json))
                    .await?;

                Ok(ScanCommandOutput { job, index, probe })
            }
            Err(err) => {
                if let Err(update_err) = self.inner.store.fail_job(job_id, err.to_string()).await {
                    warn!(
                        job_id = %job_id,
                        library_id = %library_id,
                        error = %update_err,
                        "failed to persist failed job state"
                    );
                }

                Err(err)
            }
        }
    }

    async fn execute_metadata_refresh_job(
        &self,
        job_id: JobId,
        item_id: MediaItemId,
    ) -> Result<MetadataRefreshCommandOutput> {
        let permit = self
            .inner
            .metadata_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| TaruError::InvalidInput {
                message: format!("metadata concurrency limiter is unavailable: {err}"),
            })?;
        let _permit = permit;

        self.inner.store.start_job(job_id).await?;

        match self.run_metadata_refresh(job_id, item_id).await {
            Ok(refresh) => {
                let summary_json =
                    serde_json::to_string(&refresh).map_err(|err| TaruError::InvalidInput {
                        message: format!("failed to serialize metadata refresh job summary: {err}"),
                    })?;
                let job = self
                    .inner
                    .store
                    .succeed_job(job_id, Some(summary_json))
                    .await?;

                Ok(MetadataRefreshCommandOutput { job, refresh })
            }
            Err(err) => {
                if let Err(update_err) = self.inner.store.fail_job(job_id, err.to_string()).await {
                    warn!(
                        job_id = %job_id,
                        item_id = %item_id,
                        error = %update_err,
                        "failed to persist failed metadata refresh job state"
                    );
                }

                Err(err)
            }
        }
    }

    async fn execute_nfo_import_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoImportCommandOutput> {
        let permit = self
            .inner
            .nfo_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| TaruError::InvalidInput {
                message: format!("NFO concurrency limiter is unavailable: {err}"),
            })?;
        let _permit = permit;

        self.inner.store.start_job(job_id).await?;

        match self.run_nfo_import(job_id, library_id).await {
            Ok(import) => {
                let summary_json =
                    serde_json::to_string(&import).map_err(|err| TaruError::InvalidInput {
                        message: format!("failed to serialize NFO import job summary: {err}"),
                    })?;
                let job = self
                    .inner
                    .store
                    .succeed_job(job_id, Some(summary_json))
                    .await?;

                Ok(NfoImportCommandOutput { job, import })
            }
            Err(err) => {
                if let Err(update_err) = self.inner.store.fail_job(job_id, err.to_string()).await {
                    warn!(
                        job_id = %job_id,
                        library_id = %library_id,
                        error = %update_err,
                        "failed to persist failed NFO import job state"
                    );
                }

                Err(err)
            }
        }
    }

    async fn execute_nfo_export_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoExportCommandOutput> {
        let permit = self
            .inner
            .nfo_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| TaruError::InvalidInput {
                message: format!("NFO concurrency limiter is unavailable: {err}"),
            })?;
        let _permit = permit;

        self.inner.store.start_job(job_id).await?;

        match self.run_nfo_export(job_id, library_id).await {
            Ok(export) => {
                let summary_json =
                    serde_json::to_string(&export).map_err(|err| TaruError::InvalidInput {
                        message: format!("failed to serialize NFO export job summary: {err}"),
                    })?;
                let job = self
                    .inner
                    .store
                    .succeed_job(job_id, Some(summary_json))
                    .await?;

                Ok(NfoExportCommandOutput { job, export })
            }
            Err(err) => {
                if let Err(update_err) = self.inner.store.fail_job(job_id, err.to_string()).await {
                    warn!(
                        job_id = %job_id,
                        library_id = %library_id,
                        error = %update_err,
                        "failed to persist failed NFO export job state"
                    );
                }

                Err(err)
            }
        }
    }

    async fn run_library_scan(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<(LibraryIndexSummary, LibraryProbeSummary)> {
        let library = self.configured_library_for(library_id)?;
        info!(
            job_id = %job_id,
            library_id = %library_id,
            probe_concurrency = self.config().probe_concurrency.max(1),
            "starting library scan pipeline"
        );

        let index_backend = LocalFsBackend::new(&self.config().library.root)?;
        let scanner = taru_library::VfsLibraryScanner::new(index_backend);
        let index_service = LibraryIndexService::new(scanner, self.inner.store.clone());
        let index = index_service
            .index_library(LibraryIndexRequest {
                job_id,
                library,
                force: false,
            })
            .await?;

        let probe_backend = LocalFsBackend::new(&self.config().library.root)?;
        let probe = FfprobeMediaProbe::new(&self.config().ffprobe_path);
        let probe_service = LibraryProbeService::with_options(
            probe_backend,
            probe,
            self.inner.store.clone(),
            LibraryProbeOptions {
                max_concurrent_probes: self.config().probe_concurrency.max(1),
            },
        );
        let probe = probe_service
            .probe_library(LibraryProbeRequest {
                job_id,
                library_id,
                force: false,
            })
            .await?;

        Ok((index, probe))
    }

    async fn run_metadata_refresh(
        &self,
        job_id: JobId,
        item_id: MediaItemId,
    ) -> Result<MetadataRefreshSummary> {
        let item = self
            .inner
            .store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let library = self.library_for_item(item_id).await?;
        let profile = self.effective_metadata_profile(&library, item.kind)?;
        let registry = self.metadata_provider_registry();
        let executor = MetadataStrategyExecutor::new(registry, self.inner.store.clone());

        executor
            .refresh_item(MetadataRefreshRequest {
                job_id,
                item_id,
                profile,
                force: false,
            })
            .await
    }

    async fn run_nfo_import(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoImportSummary> {
        let library = self.configured_library_for(library_id)?;
        info!(
            job_id = %job_id,
            library_id = %library_id,
            policy = ?library.options.metadata_profile.local_metadata_policy,
            "starting NFO import job"
        );

        let backend = LocalFsBackend::new(&self.config().library.root)?;
        let service = NfoService::new(backend, self.inner.store.clone(), MovieNfoCodec);

        service
            .import_library(NfoImportRequest {
                job_id,
                library_id,
                policy: library.options.metadata_profile.local_metadata_policy,
                force: false,
            })
            .await
    }

    async fn run_nfo_export(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoExportSummary> {
        let library = self.configured_library_for(library_id)?;
        info!(
            job_id = %job_id,
            library_id = %library_id,
            policy = ?library.options.metadata_profile.local_metadata_policy,
            "starting NFO export job"
        );

        let backend = LocalFsBackend::new(&self.config().library.root)?;
        let service = NfoService::new(backend, self.inner.store.clone(), MovieNfoCodec);

        service
            .export_library(NfoExportRequest {
                job_id,
                library_id,
                policy: library.options.metadata_profile.local_metadata_policy,
                force: false,
            })
            .await
    }

    async fn library_for_item(&self, item_id: MediaItemId) -> Result<Library> {
        let configured = library_from_config(self.config());
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

        Ok(configured)
    }

    fn effective_metadata_profile(
        &self,
        library: &Library,
        item_kind: taru_core::MediaKind,
    ) -> Result<MetadataProfile> {
        let mut profile = library.options.metadata_profile.clone();

        if !profile.item_kinds.is_empty()
            && !profile.item_kinds.contains(&item_kind)
            && !profile.item_kinds.contains(&taru_core::MediaKind::Unknown)
        {
            return Err(TaruError::Unsupported(
                "library metadata profile does not apply to this item kind",
            ));
        }

        if profile.language.is_none() && !self.config().metadata.tmdb.language.trim().is_empty() {
            profile.language = Some(self.config().metadata.tmdb.language.clone());
        }

        Ok(profile)
    }

    fn first_metadata_provider(&self, profile: &MetadataProfile) -> Result<ExternalProvider> {
        let Some(provider) = profile.metadata_providers.first().cloned() else {
            return Err(TaruError::InvalidInput {
                message: "library metadata profile does not enable any metadata provider"
                    .to_owned(),
            });
        };

        Ok(provider)
    }

    fn metadata_provider_registry(&self) -> MetadataProviderRegistry {
        let mut registry = MetadataProviderRegistry::new();
        match self.tmdb_provider() {
            Ok(provider) => {
                registry.register(provider);
            }
            Err(TmdbProviderBuildError::Disabled(message)) => {
                registry.register_disabled(ExternalProvider::Tmdb, message);
            }
            Err(TmdbProviderBuildError::Unavailable(message)) => {
                registry.register_unavailable(ExternalProvider::Tmdb, message);
            }
        }

        registry
    }

    fn tmdb_provider(&self) -> std::result::Result<TmdbMetadataProvider, TmdbProviderBuildError> {
        let settings = &self.config().metadata.tmdb;

        if !settings.enabled {
            return Err(TmdbProviderBuildError::Disabled(
                "TMDB metadata provider is disabled in config".to_owned(),
            ));
        }

        let token = env::var(&settings.access_token_env).map_err(|err| {
            TmdbProviderBuildError::Unavailable(format!(
                "failed to read TMDB access token from environment variable {}: {err}",
                settings.access_token_env
            ))
        })?;

        if token.trim().is_empty() {
            return Err(TmdbProviderBuildError::Unavailable(format!(
                "TMDB access token environment variable {} is empty",
                settings.access_token_env
            )));
        }

        let mut config = TmdbProviderConfig::new(token);
        config.api_base_url = settings.api_base_url.clone();
        config.image_base_url = settings.image_base_url.clone();
        config.language = settings.language.clone();
        config.include_adult = settings.include_adult;

        Ok(TmdbMetadataProvider::new(config))
    }

    fn configured_library_for(&self, library_id: LibraryId) -> Result<Library> {
        let library = library_from_config(self.config());

        if library.id == library_id {
            Ok(library)
        } else {
            Err(TaruError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
        }
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

fn provider_resource_name(provider: &ExternalProvider) -> &str {
    match provider {
        ExternalProvider::Tmdb => "tmdb",
        ExternalProvider::Douban => "douban",
        ExternalProvider::Bangumi => "bangumi",
        ExternalProvider::Imdb => "imdb",
        ExternalProvider::Local => "local",
        ExternalProvider::Other(_) => "other",
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TmdbProviderBuildError {
    Disabled(String),
    Unavailable(String),
}

#[derive(Clone, Debug, Serialize)]
struct ScanJobSummary {
    index: LibraryIndexSummary,
    probe: LibraryProbeSummary,
}

#[derive(Clone, Debug, Serialize)]
struct LibraryScanJobInput {
    library_id: LibraryId,
    force: bool,
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use taru_core::{
        CanonicalMetadata, JobKind, JobStatus, LibraryId, MediaItem, MediaItemId, MediaKind,
        MediaRepository, MediaSource, MediaSourceId, MetadataField, MetadataRepository,
        MetadataSource,
    };

    use super::*;
    use crate::config::{LocalLibraryConfig, MetadataConfig};

    #[tokio::test]
    async fn scan_configured_library_persists_job_success() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            metadata: MetadataConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store).await.unwrap();

        let output = app.scan_configured_library().await.unwrap();
        let job = app.get_job(output.job.id).await.unwrap();

        assert_eq!(output.job.status, JobStatus::Succeeded);
        assert_eq!(job.status, JobStatus::Succeeded);
        assert_eq!(output.index.discovered_files, 0);
        assert_eq!(output.probe.total_sources, 0);
    }

    #[tokio::test]
    async fn metadata_refresh_job_input_does_not_include_secrets() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let mut metadata = MetadataConfig::default();
        metadata.tmdb.enabled = true;
        metadata.tmdb.access_token_env = "TARU_TEST_MISSING_TMDB_TOKEN".to_owned();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            metadata,
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_media_item(&item).await.unwrap();

        let job = app.create_metadata_refresh_job(item.id).await.unwrap();
        let input = job
            .input_json
            .as_ref()
            .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
            .unwrap();

        assert_eq!(job.kind, JobKind::MetadataRefresh);
        assert_eq!(job.resource_class, "metadata.tmdb");
        assert_eq!(job.library_id, Some(library_id));
        assert_eq!(
            input.get("item_id").and_then(serde_json::Value::as_str),
            Some(item.id.to_string().as_str())
        );
        assert_eq!(
            input.get("provider").and_then(serde_json::Value::as_str),
            Some("tmdb")
        );
        assert_eq!(
            input
                .get("refresh_mode")
                .and_then(serde_json::Value::as_str),
            Some("default")
        );
        assert!(input.get("access_token").is_none());
        assert!(input.get("api_key").is_none());
    }

    #[tokio::test]
    async fn metadata_refresh_job_records_disabled_profile_provider_for_executor() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            metadata: MetadataConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_media_item(&item).await.unwrap();

        let job = app.create_metadata_refresh_job(item.id).await.unwrap();
        let err = app.run_metadata_refresh(job.id, item.id).await.unwrap_err();

        assert_eq!(job.kind, JobKind::MetadataRefresh);
        assert_eq!(job.resource_class, "metadata.tmdb");
        let TaruError::Provider { provider, message } = err else {
            panic!("expected provider exhaustion error");
        };
        assert_eq!(provider, "metadata_strategy");
        assert!(message.contains("tmdb=skipped_disabled"));
        assert!(message.contains("disabled in config"));
    }

    #[tokio::test]
    async fn metadata_refresh_falls_back_from_unimplemented_bangumi_to_tmdb_unavailable() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let mut metadata = MetadataConfig::default();
        metadata.tmdb.enabled = true;
        metadata.tmdb.access_token_env = "TARU_TEST_MISSING_TMDB_TOKEN".to_owned();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            metadata,
            library: LocalLibraryConfig {
                id: library_id,
                name: "Anime".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Anime,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Anime Movie".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_media_item(&item).await.unwrap();
        let job = app.create_metadata_refresh_job(item.id).await.unwrap();
        let err = app.run_metadata_refresh(job.id, item.id).await.unwrap_err();

        assert_eq!(job.resource_class, "metadata.bangumi");
        let TaruError::Provider { provider, message } = err else {
            panic!("expected provider exhaustion error");
        };
        assert_eq!(provider, "metadata_strategy");
        assert!(message.contains("bangumi=not_implemented"));
        assert!(message.contains("tmdb=skipped_unavailable"));
        assert_eq!(app.get_job(job.id).await.unwrap().status, JobStatus::Queued);
    }

    #[tokio::test]
    async fn metadata_refresh_resolves_provider_order_from_library_profile() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let mut metadata = MetadataConfig::default();
        metadata.tmdb.enabled = true;
        metadata.tmdb.access_token_env = "TARU_TEST_MISSING_TMDB_TOKEN".to_owned();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            metadata,
            library: LocalLibraryConfig {
                id: library_id,
                name: "Anime".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Anime,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Anime Movie".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_media_item(&item).await.unwrap();
        let job = app.create_metadata_refresh_job(item.id).await.unwrap();
        let input = job
            .input_json
            .as_ref()
            .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
            .unwrap();

        assert_eq!(job.resource_class, "metadata.bangumi");
        assert_eq!(
            input.get("provider").and_then(serde_json::Value::as_str),
            Some("bangumi")
        );
    }

    #[tokio::test]
    async fn nfo_import_job_imports_sidecar_and_persists_summary() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
        fs::write(
            temp.path().join("demo.nfo"),
            r#"<movie>
  <title>NFO Title</title>
  <plot>NFO overview</plot>
</movie>
"#,
        )
        .unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            metadata: MetadataConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "File Title".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            item_id: item.id,
            locator: "local:///demo.mkv".to_owned(),
            file_name: "demo.mkv".to_owned(),
            size_bytes: Some(5),
            fingerprint: None,
        };
        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert_media_source(library_id, &source)
            .await
            .unwrap();

        let output = app.import_library_nfo(library_id).await.unwrap();
        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        let locks = store.list_field_locks(item.id).await.unwrap();
        let job = app.get_job(output.job.id).await.unwrap();

        assert_eq!(output.job.kind, JobKind::NfoImport);
        assert_eq!(output.job.status, JobStatus::Succeeded);
        assert_eq!(output.import.imported_items, 1);
        assert_eq!(loaded.metadata.title, "NFO Title");
        assert_eq!(loaded.metadata.overview, Some("NFO overview".to_owned()));
        assert!(locks.iter().any(|lock| {
            lock.field == MetadataField::Title && lock.locked && lock.source == MetadataSource::Nfo
        }));
        assert_eq!(job.status, JobStatus::Succeeded);
        assert!(job.summary_json.unwrap().contains("\"imported_items\":1"));
    }
}
