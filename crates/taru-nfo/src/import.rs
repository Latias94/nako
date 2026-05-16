use taru_catalog::hydrate_item_catalog;
use taru_core::{
    CanonicalMetadata, CatalogRepository, LibraryItemRepository, LocalMetadataPolicy, MediaItem,
    MediaRepository, MediaSource, MetadataField, MetadataFieldLock, MetadataRefreshMode,
    MetadataRepository, MetadataSource, ProviderMappingRepository, Result, TaruError,
};
use taru_metadata::{
    HierarchyConfirmationItem, HierarchyConfirmationRequest, HierarchyConfirmationService,
};
use taru_search::SearchIndex;
use taru_vfs::StorageBackend;

use super::{
    NfoCodec, NfoDocument, NfoFailure, NfoImportRequest, NfoImportSummary, NfoService,
    workflow::nfo_uri_for_source,
};

impl<B, R, C> NfoService<B, R, C>
where
    B: StorageBackend,
    R: CatalogRepository
        + Clone
        + LibraryItemRepository
        + MediaRepository
        + MetadataRepository
        + ProviderMappingRepository
        + SearchIndex,
    C: NfoCodec,
{
    pub async fn import_library(&self, request: NfoImportRequest) -> Result<NfoImportSummary> {
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

        summary
            .failures
            .sort_by(|left, right| left.locator.cmp(&right.locator));
        Ok(summary)
    }

    async fn import_source(
        &self,
        source: MediaSource,
        policy: LocalMetadataPolicy,
        force: bool,
    ) -> NfoImportOutcome {
        let nfo_uri = match nfo_uri_for_source(&source) {
            Ok(uri) => uri,
            Err(err) => return import_failure(&source, err),
        };
        let xml = match self.backend.read_to_string(&nfo_uri).await {
            Ok(xml) => xml,
            Err(TaruError::NotFound { .. }) => {
                return NfoImportOutcome::Skipped { discovered: false };
            }
            Err(err) => return import_failure(&source, err),
        };
        let document = match self.codec.parse(&xml) {
            Ok(document) => document,
            Err(err) => return import_failure(&source, err),
        };
        let existing = match self.repository.get_media_item(source.item_id).await {
            Ok(Some(item)) => item,
            Ok(None) => {
                return import_failure(
                    &source,
                    TaruError::NotFound {
                        entity: "media_item",
                        id: source.item_id.to_string(),
                    },
                );
            }
            Err(err) => return import_failure(&source, err),
        };

        if !force && policy == LocalMetadataPolicy::RemoteFirst && !is_missing_metadata(&existing) {
            return NfoImportOutcome::Skipped { discovered: true };
        }

        let locks = match self.repository.list_field_locks(existing.id).await {
            Ok(locks) => locks,
            Err(err) => return import_failure(&source, err),
        };
        let merged = merge_nfo_metadata(&existing.metadata, &document.metadata, policy, &locks);
        let changed = merged != existing.metadata;
        let confirmation_items = match self
            .nfo_hierarchy_confirmation_items(&existing, &document)
            .await
        {
            Ok(items) => items,
            Err(err) => return import_failure(&source, err),
        };
        if !changed && confirmation_items.is_empty() && !force {
            if let Err(err) =
                hydrate_item_catalog(&self.repository, existing.id, MetadataSource::Nfo).await
            {
                return import_failure(&source, err);
            }
            return NfoImportOutcome::Skipped { discovered: true };
        }

        let updated = MediaItem {
            metadata: merged,
            ..existing
        };
        if let Err(err) = self.repository.upsert_media_item(&updated).await {
            return import_failure(&source, err);
        }

        if locks_should_be_written(policy) {
            for field in populated_fields(&document.metadata) {
                if let Err(err) = self
                    .repository
                    .upsert_field_lock(&MetadataFieldLock {
                        item_id: updated.id,
                        field,
                        locked: true,
                        source: MetadataSource::Nfo,
                    })
                    .await
                {
                    return import_failure(&source, err);
                }
            }
        }

        if confirmation_items.is_empty() {
            if let Err(err) =
                hydrate_item_catalog(&self.repository, updated.id, MetadataSource::Nfo).await
            {
                return import_failure(&source, err);
            }
        } else {
            let confirmation = HierarchyConfirmationService::new(self.repository.clone());
            if let Err(err) = confirmation
                .confirm_hierarchy(HierarchyConfirmationRequest {
                    library_id: source.library_id,
                    source: MetadataSource::Nfo,
                    refresh_mode: MetadataRefreshMode::FullRefresh,
                    items: confirmation_items,
                })
                .await
            {
                return import_failure(&source, err);
            }
        }

        NfoImportOutcome::Imported
    }

    async fn nfo_hierarchy_confirmation_items(
        &self,
        item: &MediaItem,
        document: &NfoDocument,
    ) -> Result<Vec<HierarchyConfirmationItem>> {
        if document.hierarchy.kind != Some(taru_core::MediaKind::Episode)
            && item.kind != taru_core::MediaKind::Episode
        {
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
                items.push(HierarchyConfirmationItem {
                    item_id: series.id,
                    kind: taru_core::MediaKind::Series,
                    parent_id: series.parent_id,
                    metadata: series_metadata,
                    provider_subject: None,
                    confidence_milli: None,
                });
            }

            let mut season_metadata = season.metadata.clone();
            if let Some(season_number) = document.hierarchy.season_number {
                season_metadata.title = format!("Season {season_number}");
            }
            items.push(HierarchyConfirmationItem {
                item_id: season.id,
                kind: taru_core::MediaKind::Season,
                parent_id: season.parent_id,
                metadata: season_metadata,
                provider_subject: None,
                confidence_milli: None,
            });
        }

        items.push(HierarchyConfirmationItem {
            item_id: item.id,
            kind: taru_core::MediaKind::Episode,
            parent_id: item.parent_id,
            metadata: document.metadata.clone(),
            provider_subject: None,
            confidence_milli: None,
        });

        Ok(items)
    }
}

