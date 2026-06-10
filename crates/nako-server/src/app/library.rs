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
    AuthenticatedPrincipal, IdentityAccessRepository, IngestionFailureFilter,
    IngestionFailurePhase, IngestionFailureRepository, IngestionFailureStatus, LibraryId,
    LibraryItemBrowseQuery, LibraryItemRepository, LibraryRepository, MediaRepository,
    MetadataProfileSource, NakoError, PageRequest, Result, UserPrincipalId,
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

    pub async fn list_libraries_for_browse(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<LibraryListResponse> {
        let page = page.clamped();
        let libraries = self.store.list_libraries(page).await?;
        let mut output_libraries = Vec::with_capacity(libraries.len());

        for library in libraries {
            if self
                .has_library_browse_access(principal, library.id)
                .await?
            {
                output_libraries.push(library_to_dto(library));
            }
        }

        Ok(LibraryListResponse {
            page: page_info_from_request(page, output_libraries.len()),
            libraries: output_libraries,
        })
    }

    pub async fn get_library(&self, library_id: LibraryId) -> Result<LibraryResponse> {
        Ok(LibraryResponse {
            library: library_to_dto(self.get_library_or_not_found(library_id).await?),
        })
    }

    pub async fn get_library_for_browse(
        &self,
        principal: &AuthenticatedPrincipal,
        library_id: LibraryId,
    ) -> Result<LibraryResponse> {
        self.ensure_library_browse_access(principal, library_id)
            .await?;

        self.get_library(library_id).await
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
        let entries = self
            .store
            .list_library_source_inventory(library.id, page)
            .await?;
        let output_sources = entries
            .into_iter()
            .map(|entry| LibrarySourceResponse {
                source: media_source_to_dto(entry.source),
                item: entry.item.map(media_item_to_dto),
                probe: entry.probe.map(media_probe_to_dto),
            })
            .collect::<Vec<_>>();

        Ok(LibrarySourcesResponse {
            library: library_to_dto(library),
            page: page_info_from_request(page, output_sources.len()),
            sources: output_sources,
        })
    }

    pub async fn list_library_sources_for_browse(
        &self,
        principal: &AuthenticatedPrincipal,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<LibrarySourcesResponse> {
        self.ensure_library_browse_access(principal, library_id)
            .await?;

        self.list_library_sources(library_id, page).await
    }

    pub async fn list_library_items(
        &self,
        library_id: LibraryId,
        query: LibraryItemBrowseQuery,
        principal_id: &UserPrincipalId,
    ) -> Result<LibraryItemsResponse> {
        let page = query.page.clamped();
        let library = self.get_library_or_not_found(library_id).await?;
        let mut query = query;
        query.page = page;
        let items = self
            .store
            .list_library_items_for_browse(library.id, principal_id, &query)
            .await?;
        let returned = items.len();

        Ok(LibraryItemsResponse {
            library: library_to_dto(library),
            page: page_info_from_request(page, returned),
            items: items.into_iter().map(media_item_to_dto).collect(),
        })
    }

    pub async fn list_library_items_for_browse(
        &self,
        principal: &AuthenticatedPrincipal,
        library_id: LibraryId,
        query: LibraryItemBrowseQuery,
    ) -> Result<LibraryItemsResponse> {
        if !self
            .has_library_browse_access(principal, library_id)
            .await?
        {
            return Err(library_not_found(library_id));
        }

        self.list_library_items(library_id, query, &principal.principal_id)
            .await
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

    async fn has_library_browse_access(
        &self,
        principal: &AuthenticatedPrincipal,
        library_id: LibraryId,
    ) -> Result<bool> {
        let effective = self
            .store
            .resolve_effective_library_access(principal.user_id, library_id)
            .await?;

        Ok(effective.access.allows_browse())
    }

    async fn ensure_library_browse_access(
        &self,
        principal: &AuthenticatedPrincipal,
        library_id: LibraryId,
    ) -> Result<()> {
        if self
            .has_library_browse_access(principal, library_id)
            .await?
        {
            Ok(())
        } else {
            Err(library_browse_access_forbidden())
        }
    }
}

fn library_browse_access_forbidden() -> NakoError {
    NakoError::Forbidden {
        message: "required Library Access level 'browse' is not available".to_owned(),
    }
}

fn library_not_found(library_id: LibraryId) -> NakoError {
    NakoError::NotFound {
        entity: "library",
        id: library_id.to_string(),
    }
}
