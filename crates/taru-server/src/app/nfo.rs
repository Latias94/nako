use std::sync::Arc;

use serde::Serialize;
use taru_core::{
    DomainEventKind, DomainEventSubject, EventId, EventOutboxRepository, Job, JobId, JobKind,
    JobRepository, Library, LibraryId, NewJob, NewOutboxEvent, Result, TaruError,
};
use taru_db::SqliteStore;
use taru_nfo::{
    MovieNfoCodec, NfoExportRequest, NfoExportSummary, NfoImportRequest, NfoImportSummary,
    NfoJobInput, NfoService,
};
use taru_vfs::{StorageBackend, StorageCapabilities, StorageUri};
use tokio::sync::Semaphore;
use tracing::{Instrument, info, info_span, warn};

use crate::config::{TaruServerConfig, libraries_from_config};

use super::{
    job_runtime::DurableJobRuntime, runtime::RuntimeSupervisor, storage::StorageBackendRegistry,
};

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

#[derive(Clone, Debug)]
pub(crate) struct NfoAppService {
    config: TaruServerConfig,
    store: SqliteStore,
    permits: Arc<Semaphore>,
    storage_backends: StorageBackendRegistry,
    runtime: RuntimeSupervisor,
}

impl NfoAppService {
    pub(super) fn new(
        config: TaruServerConfig,
        store: SqliteStore,
        permits: Arc<Semaphore>,
        storage_backends: StorageBackendRegistry,
        runtime: RuntimeSupervisor,
    ) -> Self {
        Self {
            config,
            store,
            permits,
            storage_backends,
            runtime,
        }
    }

    pub(crate) async fn enqueue_nfo_import(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_nfo_import_job(library_id).await?;
        let job_id = job.id;
        let service = self.clone();

        self.runtime.spawn_job(
            "nfo_import_background_job",
            job.resource_class.clone(),
            job_id,
            move |_context| {
                async move { service.finish_nfo_import_job(job_id, library_id).await }.instrument(
                    info_span!(
                        "nfo_import_background_job",
                        job_id = %job_id,
                        library_id = %library_id,
                        resource_class = "metadata.nfo.import"
                    ),
                )
            },
        );

        Ok(job)
    }

    pub(crate) async fn enqueue_nfo_export(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_nfo_export_job(library_id).await?;
        let job_id = job.id;
        let service = self.clone();

        self.runtime.spawn_job(
            "nfo_export_background_job",
            job.resource_class.clone(),
            job_id,
            move |_context| {
                async move { service.finish_nfo_export_job(job_id, library_id).await }.instrument(
                    info_span!(
                        "nfo_export_background_job",
                        job_id = %job_id,
                        library_id = %library_id,
                        resource_class = "metadata.nfo.export"
                    ),
                )
            },
        );

        Ok(job)
    }

    pub(crate) async fn import_library_nfo(
        &self,
        library_id: LibraryId,
    ) -> Result<NfoImportCommandOutput> {
        let job = self.create_nfo_import_job(library_id).await?;
        self.execute_nfo_import_job(job.id, library_id).await
    }

    pub(crate) async fn export_library_nfo(
        &self,
        library_id: LibraryId,
    ) -> Result<NfoExportCommandOutput> {
        let job = self.create_nfo_export_job(library_id).await?;
        self.execute_nfo_export_job(job.id, library_id).await
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

        self.store
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

        self.store
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

    async fn finish_nfo_import_job(&self, job_id: JobId, library_id: LibraryId) -> Result<Job> {
        let output = self.execute_nfo_import_job(job_id, library_id).await?;
        info!(
            job_id = %output.job.id,
            library_id = %library_id,
            imported_items = output.import.imported_items,
            status = ?output.job.status,
            "NFO import job completed"
        );
        Ok(output.job)
    }

    async fn finish_nfo_export_job(&self, job_id: JobId, library_id: LibraryId) -> Result<Job> {
        let output = self.execute_nfo_export_job(job_id, library_id).await?;
        info!(
            job_id = %output.job.id,
            library_id = %library_id,
            exported_items = output.export.exported_items,
            status = ?output.job.status,
            "NFO export job completed"
        );
        Ok(output.job)
    }

    async fn execute_nfo_import_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoImportCommandOutput> {
        let permit =
            self.permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|err| TaruError::InvalidInput {
                    message: format!("NFO concurrency limiter is unavailable: {err}"),
                })?;
        let _permit = permit;

        let runtime = DurableJobRuntime::new(self.store.clone());
        let run = runtime
            .run_job(
                job_id,
                "NFO import job",
                || async { self.run_nfo_import(job_id, library_id).await },
                |import| DurableJobRuntime::serialize_summary(import, "NFO import job summary"),
            )
            .await?;
        let import = run.output;
        self.record_nfo_imported_event(job_id, library_id, &import)
            .await;

        Ok(NfoImportCommandOutput {
            job: run.job,
            import,
        })
    }

