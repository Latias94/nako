use nako_catalog::{CatalogHydrationPort, hydrate_item_catalog};
use nako_core::{
    CanonicalMetadata, ExternalProvider, LibraryId, LibraryItemRepository, LibraryItemState,
    MediaItem, MediaItemId, MediaKind, MediaRepository, MetadataMergePolicy, MetadataRefreshMode,
    MetadataRepository, MetadataSource, NakoError, PageRequest, ProviderMapping, ProviderMappingId,
    ProviderMappingRepository, ProviderMappingStatus, ProviderSubject, ProviderSubjectId,
    ProviderSubjectKind, Result,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HierarchyConfirmationRequest {
    pub library_id: LibraryId,
    pub source: MetadataSource,
    pub refresh_mode: MetadataRefreshMode,
    pub items: Vec<HierarchyConfirmationItem>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HierarchyConfirmationItem {
    pub item_id: MediaItemId,
    pub kind: MediaKind,
    pub parent_id: Option<MediaItemId>,
    pub metadata: CanonicalMetadata,
    pub provider_subject: Option<HierarchyProviderSubject>,
    pub confidence_milli: Option<u16>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HierarchyProviderSubject {
    pub provider: ExternalProvider,
    pub subject_kind: ProviderSubjectKind,
    pub subject_key: String,
    pub title: Option<String>,
    pub release_year: Option<i32>,
    pub locale: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HierarchyConfirmationSummary {
    pub library_id: LibraryId,
    pub confirmed_items: u64,
    pub updated_items: u64,
    pub provider_mappings: u64,
}

#[derive(Debug)]
pub struct HierarchyConfirmationService<R> {
    repository: R,
}

impl<R> HierarchyConfirmationService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    #[must_use]
    pub fn repository(&self) -> &R {
        &self.repository
    }
}

impl<R> HierarchyConfirmationService<R>
where
    R: CatalogHydrationPort
        + LibraryItemRepository
        + MediaRepository
        + MetadataRepository
        + ProviderMappingRepository,
{
    pub async fn confirm_hierarchy(
        &self,
        request: HierarchyConfirmationRequest,
    ) -> Result<HierarchyConfirmationSummary> {
        let mut summary = HierarchyConfirmationSummary {
            library_id: request.library_id,
            ..HierarchyConfirmationSummary::default()
        };

        for item in request.items {
            let existing = self
                .repository
                .get_media_item(item.item_id)
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "media_item",
                    id: item.item_id.to_string(),
                })?;
            let state = self
                .repository
                .get_library_item_state(request.library_id, item.item_id)
                .await?;
            reject_confirmed_structure_change(&existing, &item, state.as_ref())?;

            let updated = self
                .confirmed_item(&existing, &item, &request.source, request.refresh_mode)
                .await?;
            let changed = updated != existing;
            self.repository.upsert_media_item(&updated).await?;
            self.repository
                .upsert_library_item_state(&LibraryItemState {
                    library_id: request.library_id,
                    item_id: updated.id,
                    provisional: false,
                })
                .await?;

            if let Some(subject) = item.provider_subject {
                self.accept_provider_mapping(
                    updated.id,
                    subject,
                    item.confidence_milli,
                    &request.source,
                )
                .await?;
                summary.provider_mappings += 1;
            }

            hydrate_item_catalog(&self.repository, updated.id, request.source.clone()).await?;
            summary.confirmed_items += 1;
            if changed {
                summary.updated_items += 1;
            }
        }

        Ok(summary)
    }

    async fn confirmed_item(
        &self,
        existing: &MediaItem,
        confirmation: &HierarchyConfirmationItem,
        source: &MetadataSource,
        refresh_mode: MetadataRefreshMode,
    ) -> Result<MediaItem> {
        let locks = self.repository.list_field_locks(existing.id).await?;
        let policy = MetadataMergePolicy::for_source_refresh_mode(&locks, source, refresh_mode);

        Ok(MediaItem {
            id: existing.id,
            kind: confirmation.kind,
            parent_id: confirmation.parent_id,
            metadata: policy.merge(&existing.metadata, &confirmation.metadata),
        })
    }

    async fn accept_provider_mapping(
        &self,
        item_id: MediaItemId,
        subject: HierarchyProviderSubject,
        confidence_milli: Option<u16>,
        source: &MetadataSource,
    ) -> Result<()> {
        let subject = self.upsert_provider_subject(subject).await?;
        let mapping_id = self.existing_mapping_id(item_id, subject.id).await?;

        self.repository
            .upsert_provider_mapping(&ProviderMapping {
                id: mapping_id.unwrap_or_else(ProviderMappingId::new),
                item_id,
                subject_id: subject.id,
                status: ProviderMappingStatus::Accepted,
                confidence_milli,
                source: source.clone(),
            })
            .await
    }

    async fn upsert_provider_subject(
        &self,
        subject: HierarchyProviderSubject,
    ) -> Result<ProviderSubject> {
        let existing = self
            .repository
            .find_provider_subject(
                &subject.provider,
                &subject.subject_kind,
                &subject.subject_key,
            )
            .await?;
        let subject = ProviderSubject {
            id: existing
                .as_ref()
                .map(|subject| subject.id)
                .unwrap_or_else(ProviderSubjectId::new),
            provider: subject.provider,
            subject_kind: subject.subject_kind,
            subject_key: subject.subject_key,
            title: subject.title,
            release_year: subject.release_year,
            locale: subject.locale,
        };

        self.repository.upsert_provider_subject(&subject).await?;
        Ok(subject)
    }

    async fn existing_mapping_id(
        &self,
        item_id: MediaItemId,
        subject_id: ProviderSubjectId,
    ) -> Result<Option<ProviderMappingId>> {
        let mut offset = 0;

        loop {
            let mappings = self
                .repository
                .list_provider_mappings_for_item(
                    item_id,
                    PageRequest {
                        limit: PageRequest::MAX_LIMIT,
                        offset,
                    },
                )
                .await?;
            let returned = mappings.len();
            if let Some(mapping) = mappings
                .into_iter()
                .find(|mapping| mapping.subject_id == subject_id)
            {
                return Ok(Some(mapping.id));
            }
            if returned < PageRequest::MAX_LIMIT as usize {
                return Ok(None);
            }
            offset += u64::from(PageRequest::MAX_LIMIT);
        }
    }
}

fn reject_confirmed_structure_change(
    existing: &MediaItem,
    confirmation: &HierarchyConfirmationItem,
    state: Option<&LibraryItemState>,
) -> Result<()> {
    if state.is_some_and(|state| state.provisional) {
        return Ok(());
    }
    if existing.kind == confirmation.kind && existing.parent_id == confirmation.parent_id {
        return Ok(());
    }

    Err(NakoError::Conflict {
        message: format!(
            "confirmed item {} structure cannot be changed through hierarchy confirmation; use hierarchy repair",
            existing.id
        ),
    })
}