enum NfoImportOutcome {
    Imported,
    Skipped { discovered: bool },
    Failed(NfoFailure),
}

fn import_failure(source: &MediaSource, err: impl ToString) -> NfoImportOutcome {
    NfoImportOutcome::Failed(NfoFailure {
        source_id: source.id,
        locator: source.locator.clone(),
        message: err.to_string(),
    })
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

fn merge_nfo_metadata(
    existing: &CanonicalMetadata,
    incoming: &CanonicalMetadata,
    policy: LocalMetadataPolicy,
    locks: &[MetadataFieldLock],
) -> CanonicalMetadata {
    match policy {
        LocalMetadataPolicy::ReadOnly
        | LocalMetadataPolicy::LocalFirst
        | LocalMetadataPolicy::WriteSidecar => merge_with_mode(existing, incoming, locks, false),
        LocalMetadataPolicy::RemoteFirst => merge_with_mode(existing, incoming, locks, true),
        LocalMetadataPolicy::Disabled => existing.clone(),
    }
}

fn merge_with_mode(
    existing: &CanonicalMetadata,
    incoming: &CanonicalMetadata,
    locks: &[MetadataFieldLock],
    missing_only: bool,
) -> CanonicalMetadata {
    let mut merged = existing.clone();

    if should_replace_text(MetadataField::Title, &merged.title, locks, missing_only) {
        merged.title = incoming.title.clone();
    }
    if should_replace_option(
        MetadataField::OriginalTitle,
        &merged.original_title,
        locks,
        missing_only,
    ) {
        merged.original_title = incoming.original_title.clone();
    }
    if should_replace_option(
        MetadataField::SortTitle,
        &merged.sort_title,
        locks,
        missing_only,
    ) {
        merged.sort_title = incoming.sort_title.clone();
    }
    if should_replace_option(
        MetadataField::Overview,
        &merged.overview,
        locks,
        missing_only,
    ) {
        merged.overview = incoming.overview.clone();
    }
    if should_replace_option(
        MetadataField::ReleaseDate,
        &merged.release_date,
        locks,
        missing_only,
    ) {
        merged.release_date = incoming.release_date.clone();
    }
    if should_replace_option(
        MetadataField::RuntimeMinutes,
        &merged.runtime_minutes,
        locks,
        missing_only,
    ) {
        merged.runtime_minutes = incoming.runtime_minutes;
    }
    if should_replace_option(MetadataField::Tagline, &merged.tagline, locks, missing_only) {
        merged.tagline = incoming.tagline.clone();
    }
    if should_replace_list(MetadataField::Genres, &merged.genres, locks, missing_only) {
        merged.genres = incoming.genres.clone();
    }
    if should_replace_list(MetadataField::Tags, &merged.tags, locks, missing_only) {
        merged.tags = incoming.tags.clone();
    }
    if should_replace_list(MetadataField::Ratings, &merged.ratings, locks, missing_only) {
        merged.ratings = incoming.ratings.clone();
    }
    if should_replace_list(MetadataField::Images, &merged.images, locks, missing_only) {
        merged.images = incoming.images.clone();
    }
    if should_replace_list(MetadataField::Credits, &merged.credits, locks, missing_only) {
        merged.credits = incoming.credits.clone();
    }
    if should_replace_list(
        MetadataField::Collections,
        &merged.collections,
        locks,
        missing_only,
    ) {
        merged.collections = incoming.collections.clone();
    }
    if should_replace_list(MetadataField::Studios, &merged.studios, locks, missing_only) {
        merged.studios = incoming.studios.clone();
    }
    if should_replace_list(
        MetadataField::ExternalIds,
        &merged.external_ids,
        locks,
        missing_only,
    ) {
        merged.external_ids = incoming.external_ids.clone();
    }

    merged
}

fn should_replace_text(
    field: MetadataField,
    existing: &str,
    locks: &[MetadataFieldLock],
    missing_only: bool,
) -> bool {
    !is_protected_by_non_nfo_lock(field, locks) && (!missing_only || existing.is_empty())
}

fn should_replace_option<T>(
    field: MetadataField,
    existing: &Option<T>,
    locks: &[MetadataFieldLock],
    missing_only: bool,
) -> bool {
    !is_protected_by_non_nfo_lock(field, locks) && (!missing_only || existing.is_none())
}

fn should_replace_list<T>(
    field: MetadataField,
    existing: &[T],
    locks: &[MetadataFieldLock],
    missing_only: bool,
) -> bool {
    !is_protected_by_non_nfo_lock(field, locks) && (!missing_only || existing.is_empty())
}

fn is_protected_by_non_nfo_lock(field: MetadataField, locks: &[MetadataFieldLock]) -> bool {
    locks.iter().any(|lock| {
        lock.locked && lock.field == field && !matches!(lock.source, MetadataSource::Nfo)
    })
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

fn populated_fields(metadata: &CanonicalMetadata) -> Vec<MetadataField> {
    let mut fields = Vec::new();

    if !metadata.title.trim().is_empty() {
        fields.push(MetadataField::Title);
    }
    if metadata.original_title.is_some() {
        fields.push(MetadataField::OriginalTitle);
    }
    if metadata.sort_title.is_some() {
        fields.push(MetadataField::SortTitle);
    }
    if metadata.overview.is_some() {
        fields.push(MetadataField::Overview);
    }
    if metadata.release_date.is_some() {
        fields.push(MetadataField::ReleaseDate);
    }
    if metadata.runtime_minutes.is_some() {
        fields.push(MetadataField::RuntimeMinutes);
    }
    if metadata.tagline.is_some() {
        fields.push(MetadataField::Tagline);
    }
    if !metadata.genres.is_empty() {
        fields.push(MetadataField::Genres);
    }
    if !metadata.tags.is_empty() {
        fields.push(MetadataField::Tags);
    }
    if !metadata.ratings.is_empty() {
        fields.push(MetadataField::Ratings);
    }
    if !metadata.images.is_empty() {
        fields.push(MetadataField::Images);
    }
    if !metadata.credits.is_empty() {
        fields.push(MetadataField::Credits);
    }
    if !metadata.collections.is_empty() {
        fields.push(MetadataField::Collections);
    }
    if !metadata.studios.is_empty() {
        fields.push(MetadataField::Studios);
    }
    if !metadata.external_ids.is_empty() {
        fields.push(MetadataField::ExternalIds);
    }

    fields
}
