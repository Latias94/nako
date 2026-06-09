use std::collections::HashSet;

use nako_api::{
    admin::{
        AdminCatalogGovernanceItem, AdminCatalogGovernanceItemDetailResponse,
        AdminCatalogGovernanceProviderMappingReviewDecision,
        AdminCatalogGovernanceProviderMappingReviewPlan,
        AdminCatalogGovernanceProviderMappingReviewPlanResponse,
        AdminCatalogGovernanceProviderMappingReviewResponse,
        AdminCatalogGovernanceProviderMappingSummary, AdminOverviewCatalogSummary,
        catalog_governance_record_from_item_sources_and_counts,
    },
    public_client::{
        GenreItemsResponse, GenreListResponse, ImagesResponse, ItemCreditsResponse,
        ItemDetailResponse, ItemsResponse, PeopleResponse, PersonItemsResponse, PersonResponse,
        SearchItemHit, SearchResponse, TagItemsResponse, TagsResponse, collection_item_to_dto,
        genre_to_dto, item_credit_to_dto, item_genre_to_dto, item_studio_to_dto, item_tag_to_dto,
        media_item_to_dto, media_probe_to_dto, media_source_to_dto, page_info_from_request,
        person_to_dto, selected_artwork_to_public_image_ref, tag_to_dto,
    },
};
use nako_core::{
    AuthenticatedPrincipal, CatalogGovernanceItemListFilter, CatalogGovernanceItemRecord,
    CatalogGovernanceRepository, CatalogRepository, GenreId, IdentityAccessRepository,
    LocalInferenceEvidence, LocalInferenceRepository, ManagedArtworkRepository, MediaItem,
    MediaItemId, MediaProbeRepository, MediaRepository, MediaSource, MediaSourceId, NakoError,
    PageRequest, PersonId, ProviderMapping, ProviderMappingId, ProviderMappingRepository,
    ProviderMappingStatus, Result, SourceDuplicateEvidenceKind, SourceDuplicateRelationship,
    SourceDuplicateRelationshipId, SourceDuplicateRelationshipStatus, SourceDuplicateRepository,
    TagId,
};
use nako_db::NakoDatabase;
use nako_search::SearchQuery;
use nako_vfs::{StorageLinkKind, StorageLinkPlan, StorageLinkPlanStatus};

#[derive(Clone, Debug)]
pub(crate) struct CatalogAppService {
    store: NakoDatabase,
}

impl CatalogAppService {
    pub(crate) fn new(store: NakoDatabase) -> Self {
        Self { store }
    }

