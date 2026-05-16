use taru_api::{
    IngestionFailureDiagnostic, IngestionFailuresResponse, LibraryListResponse,
    LibrarySourceResponse, LibrarySourcesResponse, page_info_from_request,
};
use taru_core::{
    IngestionFailureFilter, IngestionFailurePhase, IngestionFailureRepository,
    IngestionFailureStatus, LibraryId, LibraryRepository, MediaProbeRepository, MediaRepository,
    PageRequest, Result, TaruError,
};
use taru_db::SqliteStore;

#[derive(Clone, Debug)]
pub(crate) struct LibraryAppService {
    store: SqliteStore,
}

impl LibraryAppService {
    pub(crate) fn new(store: SqliteStore) -> Self {
        Self { store }
    }

    pub async fn list_libraries(&self, page: PageRequest) -> Result<LibraryListResponse> {
        let page = page.clamped();
        let libraries = self.store.list_libraries(page).await?;

        Ok(LibraryListResponse {
            page: page_info_from_request(page, libraries.len()),
            libraries: libraries.into_iter().map(Into::into).collect(),
        })
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
                source: source.into(),
                item: item.map(Into::into),
                probe: probe.map(Into::into),
            });
        }

        Ok(LibrarySourcesResponse {
            library: library.into(),
            page: page_info_from_request(page, output_sources.len()),
            sources: output_sources,
        })
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
            .ok_or_else(|| TaruError::NotFound {
                entity: "ingestion_failure",
                id: format!("{}:{}:{target_uri}", library_id, phase.as_str()),
            })?;

        Ok(IngestionFailureDiagnostic::from_record(record))
    }

    async fn get_library_or_not_found(&self, library_id: LibraryId) -> Result<taru_core::Library> {
        self.store
            .get_library(library_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
    }
}
