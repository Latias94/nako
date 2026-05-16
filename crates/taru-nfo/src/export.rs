use taru_core::{
    CatalogRepository, LibraryItemRepository, LocalMetadataPolicy, MediaKind, MediaRepository,
    MediaSource, MetadataRepository, ProviderMappingRepository, Result, TaruError,
};
use taru_search::SearchIndex;
use taru_vfs::StorageBackend;

use super::{
    NfoCodec, NfoDocument, NfoExportRequest, NfoExportSummary, NfoFailure, NfoHierarchy,
    NfoService, workflow::nfo_uri_for_source,
};

impl<B, R, C> NfoService<B, R, C>
where
    B: StorageBackend,
    R: CatalogRepository
        + LibraryItemRepository
        + MediaRepository
        + MetadataRepository
        + ProviderMappingRepository
        + SearchIndex,
    C: NfoCodec,
{
    pub async fn export_library(&self, request: NfoExportRequest) -> Result<NfoExportSummary> {
        ensure_export_policy(request.policy)?;

        let sources = self.list_all_sources(request.library_id).await?;
        let mut summary = NfoExportSummary {
            job_id: request.job_id,
            library_id: request.library_id,
            scanned_sources: sources.len() as u64,
            exported_items: 0,
            skipped_items: 0,
            failed_items: 0,
            failures: Vec::new(),
        };

        for source in sources {
            match self.export_source(source, request.force).await {
                NfoExportOutcome::Exported => summary.exported_items += 1,
                NfoExportOutcome::Skipped => summary.skipped_items += 1,
                NfoExportOutcome::Failed(failure) => {
                    summary.failed_items += 1;
                    summary.failures.push(failure);
                }
            }
        }

        summary
            .failures
            .sort_by(|left, right| left.locator.cmp(&right.locator));
        Ok(summary)
    }

    async fn export_source(&self, source: MediaSource, force: bool) -> NfoExportOutcome {
        let nfo_uri = match nfo_uri_for_source(&source) {
            Ok(uri) => uri,
            Err(err) => return export_failure(&source, err),
        };
        if !force {
            match self.backend.stat(&nfo_uri).await {
                Ok(_) => return NfoExportOutcome::Skipped,
                Err(TaruError::NotFound { .. }) => {}
                Err(err) => return export_failure(&source, err),
            }
        }

        let item = match self.repository.get_media_item(source.item_id).await {
            Ok(Some(item)) => item,
            Ok(None) => {
                return export_failure(
                    &source,
                    TaruError::NotFound {
                        entity: "media_item",
                        id: source.item_id.to_string(),
                    },
                );
            }
            Err(err) => return export_failure(&source, err),
        };

        if item.kind != MediaKind::Movie {
            return NfoExportOutcome::Skipped;
        }

        let xml = match self.codec.render(&NfoDocument {
            metadata: item.metadata,
            external_ids: Vec::new(),
            hierarchy: NfoHierarchy::default(),
        }) {
            Ok(xml) => xml,
            Err(err) => return export_failure(&source, err),
        };

        match self.backend.write_string(&nfo_uri, &xml).await {
            Ok(()) => NfoExportOutcome::Exported,
            Err(err) => export_failure(&source, err),
        }
    }
}

enum NfoExportOutcome {
    Exported,
    Skipped,
    Failed(NfoFailure),
}

fn export_failure(source: &MediaSource, err: impl ToString) -> NfoExportOutcome {
    NfoExportOutcome::Failed(NfoFailure {
        source_id: source.id,
        locator: source.locator.clone(),
        message: err.to_string(),
    })
}

fn ensure_export_policy(policy: LocalMetadataPolicy) -> Result<()> {
    if policy == LocalMetadataPolicy::WriteSidecar {
        Ok(())
    } else {
        Err(TaruError::Unsupported(
            "NFO export requires write-sidecar local metadata policy",
        ))
    }
}
