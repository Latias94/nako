use taru_core::{
    CatalogRepository, LibraryItemRepository, LocalMetadataPolicy, MediaKind, MediaRepository,
    MediaSource, MetadataRepository, ProviderMappingRepository, Result, StorageErrorKind,
    TaruError,
};
use taru_search::SearchIndex;
use taru_vfs::{StorageBackend, StorageBackupPolicy, StorageWriteRequest};

use super::{
    NfoBackupPruneFailure, NfoBackupReport, NfoCodec, NfoDocument, NfoExportRequest,
    NfoExportSourceRequest, NfoExportSourceSummary, NfoExportSummary, NfoFailure, NfoFailureKind,
    NfoHierarchy, NfoService, workflow::nfo_uri_for_source,
};

const DEFAULT_NFO_BACKUP_KEEP_LATEST: usize = 5;

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
            backed_up_items: 0,
            backups: Vec::new(),
            pruned_backup_items: 0,
            pruned_backups: 0,
            prune_failures: Vec::new(),
            failures: Vec::new(),
        };

        for source in sources {
            match self.export_source(source, request.force).await {
                NfoExportOutcome::Exported { backup } => {
                    summary.exported_items += 1;
                    if let Some(backup) = backup {
                        summary.backed_up_items += 1;
                        if !backup.pruned_backups.is_empty() {
                            summary.pruned_backup_items += 1;
                            summary.pruned_backups += backup.pruned_backups.len() as u64;
                        }
                        summary.backups.push(backup);
                    }
                }
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
        summary
            .backups
            .sort_by(|left, right| left.locator.cmp(&right.locator));
        for backup in &summary.backups {
            summary.prune_failures.extend(backup.prune_failures.clone());
        }
        summary
            .prune_failures
            .sort_by(|left, right| left.locator.cmp(&right.locator));
        Ok(summary)
    }

    pub async fn export_media_source(
        &self,
        request: NfoExportSourceRequest,
    ) -> Result<NfoExportSourceSummary> {
        ensure_export_policy(request.policy)?;
        let source = self
            .repository
            .get_media_source(request.source_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_source",
                id: request.source_id.to_string(),
            })?;
        if source.library_id != request.library_id {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "NFO export source {} is not in library {}",
                    source.id, request.library_id
                ),
            });
        }

        let mut summary = NfoExportSourceSummary {
            library_id: request.library_id,
            source_id: request.source_id,
            scanned_sources: 1,
            exported_items: 0,
            skipped_items: 0,
            failed_items: 0,
            backed_up_items: 0,
            backups: Vec::new(),
            pruned_backup_items: 0,
            pruned_backups: 0,
            prune_failures: Vec::new(),
            failures: Vec::new(),
        };

        match self.export_source(source, request.force).await {
            NfoExportOutcome::Exported { backup } => {
                summary.exported_items = 1;
                if let Some(backup) = backup {
                    summary.backed_up_items = 1;
                    if !backup.pruned_backups.is_empty() {
                        summary.pruned_backup_items = 1;
                        summary.pruned_backups = backup.pruned_backups.len() as u64;
                    }
                    summary.prune_failures.extend(backup.prune_failures.clone());
                    summary.backups.push(backup);
                }
            }
            NfoExportOutcome::Skipped => {
                summary.skipped_items = 1;
            }
            NfoExportOutcome::Failed(failure) => {
                summary.failed_items = 1;
                summary.failures.push(failure);
            }
        }

        Ok(summary)
    }

    async fn export_source(&self, source: MediaSource, force: bool) -> NfoExportOutcome {
        let nfo_uri = match nfo_uri_for_source(&source) {
            Ok(uri) => uri,
            Err(err) => return export_failure(&source, NfoFailureKind::InvalidSidecarPath, err),
        };
        let mut should_backup = false;
        let existing_xml = if force {
            match self.backend.stat(&nfo_uri).await {
                Ok(_) => match self.backend.read_to_string(&nfo_uri).await {
                    Ok(xml) => {
                        should_backup = true;
                        Some(xml)
                    }
                    Err(err) => return export_failure(&source, classify_read_failure(&err), err),
                },
                Err(TaruError::NotFound { .. }) => None,
                Err(err) => return export_failure(&source, classify_read_failure(&err), err),
            }
        } else {
            match self.backend.stat(&nfo_uri).await {
                Ok(_) => return NfoExportOutcome::Skipped,
                Err(TaruError::NotFound { .. }) => {}
                Err(err) => return export_failure(&source, classify_read_failure(&err), err),
            }
            None
        };

        let item = match self.repository.get_media_item(source.item_id).await {
            Ok(Some(item)) => item,
            Ok(None) => {
                return export_failure(
                    &source,
                    NfoFailureKind::MissingMediaItem,
                    TaruError::NotFound {
                        entity: "media_item",
                        id: source.item_id.to_string(),
                    },
                );
            }
            Err(err) => return export_failure(&source, NfoFailureKind::Unknown, err),
        };

        if item.kind != MediaKind::Movie {
            return NfoExportOutcome::Skipped;
        }

        let document = NfoDocument {
            metadata: item.metadata,
            external_ids: Vec::new(),
            hierarchy: NfoHierarchy::default(),
        };
        let xml = match existing_xml {
            Some(existing_xml) => match self.codec.render_preserving(&document, &existing_xml) {
                Ok(rendered) => rendered.xml,
                Err(err) => {
                    return export_failure(&source, classify_preservation_failure(&err), err);
                }
            },
            None => match self.codec.render(&document) {
                Ok(xml) => xml,
                Err(err) => return export_failure(&source, classify_render_failure(&err), err),
            },
        };

        match self
            .backend
            .write(write_request(nfo_uri, xml, should_backup))
            .await
        {
            Ok(report) => NfoExportOutcome::Exported {
                backup: report.backup.map(|backup| NfoBackupReport {
                    source_id: source.id,
                    locator: source.locator.clone(),
                    original_uri: backup.original_uri,
                    backup_uri: backup.backup_uri,
                    pruned_backups: backup.pruned_backups,
                    prune_failures: backup
                        .prune_failures
                        .into_iter()
                        .map(|failure| NfoBackupPruneFailure {
                            source_id: source.id,
                            locator: source.locator.clone(),
                            backup_uri: failure.uri,
                            message: failure.message,
                        })
                        .collect(),
                }),
            },
            Err(err) => export_failure(&source, classify_write_failure(&err), err),
        }
    }
}

