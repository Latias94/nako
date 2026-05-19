use std::collections::HashSet;

use taru_catalog::plan_item_catalog_projection;
use taru_core::{
    CanonicalMetadata, CatalogRepository, LibraryId, LibraryItemRepository, LibraryItemState,
    LocalMetadataPolicy, MediaItem, MediaItemId, MediaKind, MediaRepository, MediaSource,
    MetadataField, MetadataFieldLock, MetadataMergePolicy, MetadataRefreshMode, MetadataRepository,
    MetadataSource, NfoImportPersistenceCommit, Result, TaruError, populated_metadata_fields,
};
use taru_vfs::StorageBackend;

use super::{
    NfoCancellationCheck, NfoCancellationDecision, NfoCodec, NfoDocument, NfoFailure,
    NfoFailureKind, NfoImportRequest, NfoImportSummary, NfoLibraryRunOutcome, NfoService,
    NfoSidecarCheckpoint, NfoSidecarOperation, NoopNfoCancellationCheck,
    workflow::nfo_uri_for_source,
};

pub trait NfoImportRepository:
    CatalogRepository + LibraryItemRepository + MediaRepository + MetadataRepository
{
}

impl<T> NfoImportRepository for T where
    T: CatalogRepository + LibraryItemRepository + MediaRepository + MetadataRepository
{
}