    async fn execute_nfo_export_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoExportCommandOutput> {
        let permit =
            self.permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|err| TaruError::InvalidInput {
                    message: format!("NFO concurrency limiter is unavailable: {err}"),
                })?;
        let _permit = permit;

        let runtime = DurableJobRuntime::new(self.store.clone());
        let run = runtime
            .run_job(
                job_id,
                "NFO export job",
                || async { self.run_nfo_export(job_id, library_id).await },
                |export| DurableJobRuntime::serialize_summary(export, "NFO export job summary"),
            )
            .await?;
        let export = run.output;
        self.record_nfo_exported_event(job_id, library_id, &export)
            .await;

        Ok(NfoExportCommandOutput {
            job: run.job,
            export,
        })
    }

    async fn record_nfo_imported_event(
        &self,
        job_id: JobId,
        library_id: LibraryId,
        import: &NfoImportSummary,
    ) {
        let payload = serde_json::json!({
            "job_id": job_id,
            "library_id": library_id,
            "scanned_sources": import.scanned_sources,
            "discovered_nfo": import.discovered_nfo,
            "imported_items": import.imported_items,
            "skipped_items": import.skipped_items,
            "failed_items": import.failed_items,
        });
        self.record_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::NfoImported,
            subject: DomainEventSubject::Library(library_id),
            library_id: Some(library_id),
            source_id: None,
            idempotency_key: format!("nfo.imported:{job_id}:{library_id}"),
            payload_json: payload.to_string(),
        })
        .await;
    }

    async fn record_nfo_exported_event(
        &self,
        job_id: JobId,
        library_id: LibraryId,
        export: &NfoExportSummary,
    ) {
        let payload = serde_json::json!({
            "job_id": job_id,
            "library_id": library_id,
            "scanned_sources": export.scanned_sources,
            "exported_items": export.exported_items,
            "skipped_items": export.skipped_items,
            "failed_items": export.failed_items,
            "backed_up_items": export.backed_up_items,
            "pruned_backup_items": export.pruned_backup_items,
            "pruned_backups": export.pruned_backups,
            "prune_failures": export.prune_failures.len(),
        });
        self.record_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::NfoExported,
            subject: DomainEventSubject::Library(library_id),
            library_id: Some(library_id),
            source_id: None,
            idempotency_key: format!("nfo.exported:{job_id}:{library_id}"),
            payload_json: payload.to_string(),
        })
        .await;
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

        let backend = self
            .storage_backends
            .backend_for_library_root(&library)
            .await?;
        let service = NfoService::new(backend, self.store.clone(), MovieNfoCodec);

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

        let backend = self
            .storage_backends
            .backend_for_library_root(&library)
            .await?;
        ensure_nfo_export_writable(backend.as_ref(), &library).await?;
        let service = NfoService::new(backend, self.store.clone(), MovieNfoCodec);

        service
            .export_library(NfoExportRequest {
                job_id,
                library_id,
                policy: library.options.metadata_profile.local_metadata_policy,
                force: false,
            })
            .await
    }

    async fn record_outbox_event(&self, event: NewOutboxEvent) {
        let kind = event.kind.as_str();
        let idempotency_key = event.idempotency_key.clone();
        if let Err(err) = self.store.enqueue_outbox_event(event).await {
            warn!(
                kind,
                idempotency_key,
                error = %err,
                "failed to persist outbox event"
            );
        }
    }

    fn configured_library_for(&self, library_id: LibraryId) -> Result<Library> {
        libraries_from_config(&self.config)
            .into_iter()
            .find(|library| library.id == library_id)
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
    }
}

async fn ensure_nfo_export_writable(
    backend: &dyn StorageBackend,
    library: &taru_core::Library,
) -> Result<()> {
    let root = library
        .roots
        .first()
        .map(String::as_str)
        .unwrap_or("local:///");
    let uri = StorageUri::parse(root)?;
    let metadata = backend.stat(&uri).await?;

    if metadata
        .capabilities
        .contains(StorageCapabilities::WRITABLE)
    {
        Ok(())
    } else {
        Err(TaruError::Unsupported(
            "NFO export requires a writable storage backend",
        ))
    }
}
