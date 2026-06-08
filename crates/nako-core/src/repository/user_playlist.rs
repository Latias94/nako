use async_trait::async_trait;

use super::PageRequest;
use crate::{
    AuthenticatedPrincipal, ManagedArtworkArtifactRecord, MediaItem, Result, SelectedArtworkRecord,
    UserPlaylistId, UserPlaylistItemRecord, UserPlaylistItemRemoval, UserPlaylistItemWrite,
    UserPlaylistNameUpdate, UserPlaylistRecord, UserPlaylistReorder, UserPrincipalId,
};

#[derive(Clone, Debug, PartialEq)]
pub struct UserPlaylistItemsProjection {
    pub playlist: UserPlaylistRecord,
    pub accessible_item_count: u32,
    pub items: Vec<UserPlaylistItemEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserPlaylistSummaryProjection {
    pub playlist: UserPlaylistRecord,
    pub accessible_item_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserPlaylistItemEntry {
    pub playlist_item: UserPlaylistItemRecord,
    pub item: MediaItem,
    pub images: Vec<UserPlaylistImageEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserPlaylistImageEntry {
    pub selected: SelectedArtworkRecord,
    pub artifact: ManagedArtworkArtifactRecord,
}

#[async_trait]
pub trait UserPlaylistRepository: Send + Sync {
    async fn create_user_playlist(
        &self,
        playlist: crate::NewUserPlaylist,
    ) -> Result<UserPlaylistRecord>;

    async fn get_user_playlist(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<Option<UserPlaylistRecord>>;

    async fn list_user_playlists(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistRecord>>;

    async fn get_user_playlist_summary_projection(
        &self,
        principal: &AuthenticatedPrincipal,
        playlist_id: UserPlaylistId,
    ) -> Result<Option<UserPlaylistSummaryProjection>>;

    async fn list_user_playlist_summary_projections(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistSummaryProjection>>;

    async fn update_user_playlist_name(
        &self,
        update: UserPlaylistNameUpdate,
    ) -> Result<Option<UserPlaylistRecord>>;

    async fn delete_user_playlist(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<bool>;

    async fn add_user_playlist_item(
        &self,
        write: UserPlaylistItemWrite,
    ) -> Result<Option<UserPlaylistRecord>>;

    async fn remove_user_playlist_item(
        &self,
        removal: UserPlaylistItemRemoval,
    ) -> Result<Option<UserPlaylistRecord>>;

    async fn replace_user_playlist_item_order(
        &self,
        reorder: UserPlaylistReorder,
    ) -> Result<Option<UserPlaylistRecord>>;

    async fn list_user_playlist_items(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistItemRecord>>;

    async fn get_user_playlist_items_projection(
        &self,
        principal: &AuthenticatedPrincipal,
        playlist_id: UserPlaylistId,
        page: PageRequest,
    ) -> Result<Option<UserPlaylistItemsProjection>>;
}