impl<B, R, C> NfoService<B, R, C>
where
    B: StorageBackend,
    R: NfoImportRepository,
    C: NfoCodec,
{
    pub async fn import_library(&self, request: NfoImportRequest) -> Result<NfoImportSummary> {
        Ok(self
            .import_library_with_cancellation(request, &NoopNfoCancellationCheck)
            .await?
            .into_summary())
    }

    pub async fn import_library_with_cancellation(
        &self,
        request: NfoImportRequest,
        cancellation: &dyn NfoCancellationCheck,
    ) -> Result<NfoLibraryRunOutcome<NfoImportSummary>> {
        ensure_import_policy(request.policy)?;

        let sources = self.list_all_sources(request.library_id).await?;
        let mut summary = NfoImportSummary {
            job_id: request.job_id,
            library_id: request.library_id,
            scanned_sources: sources.len() as u64,
            discovered_nfo: 0,
            imported_items: 0,
            skipped_items: 0,
            failed_items: 0,
            failures: Vec::new(),
        };

        for source in sources {
            if cancellation
                .check(NfoSidecarCheckpoint {
                    operation: NfoSidecarOperation::Import,
                    library_id: request.library_id,
                    source_id: source.id,
                    item_id: source.item_id,
                })
                .await?
                == NfoCancellationDecision::Cancel
            {
                sort_import_failures(&mut summary);
                return Ok(NfoLibraryRunOutcome::Cancelled(summary));
            }

            match self
                .import_source(source, request.policy, request.force)
                .await
            {
                NfoImportOutcome::Imported => {
                    summary.discovered_nfo += 1;
                    summary.imported_items += 1;
                }
                NfoImportOutcome::Skipped { discovered } => {
                    if discovered {
                        summary.discovered_nfo += 1;
                    }
                    summary.skipped_items += 1;
                }
                NfoImportOutcome::Failed(failure) => {
                    summary.failed_items += 1;
                    summary.failures.push(failure);
                }
            }
        }

        sort_import_failures(&mut summary);
        Ok(NfoLibraryRunOutcome::Completed(summary))
    }

    async fn import_source(
        &self,
        source: MediaSource,
        policy: LocalMetadataPolicy,
        force: bool,
    ) -> NfoImportOutcome {
        let nfo_uri = match nfo_uri_for_source(&source) {
            Ok(uri) => uri,
            Err(err) => return import_failure(&source, NfoFailureKind::InvalidSidecarPath, err),
        };
        let xml = match self.backend.read_to_string(&nfo_uri).await {
            Ok(xml) => xml,
            Err(TaruError::NotFound { .. }) => {
                return NfoImportOutcome::Skipped { discovered: false };
            }
            Err(err) => return import_failure(&source, classify_read_failure(&err), err),
        };
        let document = match self.codec.parse(&xml) {
            Ok(document) => document,
            Err(err) => return import_failure(&source, NfoFailureKind::NfoParse, err),
        };
        let existing = match self.repository.get_media_item(source.item_id).await {
            Ok(Some(item)) => item,
            Ok(None) => {
                return import_failure(
                    &source,
                    NfoFailureKind::MissingMediaItem,
                    TaruError::NotFound {
                        entity: "media_item",
                        id: source.item_id.to_string(),
                    },
                );
            }
            Err(err) => return import_failure(&source, NfoFailureKind::Unknown, err),
        };

        if !force && policy == LocalMetadataPolicy::RemoteFirst && !is_missing_metadata(&existing) {
            return NfoImportOutcome::Skipped { discovered: true };
        }

        let plan = match self
            .plan_nfo_import_commit(&source, existing, &document, policy, force)
            .await
        {
            Ok(plan) => plan,
            Err(err) => return import_failure(&source, NfoFailureKind::Unknown, err),
        };

        if let Err(err) = self.repository.commit_nfo_import(&plan.commit).await {
            return import_failure(&source, NfoFailureKind::Unknown, err);
        }

        if !plan.changed && !plan.confirmed_hierarchy && !force {
            return NfoImportOutcome::Skipped { discovered: true };
        }

        NfoImportOutcome::Imported
    }

    async fn plan_nfo_import_commit(
        &self,
        source: &MediaSource,
        existing: MediaItem,
        document: &NfoDocument,
        policy: LocalMetadataPolicy,
        force: bool,
    ) -> Result<PlannedNfoImport> {
        let locks = self.repository.list_field_locks(existing.id).await?;
        let merged = MetadataMergePolicy::for_nfo_import(policy, &locks)
            .merge(&existing.metadata, &document.metadata);
        let changed = merged != existing.metadata;
        let updated = MediaItem {
            metadata: merged,
            ..existing.clone()
        };
        let confirmation_items = self
            .nfo_hierarchy_confirmation_items(&existing, document)
            .await?;
        let confirmed_hierarchy = !confirmation_items.is_empty();
        let mut items = Vec::new();
        let mut library_item_states = Vec::new();

        if confirmed_hierarchy {
            let confirmation = self
                .plan_nfo_hierarchy_confirmation(source.library_id, &confirmation_items)
                .await?;
            items.extend(confirmation.items);
            library_item_states.extend(confirmation.library_item_states);
        } else if changed || force {
            items.push(updated.clone());
        }

        let field_locks =
            if (changed || confirmed_hierarchy || force) && locks_should_be_written(policy) {
                nfo_field_locks(updated.id, &document.metadata, &locks)
            } else {
                Vec::new()
            };
        let projection_items = if confirmed_hierarchy {
            items.clone()
        } else {
            vec![updated]
        };
        let mut catalog_projections = Vec::new();
        for item in projection_items {
            catalog_projections.push(
                plan_item_catalog_projection(&self.repository, item, MetadataSource::Nfo).await?,
            );
        }

        Ok(PlannedNfoImport {
            commit: NfoImportPersistenceCommit {
                items,
                field_locks,
                library_item_states,
                catalog_projections,
            },
            changed,
            confirmed_hierarchy,
        })
    }

    async fn plan_nfo_hierarchy_confirmation(
        &self,
        library_id: LibraryId,
        items: &[NfoHierarchyConfirmationItem],
    ) -> Result<PlannedNfoHierarchyConfirmation> {
        let mut planned_items = Vec::new();
        let mut library_item_states = Vec::new();

        for item in items {
            let existing = self
                .repository
                .get_media_item(item.item_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "media_item",
                    id: item.item_id.to_string(),
                })?;
            let state = self
                .repository
                .get_library_item_state(library_id, item.item_id)
                .await?;
            reject_confirmed_structure_change(&existing, item, state.as_ref())?;

            let locks = self.repository.list_field_locks(existing.id).await?;
            let policy = MetadataMergePolicy::for_source_refresh_mode(
                &locks,
                &MetadataSource::Nfo,
                MetadataRefreshMode::FullRefresh,
            );
            let updated = MediaItem {
                id: existing.id,
                kind: item.kind,
                parent_id: item.parent_id,
                metadata: policy.merge(&existing.metadata, &item.metadata),
            };
            library_item_states.push(LibraryItemState {
                library_id,
                item_id: updated.id,
                provisional: false,
            });
            planned_items.push(updated);
        }

        Ok(PlannedNfoHierarchyConfirmation {
            items: planned_items,
            library_item_states,
        })
    }

    async fn nfo_hierarchy_confirmation_items(
        &self,
        item: &MediaItem,
        document: &NfoDocument,
    ) -> Result<Vec<NfoHierarchyConfirmationItem>> {
        if document.hierarchy.kind != Some(MediaKind::Episode) && item.kind != MediaKind::Episode {
            return Ok(Vec::new());
        }

        let mut items = Vec::new();
        if let Some(season_id) = item.parent_id {
            let season = self
                .repository
                .get_media_item(season_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "media_item",
                    id: season_id.to_string(),
                })?;
            if let Some(series_id) = season.parent_id {
                let series = self
                    .repository
                    .get_media_item(series_id)
                    .await?
                    .ok_or_else(|| TaruError::NotFound {
                        entity: "media_item",
                        id: series_id.to_string(),
                    })?;
                let mut series_metadata = series.metadata.clone();
                if let Some(title) = document.hierarchy.series_title.as_ref() {
                    series_metadata.title = title.clone();
                }
                items.push(NfoHierarchyConfirmationItem {
                    item_id: series.id,
                    kind: MediaKind::Series,
                    parent_id: series.parent_id,
                    metadata: series_metadata,
                });
            }

            let mut season_metadata = season.metadata.clone();
            if let Some(season_number) = document.hierarchy.season_number {
                season_metadata.title = format!("Season {season_number}");
            }
            items.push(NfoHierarchyConfirmationItem {
                item_id: season.id,
                kind: MediaKind::Season,
                parent_id: season.parent_id,
                metadata: season_metadata,
            });
        }

        items.push(NfoHierarchyConfirmationItem {
            item_id: item.id,
            kind: MediaKind::Episode,
            parent_id: item.parent_id,
            metadata: document.metadata.clone(),
        });

        Ok(items)
    }
}

