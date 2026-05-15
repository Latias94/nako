use taru_api::{LibraryListResponse, LibrarySourceResponse, LibrarySourcesResponse, PageInfo};
use taru_core::{
    LibraryId, LibraryRepository, MediaProbeRepository, MediaRepository, PageRequest, Result,
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
}
