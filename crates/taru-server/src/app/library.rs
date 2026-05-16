use taru_api::{
    IngestionFailureDiagnostic, IngestionFailuresResponse, LibraryListResponse,
    LibrarySourceResponse, LibrarySourcesResponse, PageInfo,
};
use taru_core::{
    IngestionFailureFilter, IngestionFailurePhase, IngestionFailureRepository,
    IngestionFailureStatus, LibraryId, LibraryRepository, MediaProbeRepository, MediaRepository,
    PageRequest, Result, TaruError,
};

use super::TaruApp;

impl TaruApp {
    pub async fn list_libraries(&self, page: PageRequest) -> Result<LibraryListResponse> {
        let page = page.clamped();
        let libraries = self.inner.store.list_libraries(page).await?;

        Ok(LibraryListResponse {
            page: PageInfo::new(page, libraries.len()),
            libraries,
        })
    }

    pub async fn list_library_sources(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<LibrarySourcesResponse> {
        let page = page.clamped();
        let library = self.get_library_or_not_found(library_id).await?;
        let sources = self
            .inner
            .store
            .list_media_sources(library.id, page)
            .await?;
        let mut output_sources = Vec::with_capacity(sources.len());

        for source in sources {
            let item = self.inner.store.get_media_item(source.item_id).await?;
            let probe = self.inner.store.get_media_probe(source.id).await?;
            output_sources.push(LibrarySourceResponse {
                source,
                item,
                probe,
            });
        }

        Ok(LibrarySourcesResponse {
            library,
            page: PageInfo::new(page, output_sources.len()),
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
            .inner
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
            page: PageInfo::new(page, output.len()),
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
            .inner
            .store
            .ignore_ingestion_failure(library_id, phase, target_uri, super::current_time_ms()?)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "ingestion_failure",
                id: format!("{}:{}:{target_uri}", library_id, phase.as_str()),
            })?;

        Ok(IngestionFailureDiagnostic::from_record(record))
    }
}