fn sort_import_failures(summary: &mut NfoImportSummary) {
    summary
        .failures
        .sort_by(|left, right| left.locator.cmp(&right.locator));
}

enum NfoImportOutcome {
    Imported,
    Skipped { discovered: bool },
    Failed(NfoFailure),
}

struct PlannedNfoImport {
    commit: NfoImportPersistenceCommit,
    changed: bool,
    confirmed_hierarchy: bool,
}

struct PlannedNfoHierarchyConfirmation {
    items: Vec<MediaItem>,
    library_item_states: Vec<LibraryItemState>,
}

struct NfoHierarchyConfirmationItem {
    item_id: MediaItemId,
    kind: MediaKind,
    parent_id: Option<MediaItemId>,
    metadata: CanonicalMetadata,
}

fn import_failure(
    source: &MediaSource,
    kind: NfoFailureKind,
    err: impl ToString,
) -> NfoImportOutcome {
    NfoImportOutcome::Failed(NfoFailure {
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

fn ensure_import_policy(policy: LocalMetadataPolicy) -> Result<()> {
    match policy {
        LocalMetadataPolicy::Disabled | LocalMetadataPolicy::WriteSidecar => {
            Err(TaruError::Unsupported(
                "NFO import requires read-only, local-first, or remote-first local metadata policy",
            ))
        }
        LocalMetadataPolicy::ReadOnly
        | LocalMetadataPolicy::LocalFirst
        | LocalMetadataPolicy::RemoteFirst => Ok(()),
    }
}

fn is_missing_metadata(item: &MediaItem) -> bool {
    let metadata = &item.metadata;
    metadata.title.trim().is_empty()
        || metadata.overview.is_none()
        || metadata.release_date.is_none()
        || metadata.runtime_minutes.is_none()
        || metadata.genres.is_empty()
        || metadata.tags.is_empty()
}

fn locks_should_be_written(policy: LocalMetadataPolicy) -> bool {
    matches!(
        policy,
        LocalMetadataPolicy::ReadOnly | LocalMetadataPolicy::LocalFirst
    )
}

fn nfo_field_locks(
    item_id: MediaItemId,
    metadata: &CanonicalMetadata,
    existing_locks: &[MetadataFieldLock],
) -> Vec<MetadataFieldLock> {
    let protected_fields = existing_locks
        .iter()
        .filter(|lock| lock.locked && !matches!(&lock.source, MetadataSource::Nfo))
        .map(|lock| lock.field)
        .collect::<HashSet<MetadataField>>();

    populated_metadata_fields(metadata)
        .into_iter()
        .filter(|field| !protected_fields.contains(field))
        .map(|field| MetadataFieldLock {
            item_id,
            field,
            locked: true,
            source: MetadataSource::Nfo,
        })
        .collect()
}

fn reject_confirmed_structure_change(
    existing: &MediaItem,
    confirmation: &NfoHierarchyConfirmationItem,
    state: Option<&LibraryItemState>,
) -> Result<()> {
    if state.is_some_and(|state| state.provisional) {
        return Ok(());
    }
    if existing.kind == confirmation.kind && existing.parent_id == confirmation.parent_id {
        return Ok(());
    }

    Err(TaruError::Conflict {
        message: format!(
            "confirmed item {} structure cannot be changed through NFO hierarchy confirmation; use hierarchy repair",
            existing.id
        ),
    })
}
