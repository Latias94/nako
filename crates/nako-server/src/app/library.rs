use std::{cmp::Ordering, collections::HashMap};

use nako_api::{
    admin::{AdminLibraryMetadataProfileResponse, AdminUpdateLibraryMetadataProfileRequest},
    admin::{IngestionFailureDiagnostic, IngestionFailuresResponse},
    public_client::{
        LibraryItemsResponse, LibraryListResponse, LibraryResponse, LibrarySourceResponse,
        LibrarySourcesResponse, library_to_dto, media_item_to_dto, media_probe_to_dto,
        media_source_to_dto, page_info_from_request,
    },
};
use nako_core::{
    IngestionFailureFilter, IngestionFailurePhase, IngestionFailureRepository,
    IngestionFailureStatus, LibraryId, LibraryItemBrowseFacet, LibraryItemBrowseQuery,
    LibraryItemBrowseSortKey, LibraryItemBrowseSortOrder, LibraryItemWatchStateFilter,
    LibraryRepository, MediaItem, MediaProbeRepository, MediaRepository, MetadataProfileSource,
    NakoError, PageRequest, Result, UserPlaybackState, UserPlaybackStateRepository,
    UserPrincipalId,
};
use nako_db::NakoDatabase;

#[derive(Clone, Debug)]
pub(crate) struct LibraryAppService {
    store: NakoDatabase,
}

impl LibraryAppService {
    pub(crate) fn new(store: NakoDatabase) -> Self {
        Self { store }
    }

    pub async fn list_libraries(&self, page: PageRequest) -> Result<LibraryListResponse> {
        let page = page.clamped();
        let libraries = self.store.list_libraries(page).await?;

        Ok(LibraryListResponse {
            page: page_info_from_request(page, libraries.len()),
            libraries: libraries.into_iter().map(library_to_dto).collect(),
        })
    }

    pub async fn get_library(&self, library_id: LibraryId) -> Result<LibraryResponse> {
        Ok(LibraryResponse {
            library: library_to_dto(self.get_library_or_not_found(library_id).await?),
        })
    }

    pub async fn get_admin_metadata_profile(
        &self,
        library_id: LibraryId,
    ) -> Result<AdminLibraryMetadataProfileResponse> {
        let library = self.get_library_or_not_found(library_id).await?;

        Ok(AdminLibraryMetadataProfileResponse::from_profile(
            library.id,
            library.options.metadata_profile,
        ))
    }

    pub async fn update_admin_metadata_profile(
        &self,
        library_id: LibraryId,
        request: AdminUpdateLibraryMetadataProfileRequest,
    ) -> Result<AdminLibraryMetadataProfileResponse> {
        let mut library = self.get_library_or_not_found(library_id).await?;
        library.options.metadata_profile = request.profile;
        library.options.metadata_profile_source = MetadataProfileSource::Admin;
        self.store.upsert_library(&library).await?;

        Ok(AdminLibraryMetadataProfileResponse::from_profile(
            library.id,
            library.options.metadata_profile,
        ))
    }

    pub async fn list_library_sources(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<LibrarySourcesResponse> {
        let page = page.clamped();
        let library = self.get_library_or_not_found(library_id).await?;
        let sources = self.store.list_media_sources(library.id, page).await?;
        let mut output_sources = Vec::with_capacity(sources.len());

        for source in sources {
            let item = self.store.get_media_item(source.item_id).await?;
            let probe = self.store.get_media_probe(source.id).await?;
            output_sources.push(LibrarySourceResponse {
                source: media_source_to_dto(source),
                item: item.map(media_item_to_dto),
                probe: probe.map(media_probe_to_dto),
            });
        }

        Ok(LibrarySourcesResponse {
            library: library_to_dto(library),
            page: page_info_from_request(page, output_sources.len()),
            sources: output_sources,
        })
    }

    pub async fn list_library_items(
        &self,
        library_id: LibraryId,
        query: LibraryItemBrowseQuery,
        principal_id: &UserPrincipalId,
    ) -> Result<LibraryItemsResponse> {
        let page = query.page.clamped();
        let library = self.get_library_or_not_found(library_id).await?;
        let items = self
            .filtered_library_items(library.id, principal_id, &query)
            .await?;
        let returned = items.len();

        Ok(LibraryItemsResponse {
            library: library_to_dto(library),
            page: page_info_from_request(page, returned),
            items: items.into_iter().map(media_item_to_dto).collect(),
        })
    }

    async fn filtered_library_items(
        &self,
        library_id: LibraryId,
        principal_id: &UserPrincipalId,
        query: &LibraryItemBrowseQuery,
    ) -> Result<Vec<MediaItem>> {
        let added_at_by_item = self
            .store
            .list_library_item_added_at(library_id)
            .await?
            .into_iter()
            .map(|fact| (fact.item_id, fact.added_at))
            .collect::<HashMap<_, _>>();
        let mut rows = Vec::new();
        for item in self.all_library_items(library_id).await? {
            if !item_matches_facets(&item, &query.facets) {
                continue;
            }
            let state = self
                .store
                .get_user_playback_state(principal_id, item.id)
                .await?;
            if !item_matches_watch_state(state.as_ref(), query.watch_state) {
                continue;
            }
            let added_at = added_at_by_item.get(&item.id).cloned();
            rows.push(LibraryItemBrowseRow {
                item,
                state,
                added_at,
            });
        }

        rows.sort_by(|left, right| compare_library_item_rows(left, right, query));

        let start = usize::try_from(query.page.offset).unwrap_or(usize::MAX);
        if start >= rows.len() {
            return Ok(Vec::new());
        }
        let limit = usize::try_from(query.page.limit).unwrap_or(usize::MAX);
        let end = start.saturating_add(limit).min(rows.len());

        Ok(rows[start..end]
            .iter()
            .map(|row| row.item.clone())
            .collect())
    }

    async fn all_library_items(&self, library_id: LibraryId) -> Result<Vec<MediaItem>> {
        let mut offset = 0;
        let mut items = Vec::new();
        loop {
            let page = PageRequest::new(PageRequest::MAX_LIMIT, offset);
            let chunk = self
                .store
                .list_media_items_for_library(library_id, page)
                .await?;
            let count = chunk.len();
            items.extend(chunk);
            if count < PageRequest::MAX_LIMIT as usize {
                break;
            }
            offset += u64::from(PageRequest::MAX_LIMIT);
        }
        Ok(items)
    }

    pub async fn list_ingestion_failures(
        &self,
        library_id: LibraryId,
        phase: Option<IngestionFailurePhase>,
        status: Option<IngestionFailureStatus>,
        page: PageRequest,
    ) -> Result<IngestionFailuresResponse> {
        let page = page.clamped();
        self.get_library_or_not_found(library_id).await?;
        let failures = self
            .store
            .list_ingestion_failures(
                IngestionFailureFilter {
                    library_id: Some(library_id),
                    phase,
                    status,
                },
                page,
            )
            .await?;
        let output = failures
            .into_iter()
            .map(IngestionFailureDiagnostic::from_record)
            .collect::<Vec<_>>();

        Ok(IngestionFailuresResponse {
            library_id,
            page: page_info_from_request(page, output.len()),
            failures: output,
        })
    }

    pub async fn ignore_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
    ) -> Result<IngestionFailureDiagnostic> {
        self.get_library_or_not_found(library_id).await?;
        let record = self
            .store
            .ignore_ingestion_failure(library_id, phase, target_uri, super::current_time_ms()?)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "ingestion_failure",
                id: format!("{}:{}:{target_uri}", library_id, phase.as_str()),
            })?;

        Ok(IngestionFailureDiagnostic::from_record(record))
    }

    async fn get_library_or_not_found(&self, library_id: LibraryId) -> Result<nako_core::Library> {
        self.store
            .get_library(library_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
    }
}