    pub async fn list_accessible_items(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<ItemsResponse> {
        let page = page.clamped();
        let items = self
            .store
            .list_accessible_media_items(principal, page)
            .await?;

        Ok(ItemsResponse {
            page: page_info_from_request(page, items.len()),
            items: items.into_iter().map(media_item_to_dto).collect(),
        })
    }

    pub async fn list_catalog_governance_items(
        &self,
        filter: CatalogGovernanceItemListFilter,
        page: PageRequest,
    ) -> Result<Vec<CatalogGovernanceItemRecord>> {
        self.store.list_catalog_governance_items(filter, page).await
    }

    pub async fn catalog_governance_summary(&self) -> Result<AdminOverviewCatalogSummary> {
        let filter = CatalogGovernanceItemListFilter::default();
        let mut offset = 0;
        let mut summary = AdminOverviewCatalogSummary::default();

        loop {
            let page = PageRequest::new(PageRequest::MAX_LIMIT, offset);
            let records = self
                .store
                .list_catalog_governance_items(filter, page)
                .await?;
            let returned = records.len();

            for record in &records {
                summary.governed_items = summary.governed_items.saturating_add(1);

                if record.item.kind == nako_core::MediaKind::Unknown {
                    summary.unknown_kind_items = summary.unknown_kind_items.saturating_add(1);
                }

                if record
                    .best_local_inference
                    .as_ref()
                    .and_then(|evidence| evidence.confidence_milli)
                    .is_some_and(|confidence| {
                        confidence
                            <= nako_core::DEFAULT_CATALOG_GOVERNANCE_CONFIDENCE_THRESHOLD_MILLI
                    })
                {
                    summary.low_confidence_items = summary.low_confidence_items.saturating_add(1);
                }

                if record.duplicate_relationship_count > 0 {
                    summary.items_with_duplicate_relationships =
                        summary.items_with_duplicate_relationships.saturating_add(1);
                }

                if record.accepted_provider_mapping_count == 0 {
                    summary.items_missing_accepted_provider_mapping = summary
                        .items_missing_accepted_provider_mapping
                        .saturating_add(1);
                }
            }

            if returned < PageRequest::MAX_LIMIT as usize {
                return Ok(summary);
            }

            offset =
                offset
                    .checked_add(returned as u64)
                    .ok_or_else(|| NakoError::InvalidInput {
                        message: "catalog governance summary pagination offset overflowed"
                            .to_owned(),
                    })?;
        }
    }

    pub async fn get_catalog_governance_item_detail(
        &self,
        item_id: MediaItemId,
    ) -> Result<AdminCatalogGovernanceItemDetailResponse> {
        let (_item, _library_id, admin_item, provider_mappings) =
            self.catalog_governance_detail_parts(item_id).await?;

        Ok(AdminCatalogGovernanceItemDetailResponse::new(
            admin_item,
            provider_mappings,
        ))
    }

    pub async fn plan_catalog_governance_provider_mapping_review(
        &self,
        item_id: MediaItemId,
        mapping_id: ProviderMappingId,
        decision: AdminCatalogGovernanceProviderMappingReviewDecision,
    ) -> Result<AdminCatalogGovernanceProviderMappingReviewPlanResponse> {
        let (_item, _library_id, admin_item, provider_mappings) =
            self.catalog_governance_detail_parts(item_id).await?;
        let mapping = provider_mappings
            .into_iter()
            .find(|mapping| mapping.mapping_id == mapping_id)
            .ok_or_else(|| NakoError::NotFound {
                entity: "provider_mapping",
                id: mapping_id.to_string(),
            })?;

        Ok(
            AdminCatalogGovernanceProviderMappingReviewPlanResponse::new(
                AdminCatalogGovernanceProviderMappingReviewPlan::new(admin_item, mapping, decision),
            ),
        )
    }

    pub async fn review_catalog_governance_provider_mapping(
        &self,
        item_id: MediaItemId,
        mapping_id: ProviderMappingId,
        decision: AdminCatalogGovernanceProviderMappingReviewDecision,
    ) -> Result<AdminCatalogGovernanceProviderMappingReviewResponse> {
        let (_item, _library_id, admin_item, provider_mappings) =
            self.catalog_governance_detail_parts(item_id).await?;
        let mapping_summary = provider_mappings
            .into_iter()
            .find(|mapping| mapping.mapping_id == mapping_id)
            .ok_or_else(|| NakoError::NotFound {
                entity: "provider_mapping",
                id: mapping_id.to_string(),
            })?;
        let previous_status = mapping_summary.status;
        let target_status = decision.target_status();
        let mut mapping = self
            .store
            .list_provider_mappings_for_item(item_id, PageRequest::first_page())
            .await?
            .into_iter()
            .find(|mapping| mapping.id == mapping_id)
            .ok_or_else(|| NakoError::NotFound {
                entity: "provider_mapping",
                id: mapping_id.to_string(),
            })?;

        if previous_status != target_status {
            mapping.status = target_status;
            self.store.upsert_provider_mapping(&mapping).await?;
        }

        let updated_summary = self.provider_mapping_summary(mapping).await?;
        let plan = AdminCatalogGovernanceProviderMappingReviewPlan::new(
            admin_item,
            updated_summary,
            decision,
        );

        Ok(AdminCatalogGovernanceProviderMappingReviewResponse::new(
            previous_status,
            plan,
        ))
    }

    async fn catalog_governance_detail_parts(
        &self,
        item_id: MediaItemId,
    ) -> Result<(
        nako_core::MediaItem,
        nako_core::LibraryId,
        AdminCatalogGovernanceItem,
        Vec<AdminCatalogGovernanceProviderMappingSummary>,
    )> {
        let item =
            self.store
                .get_media_item(item_id)
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "media_item",
                    id: item_id.to_string(),
                })?;
        let sources = self
            .store
            .list_item_sources(item.id, PageRequest::first_page())
            .await?;
        let library_id = sources
            .first()
            .map(|source| source.library_id)
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_source",
                id: format!("item {}", item.id),
            })?;
        let sources_for_library = sources
            .into_iter()
            .filter(|source| source.library_id == library_id)
            .collect::<Vec<_>>();
        let best_local_inference =
            best_local_inference_for_sources(&self.store, &sources_for_library).await?;
        let provider_mappings = self.provider_mapping_summaries(item.id).await?;
        let accepted_provider_mapping_count = provider_mappings
            .iter()
            .filter(|mapping| mapping.status == ProviderMappingStatus::Accepted)
            .count() as u32;
        let duplicate_relationship_count = self
            .duplicate_relationship_count_for_sources(&sources_for_library)
            .await?;
        let record = catalog_governance_record_from_item_sources_and_counts(
            item.clone(),
            library_id,
            sources_for_library,
            best_local_inference,
            provider_mappings.len() as u32,
            accepted_provider_mapping_count,
            duplicate_relationship_count,
        );
        let admin_item = AdminCatalogGovernanceItem::from_record(
            record,
            nako_core::DEFAULT_CATALOG_GOVERNANCE_CONFIDENCE_THRESHOLD_MILLI,
        );

        Ok((item, library_id, admin_item, provider_mappings))
    }

    async fn provider_mapping_summaries(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<AdminCatalogGovernanceProviderMappingSummary>> {
        let mappings = self
            .store
            .list_provider_mappings_for_item(item_id, PageRequest::first_page())
            .await?;
        let mut summaries = Vec::with_capacity(mappings.len());

        for mapping in mappings {
            summaries.push(self.provider_mapping_summary(mapping).await?);
        }

        Ok(summaries)
    }

    async fn provider_mapping_summary(
        &self,
        mapping: ProviderMapping,
    ) -> Result<AdminCatalogGovernanceProviderMappingSummary> {
        let subject = self
            .store
            .get_provider_subject(mapping.subject_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "provider_subject",
                id: mapping.subject_id.to_string(),
            })?;

        Ok(
            AdminCatalogGovernanceProviderMappingSummary::from_mapping_and_subject(
                mapping, subject,
            ),
        )
    }

    async fn duplicate_relationship_count_for_sources(
        &self,
        sources: &[MediaSource],
    ) -> Result<u32> {
        let mut relationship_ids = HashSet::new();

        for source in sources {
            for relationship in self
                .store
                .list_source_duplicate_relationships(source.id, PageRequest::first_page())
                .await?
            {
                relationship_ids.insert(relationship.id);
            }
        }

        Ok(relationship_ids.len() as u32)
    }

    pub async fn record_filesystem_link_duplicate_suggestion(
        &self,
        source_id: MediaSourceId,
        duplicate_source_id: MediaSourceId,
        link_plan: &StorageLinkPlan,
    ) -> Result<SourceDuplicateRelationship> {
        if source_id == duplicate_source_id {
            return Err(NakoError::InvalidInput {
                message: "source duplicate relationship requires two distinct media sources"
                    .to_owned(),
            });
        }

        if !is_filesystem_link_duplicate_evidence_status(link_plan.status) {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "link plan status {} cannot create source duplicate evidence",
                    storage_link_plan_status_label(link_plan.status)
                ),
            });
        }

        let source = self.get_media_source_record(source_id).await?;
        let duplicate_source = self.get_media_source_record(duplicate_source_id).await?;
        validate_link_plan_matches_source_records(&source, &duplicate_source, link_plan)?;

        let relationship = SourceDuplicateRelationship {
            id: SourceDuplicateRelationshipId::new(),
            source_id,
            duplicate_source_id,
            evidence_kind: SourceDuplicateEvidenceKind::FilesystemLink,
            evidence_value: Some(filesystem_link_evidence_value(link_plan)),
            status: SourceDuplicateRelationshipStatus::Suggested,
            confidence_milli: Some(filesystem_link_duplicate_confidence_milli(link_plan.status)),
        }
        .canonicalized();

        self.store
            .upsert_source_duplicate_relationship(&relationship)
            .await?;

        Ok(relationship)
    }

    pub async fn get_item(
        &self,
        principal: &AuthenticatedPrincipal,
        item_id: MediaItemId,
    ) -> Result<ItemDetailResponse> {
        let item = self.get_accessible_item(principal, item_id).await?;
        let sources = self
            .list_accessible_item_sources(principal, item.id)
            .await?;
        let credits = self.store.list_item_credits(item.id).await?;
        let genres = self.store.list_item_genres(item.id).await?;
        let tags = self.store.list_item_tags(item.id).await?;
        let collections = self.store.list_item_collections(item.id).await?;
        let studios = self.store.list_item_studios(item.id).await?;
        let images = self.list_selected_item_image_refs(item.id).await?;

        Ok(ItemDetailResponse {
            item: media_item_to_dto(item),
            sources: sources.into_iter().map(media_source_to_dto).collect(),
            credits: credits.into_iter().map(item_credit_to_dto).collect(),
            genres: genres.into_iter().map(item_genre_to_dto).collect(),
            tags: tags.into_iter().map(item_tag_to_dto).collect(),
            collections: collections
                .into_iter()
                .map(collection_item_to_dto)
                .collect(),
            studios: studios.into_iter().map(item_studio_to_dto).collect(),
            images,
        })
    }

    pub async fn list_item_credits(
        &self,
        principal: &AuthenticatedPrincipal,
        item_id: MediaItemId,
    ) -> Result<ItemCreditsResponse> {
        let item = self.get_accessible_item(principal, item_id).await?;
        let credits = self.store.list_item_credits(item.id).await?;
        let mut people = Vec::with_capacity(credits.len());

        for credit in &credits {
            if let Some(person) = self.store.get_person(credit.person_id).await? {
                people.push(person);
            }
        }

        Ok(ItemCreditsResponse {
            item_id: item.id.to_string(),
            credits: credits.into_iter().map(item_credit_to_dto).collect(),
            people: people.into_iter().map(person_to_dto).collect(),
        })
    }

    pub async fn list_item_images(
        &self,
        principal: &AuthenticatedPrincipal,
        item_id: MediaItemId,
    ) -> Result<ImagesResponse> {
        let item = self.get_accessible_item(principal, item_id).await?;
        let images = self.list_selected_item_image_refs(item.id).await?;

        Ok(ImagesResponse {
            item_id: item.id.to_string(),
            images,
        })
    }

    async fn get_accessible_item(
        &self,
        principal: &AuthenticatedPrincipal,
        item_id: MediaItemId,
    ) -> Result<MediaItem> {
        let item =
            self.store
                .get_media_item(item_id)
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "media_item",
                    id: item_id.to_string(),
                })?;

        if principal.is_administrator() {
            return Ok(item);
        }

        let accessible = self
            .store
            .list_accessible_media_items_by_ids(principal, &[item.id])
            .await?;
        if accessible.is_empty() {
            return Err(library_browse_access_forbidden());
        }

        Ok(item)
    }

    async fn list_accessible_item_sources(
        &self,
        principal: &AuthenticatedPrincipal,
        item_id: MediaItemId,
    ) -> Result<Vec<MediaSource>> {
        let sources = self
            .store
            .list_item_sources(item_id, PageRequest::first_page())
            .await?;

        if principal.is_administrator() {
            return Ok(sources);
        }

        let mut visible_sources = Vec::with_capacity(sources.len());
        for source in sources {
            let effective = self
                .store
                .resolve_effective_library_access(principal.user_id, source.library_id)
                .await?;
            if effective.access.allows_browse() {
                visible_sources.push(source);
            }
        }

        Ok(visible_sources)
    }

    async fn list_selected_item_image_refs(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<nako_api::public_client::PublicImageRefDto>> {
        let selected = self.store.list_selected_artwork_for_item(item_id).await?;
        let mut images = Vec::with_capacity(selected.len());

        for selected in selected {
            let artifact = self
                .store
                .get_managed_artwork_artifact(selected.artifact_id)
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "managed_artwork_artifact",
                    id: selected.artifact_id.to_string(),
                })?;
            images.push(selected_artwork_to_public_image_ref(selected, artifact));
        }

        Ok(images)
    }

    pub async fn list_accessible_people(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<PeopleResponse> {
        let page = page.clamped();
        let people = self.store.list_accessible_people(principal, page).await?;

        Ok(PeopleResponse {
            page: page_info_from_request(page, people.len()),
            people: people.into_iter().map(person_to_dto).collect(),
        })
    }

    pub async fn get_person(
        &self,
        principal: &AuthenticatedPrincipal,
        person_id: PersonId,
    ) -> Result<PersonResponse> {
        let person = self.get_person_record(person_id).await?;
        if !principal.is_administrator() {
            let visible_items = self
                .store
                .list_accessible_person_items(principal, person.id, PageRequest::new(1, 0))
                .await?;
            if visible_items.is_empty() {
                return Err(library_browse_access_forbidden());
            }
        }

        Ok(PersonResponse {
            person: person_to_dto(person),
        })
    }

    async fn get_person_record(&self, person_id: PersonId) -> Result<nako_core::Person> {
        self.store
            .get_person(person_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "person",
                id: person_id.to_string(),
            })
    }

    pub async fn list_accessible_person_items(
        &self,
        principal: &AuthenticatedPrincipal,
        person_id: PersonId,
        page: PageRequest,
    ) -> Result<PersonItemsResponse> {
        let page = page.clamped();
        let person = self.get_person_record(person_id).await?;
        let items = self
            .store
            .list_accessible_person_items(principal, person.id, page)
            .await?;

        Ok(PersonItemsResponse {
            person: person_to_dto(person),
            page: page_info_from_request(page, items.len()),
            items: items.into_iter().map(media_item_to_dto).collect(),
        })
    }

    pub async fn list_accessible_tags(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<TagsResponse> {
        let page = page.clamped();
        let tags = self.store.list_accessible_tags(principal, page).await?;

        Ok(TagsResponse {
            page: page_info_from_request(page, tags.len()),
            tags: tags.into_iter().map(tag_to_dto).collect(),
        })
    }

    pub async fn list_accessible_tag_items(
        &self,
        principal: &AuthenticatedPrincipal,
        tag_id: TagId,
        page: PageRequest,
    ) -> Result<TagItemsResponse> {
        let page = page.clamped();
        let tag = self
            .store
            .get_tag(tag_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "tag",
                id: tag_id.to_string(),
            })?;
        let items = self
            .store
            .list_accessible_tag_items(principal, tag.id, page)
            .await?;

        Ok(TagItemsResponse {
            tag: tag_to_dto(tag),
            page: page_info_from_request(page, items.len()),
            items: items.into_iter().map(media_item_to_dto).collect(),
        })
    }

    pub async fn list_accessible_genres(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<GenreListResponse> {
        let page = page.clamped();
        let genres = self.store.list_accessible_genres(principal, page).await?;

        Ok(GenreListResponse {
            page: page_info_from_request(page, genres.len()),
            genres: genres.into_iter().map(genre_to_dto).collect(),
        })
    }

    pub async fn list_accessible_genre_items(
        &self,
        principal: &AuthenticatedPrincipal,
        genre_id: GenreId,
        page: PageRequest,
    ) -> Result<GenreItemsResponse> {
        let page = page.clamped();
        let genre = self
            .store
            .get_genre(genre_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "genre",
                id: genre_id.to_string(),
            })?;
        let items = self
            .store
            .list_accessible_genre_items(principal, genre.id, page)
            .await?;

        Ok(GenreItemsResponse {
            genre: genre_to_dto(genre),
            page: page_info_from_request(page, items.len()),
            items: items.into_iter().map(media_item_to_dto).collect(),
        })
    }

    pub async fn search_accessible_items(
        &self,
        principal: &AuthenticatedPrincipal,
        query: String,
        facets: Vec<String>,
        page: PageRequest,
    ) -> Result<SearchResponse> {
        let page = page.clamped();
        let search_offset = u32::try_from(page.offset).map_err(|err| NakoError::InvalidInput {
            message: format!("search pagination offset cannot fit search query offset: {err}"),
        })?;
        let search_query =
            SearchQuery::from_facet_labels(query, facets, page.limit, search_offset)?;
        let hits = self
            .store
            .search_accessible(principal, search_query)
            .await?;
        let hit_ids = hits.iter().map(|hit| hit.item_id).collect::<Vec<_>>();
        let visible_items = self
            .store
            .list_accessible_media_items_by_ids(principal, &hit_ids)
            .await?;
        let mut visible_items_by_id = visible_items
            .into_iter()
            .map(|item| (item.id, item))
            .collect::<std::collections::HashMap<_, _>>();
        let output_hits = hits
            .into_iter()
            .filter_map(|hit| {
                visible_items_by_id
                    .remove(&hit.item_id)
                    .map(|item| SearchItemHit {
                        item: media_item_to_dto(item),
                        score: hit.score,
                    })
            })
            .collect::<Vec<_>>();

        Ok(SearchResponse {
            page: page_info_from_request(page, output_hits.len()),
            hits: output_hits,
        })
    }

    pub async fn get_source_probe(
        &self,
        source_id: MediaSourceId,
    ) -> Result<nako_api::public_client::SourceProbeResponse> {
        let probe = self
            .store
            .get_media_probe(source_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_source_probe",
                id: source_id.to_string(),
            })?;

        Ok(nako_api::public_client::SourceProbeResponse {
            source_id: source_id.to_string(),
            probe: media_probe_to_dto(probe),
        })
    }

    async fn get_media_source_record(&self, source_id: MediaSourceId) -> Result<MediaSource> {
        self.store
            .get_media_source(source_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_source",
                id: source_id.to_string(),
            })
    }
}

