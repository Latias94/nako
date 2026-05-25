use nako_core::{JobId, Library};
use nako_db::NakoDatabase;
use nako_nfo::{
    MovieNfoCodec, NfoImportRequest, NfoImportSummary, NfoLibraryRunOutcome, NfoService,
};
use serde::Serialize;
use tracing::info;

use super::{
    addons::{
        AddonAppService, ScanAddonBulkMetadataScrapeRequest, ScanAddonBulkMetadataScrapeSummary,
    },
    job_runtime::{DurableJobContext, DurableJobOperationError, DurableJobOperationResult},
    storage::StorageBackendRegistry,
};

#[derive(Clone, Debug)]
pub(crate) struct MetadataScanAcquisitionService {
    store: NakoDatabase,
    storage_backends: StorageBackendRegistry,
    addons: AddonAppService,
}

#[derive(Clone, Debug)]
pub(crate) struct MetadataScanAcquisitionRequest<'a> {
    pub(crate) job_id: JobId,
    pub(crate) library: &'a Library,
    pub(crate) context: DurableJobContext,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct LibraryScanMetadataSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nfo_import: Option<NfoImportSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addon_scrape: Option<ScanAddonBulkMetadataScrapeSummary>,
}

impl MetadataScanAcquisitionService {
    pub(crate) fn new(
        store: NakoDatabase,
        storage_backends: StorageBackendRegistry,
        addons: AddonAppService,
    ) -> Self {
        Self {
            store,
            storage_backends,
            addons,
        }
    }

    pub(crate) async fn run(
        &self,
        request: MetadataScanAcquisitionRequest<'_>,
    ) -> DurableJobOperationResult<LibraryScanMetadataSummary> {
        let plan = request
            .library
            .options
            .metadata_profile
            .scan_acquisition_plan();
        let mut summary = LibraryScanMetadataSummary::default();

        if plan.local_nfo_import {
            request.context.check_cancelled().await?;
            info!(
                job_id = %request.job_id,
                library_id = %request.library.id,
                "running scan-time NFO import"
            );
            let storage_backend = self
                .storage_backends
                .backend_for_library_root(request.library)
                .await?;
            let nfo = NfoService::new(storage_backend, self.store.clone(), MovieNfoCodec);
            let import = nfo
                .import_library_with_cancellation(
                    NfoImportRequest {
                        job_id: request.job_id,
                        library_id: request.library.id,
                        policy: request
                            .library
                            .options
                            .metadata_profile
                            .local_metadata_policy,
                        force: false,
                    },
                    &ScanMetadataCancellationCheck {
                        context: request.context.clone(),
                    },
                )
                .await?;
            match import {
                NfoLibraryRunOutcome::Completed(import) => {
                    summary.nfo_import = Some(import);
                }
                NfoLibraryRunOutcome::Cancelled(_) => {
                    return Err(DurableJobOperationError::Cancelled);
                }
            }
        }

        if plan.addon_scrape {
            request.context.check_cancelled().await?;
            info!(
                job_id = %request.job_id,
                library_id = %request.library.id,
                "creating scan-time Addon bulk metadata scrape task runs"
            );
            let addon_scrape = self
                .addons
                .create_scan_bulk_metadata_scrape_task_runs(ScanAddonBulkMetadataScrapeRequest {
                    scan_job_id: request.job_id,
                    library: request.library,
                    writeback: plan.addon_writeback,
                })
                .await?;
            summary.addon_scrape = Some(addon_scrape);
        }

        Ok(summary)
    }
}

#[derive(Clone, Debug)]
struct ScanMetadataCancellationCheck {
    context: DurableJobContext,
}

#[async_trait::async_trait]
impl nako_nfo::NfoCancellationCheck for ScanMetadataCancellationCheck {
    async fn check(
        &self,
        _checkpoint: nako_nfo::NfoSidecarCheckpoint,
    ) -> nako_core::Result<nako_nfo::NfoCancellationDecision> {
        match self.context.check_cancelled().await {
            Ok(()) => Ok(nako_nfo::NfoCancellationDecision::Continue),
            Err(DurableJobOperationError::Cancelled) => {
                Ok(nako_nfo::NfoCancellationDecision::Cancel)
            }
            Err(DurableJobOperationError::Failed(err)) => Err(err),
        }
    }
}