struct LibraryItemBrowseRow {
    item: MediaItem,
    state: Option<UserPlaybackState>,
    added_at: Option<String>,
}

fn item_matches_facets(item: &MediaItem, facets: &[LibraryItemBrowseFacet]) -> bool {
    facets.iter().all(|facet| match facet {
        LibraryItemBrowseFacet::Kind(kind) => item.kind == *kind,
    })
}

fn item_matches_watch_state(
    state: Option<&UserPlaybackState>,
    watch_state: LibraryItemWatchStateFilter,
) -> bool {
    match watch_state {
        LibraryItemWatchStateFilter::Any => true,
        LibraryItemWatchStateFilter::Watched => state.is_some_and(|state| state.watched),
        LibraryItemWatchStateFilter::Unwatched => state.is_none_or(|state| !state.watched),
        LibraryItemWatchStateFilter::InProgress => state.is_some_and(|state| {
            !state.watched
                && state
                    .resume_position_ms
                    .is_some_and(|position| position > 0)
        }),
    }
}

fn compare_library_item_rows(
    left: &LibraryItemBrowseRow,
    right: &LibraryItemBrowseRow,
    query: &LibraryItemBrowseQuery,
) -> Ordering {
    let base = match query.sort {
        LibraryItemBrowseSortKey::Title => compare_optional_str(
            Some(sortable_title(&left.item)),
            Some(sortable_title(&right.item)),
            query.order,
        ),
        LibraryItemBrowseSortKey::ReleaseDate => compare_optional_str(
            left.item.metadata.release_date.as_deref(),
            right.item.metadata.release_date.as_deref(),
            query.order,
        ),
        LibraryItemBrowseSortKey::DateAdded => compare_optional_str(
            left.added_at.as_deref(),
            right.added_at.as_deref(),
            query.order,
        ),
        LibraryItemBrowseSortKey::LastPlayed => compare_optional_i64(
            left.state
                .as_ref()
                .and_then(|state| state.last_played_at_ms),
            right
                .state
                .as_ref()
                .and_then(|state| state.last_played_at_ms),
            query.order,
        ),
    };

    base.then_with(|| left.item.id.cmp(&right.item.id))
}

fn sortable_title(item: &MediaItem) -> &str {
    item.metadata
        .sort_title
        .as_deref()
        .unwrap_or(item.metadata.title.as_str())
}

fn compare_optional_str(
    left: Option<&str>,
    right: Option<&str>,
    order: LibraryItemBrowseSortOrder,
) -> Ordering {
    compare_option_values(left, right, order, |left, right| left.cmp(right))
}

fn compare_optional_i64(
    left: Option<i64>,
    right: Option<i64>,
    order: LibraryItemBrowseSortOrder,
) -> Ordering {
    compare_option_values(left, right, order, Ord::cmp)
}

fn compare_option_values<T>(
    left: Option<T>,
    right: Option<T>,
    order: LibraryItemBrowseSortOrder,
    compare: impl Fn(&T, &T) -> Ordering,
) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => match order {
            LibraryItemBrowseSortOrder::Asc => compare(&left, &right),
            LibraryItemBrowseSortOrder::Desc => compare(&right, &left),
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}