fn library_browse_access_forbidden() -> NakoError {
    NakoError::Forbidden {
        message: "required Library Access level 'browse' is not available".to_owned(),
    }
}

async fn best_local_inference_for_sources(
    store: &NakoDatabase,
    sources: &[MediaSource],
) -> Result<Option<LocalInferenceEvidence>> {
    let mut best: Option<LocalInferenceEvidence> = None;

    for source in sources {
        for evidence in store
            .list_local_inference_evidence_for_source(source.id, PageRequest::first_page())
            .await?
        {
            if local_inference_is_better(best.as_ref(), &evidence) {
                best = Some(evidence);
            }
        }
    }

    Ok(best)
}

fn local_inference_is_better(
    current: Option<&LocalInferenceEvidence>,
    candidate: &LocalInferenceEvidence,
) -> bool {
    let Some(current) = current else {
        return true;
    };

    (
        candidate.confidence_milli.unwrap_or(0),
        &candidate.inference_version,
        candidate.id,
    ) > (
        current.confidence_milli.unwrap_or(0),
        &current.inference_version,
        current.id,
    )
}

fn is_filesystem_link_duplicate_evidence_status(status: StorageLinkPlanStatus) -> bool {
    matches!(
        status,
        StorageLinkPlanStatus::Ready | StorageLinkPlanStatus::TargetExists
    )
}