enum NfoExportOutcome {
    Exported { backup: Option<NfoBackupReport> },
    Skipped,
    Failed(NfoFailure),
}

fn export_failure(
    source: &MediaSource,
    kind: NfoFailureKind,
    err: impl ToString,
) -> NfoExportOutcome {
    NfoExportOutcome::Failed(NfoFailure {
        source_id: source.id,
        locator: source.locator.clone(),
        kind,
        message: err.to_string(),
    })
}

fn classify_read_failure(err: &TaruError) -> NfoFailureKind {
    match err {
        TaruError::Storage { .. } => NfoFailureKind::StorageRead,
        TaruError::Unsupported(_) => NfoFailureKind::StorageUnsupported,
        _ => NfoFailureKind::Unknown,
    }
}

fn classify_preservation_failure(err: &TaruError) -> NfoFailureKind {
    match err {
        TaruError::InvalidInput { .. } => NfoFailureKind::NfoPreservation,
        TaruError::Conflict { .. } => NfoFailureKind::NfoConflict,
        TaruError::Unsupported(_) => NfoFailureKind::NfoPreservation,
        _ => NfoFailureKind::Unknown,
    }
}

fn classify_render_failure(err: &TaruError) -> NfoFailureKind {
    match err {
        TaruError::InvalidInput { .. } => NfoFailureKind::NfoRender,
        _ => NfoFailureKind::Unknown,
    }
}

fn classify_write_failure(err: &TaruError) -> NfoFailureKind {
    match err {
        TaruError::Storage {
            kind: StorageErrorKind::Backup,
            ..
        } => NfoFailureKind::StorageBackup,
        TaruError::Storage { .. } => NfoFailureKind::StorageWrite,
        TaruError::Unsupported(_) => NfoFailureKind::StorageUnsupported,
        _ => NfoFailureKind::Unknown,
    }
}

fn write_request(
    nfo_uri: taru_vfs::StorageUri,
    xml: String,
    should_backup: bool,
) -> StorageWriteRequest {
    let request = StorageWriteRequest::atomic_replace(nfo_uri, xml);
    if should_backup {
        request.with_backup_policy(
            StorageBackupPolicy::existing_file().keep_latest(DEFAULT_NFO_BACKUP_KEEP_LATEST),
        )
    } else {
        request
    }
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
