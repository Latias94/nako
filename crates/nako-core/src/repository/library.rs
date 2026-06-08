use async_trait::async_trait;

use super::PageRequest;
use crate::{
    Library, LibraryId, LibraryItemBrowseQuery, LibraryItemState, MediaItem, MediaItemId,
    MediaKind, Result, UserPrincipalId,
};

#[async_trait]
pub trait LibraryRepository: Send + Sync {
    async fn upsert_library(&self, library: &Library) -> Result<()>;

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>>;

    async fn list_libraries(&self, page: PageRequest) -> Result<Vec<Library>>;
}

#[async_trait]
pub trait LibraryItemRepository: Send + Sync {
    async fn upsert_library_item_state(&self, state: &LibraryItemState) -> Result<()>;

    async fn get_library_item_state(
        &self,
        library_id: LibraryId,
        item_id: MediaItemId,
    ) -> Result<Option<LibraryItemState>>;

    async fn list_library_item_states_for_item(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<LibraryItemState>>;

    async fn list_library_items_for_browse(
        &self,
        library_id: LibraryId,
        principal_id: &UserPrincipalId,
        query: &LibraryItemBrowseQuery,
    ) -> Result<Vec<MediaItem>>;

    async fn find_library_item_by_kind_parent_title(
        &self,
        library_id: LibraryId,
        kind: MediaKind,
        parent_id: Option<MediaItemId>,
        title: &str,
    ) -> Result<Option<MediaItem>>;
}