fn validate_link_plan_matches_source_records(
    source: &MediaSource,
    duplicate_source: &MediaSource,
    link_plan: &StorageLinkPlan,
) -> Result<()> {
    if link_plan.source_uri.scheme() != "local" || link_plan.target_uri.scheme() != "local" {
        return Err(NakoError::InvalidInput {
            message: "filesystem link duplicate evidence currently requires local storage URIs"
                .to_owned(),
        });
    }

    if source.locator != link_plan.source_uri.as_str() {
        return Err(NakoError::InvalidInput {
            message: format!(
                "link plan source URI does not match media source locator: {}",
                source.id
            ),
        });
    }

    if duplicate_source.locator != link_plan.target_uri.as_str() {
        return Err(NakoError::InvalidInput {
            message: format!(
                "link plan target URI does not match duplicate media source locator: {}",
                duplicate_source.id
            ),
        });
    }

    Ok(())
}

fn filesystem_link_evidence_value(link_plan: &StorageLinkPlan) -> String {
    format!(
        "link_plan:scheme={};kind={};status={}",
        link_plan.source_uri.scheme(),
        storage_link_kind_label(link_plan.kind),
        storage_link_plan_status_label(link_plan.status)
    )
}

fn filesystem_link_duplicate_confidence_milli(status: StorageLinkPlanStatus) -> u16 {
    match status {
        StorageLinkPlanStatus::TargetExists => 700,
        StorageLinkPlanStatus::Ready => 600,
        StorageLinkPlanStatus::Unsupported
        | StorageLinkPlanStatus::SourceMissing
        | StorageLinkPlanStatus::SourceNotFile
        | StorageLinkPlanStatus::TargetParentMissing
        | StorageLinkPlanStatus::TargetParentNotDirectory
        | StorageLinkPlanStatus::SecurityViolation => 0,
    }
}

fn storage_link_kind_label(kind: StorageLinkKind) -> &'static str {
    match kind {
        StorageLinkKind::Hard => "hard",
        StorageLinkKind::Soft => "soft",
    }
}

fn storage_link_plan_status_label(status: StorageLinkPlanStatus) -> &'static str {
    match status {
        StorageLinkPlanStatus::Ready => "ready",
        StorageLinkPlanStatus::Unsupported => "unsupported",
        StorageLinkPlanStatus::SourceMissing => "source_missing",
        StorageLinkPlanStatus::SourceNotFile => "source_not_file",
        StorageLinkPlanStatus::TargetParentMissing => "target_parent_missing",
        StorageLinkPlanStatus::TargetParentNotDirectory => "target_parent_not_directory",
        StorageLinkPlanStatus::TargetExists => "target_exists",
        StorageLinkPlanStatus::SecurityViolation => "security_violation",
    }
}
