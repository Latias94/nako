use std::collections::BTreeSet;

use async_trait::async_trait;
use nako_core::{
    AuthenticatedPrincipal, MediaItem, MediaItemId, MediaRepository, NakoError, NewUserPlaylist,
    PageRequest, Result, UserPlaylistId, UserPlaylistItemRecord, UserPlaylistItemRemoval,
    UserPlaylistItemWrite, UserPlaylistItemsProjection, UserPlaylistNameUpdate, UserPlaylistRecord,
    UserPlaylistReorder, UserPlaylistRepository, UserPlaylistSummaryProjection, UserPrincipalId,
};
use nako_db::NakoDatabase;

use super::current_time_ms;

const MAX_USER_PLAYLIST_NAME_LEN: usize = 200;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CreateUserPlaylistRequest {
    pub principal_id: UserPrincipalId,
    pub name: String,
    pub created_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenameUserPlaylistRequest {
    pub principal_id: UserPrincipalId,
    pub playlist_id: UserPlaylistId,
    pub name: String,
    pub expected_version: Option<u64>,
    pub updated_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AddUserPlaylistItemRequest {
    pub principal_id: UserPrincipalId,
    pub playlist_id: UserPlaylistId,
    pub item_id: MediaItemId,
    pub position: Option<u32>,
    pub expected_version: Option<u64>,
    pub added_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RemoveUserPlaylistItemRequest {
    pub principal_id: UserPrincipalId,
    pub playlist_id: UserPlaylistId,
    pub item_id: MediaItemId,
    pub expected_version: Option<u64>,
    pub updated_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReorderUserPlaylistItemsRequest {
    pub principal_id: UserPrincipalId,
    pub playlist_id: UserPlaylistId,
    pub item_ids: Vec<MediaItemId>,
    pub expected_version: Option<u64>,
    pub updated_at_ms: Option<i64>,
}

#[async_trait]
pub(crate) trait UserPlaylistStore: Clone + Send + Sync + std::fmt::Debug {
    async fn create_user_playlist_record(
        &self,
        playlist: NewUserPlaylist,
    ) -> Result<UserPlaylistRecord>;

    async fn load_user_playlist(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<Option<UserPlaylistRecord>>;

    async fn list_user_playlist_records(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistRecord>>;

    async fn update_user_playlist_name_record(
        &self,
        update: UserPlaylistNameUpdate,
    ) -> Result<Option<UserPlaylistRecord>>;

    async fn delete_user_playlist_record(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<bool>;

    async fn add_user_playlist_item_record(
        &self,
        write: UserPlaylistItemWrite,
    ) -> Result<Option<UserPlaylistRecord>>;

    async fn remove_user_playlist_item_record(
        &self,
        removal: UserPlaylistItemRemoval,
    ) -> Result<Option<UserPlaylistRecord>>;

    async fn replace_user_playlist_item_order_record(
        &self,
        reorder: UserPlaylistReorder,
    ) -> Result<Option<UserPlaylistRecord>>;

    async fn list_user_playlist_item_records(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistItemRecord>>;

    async fn load_user_playlist_items_projection(
        &self,
        principal: &AuthenticatedPrincipal,
        playlist_id: UserPlaylistId,
        page: PageRequest,
    ) -> Result<Option<UserPlaylistItemsProjection>>;

    async fn load_user_playlist_summary_projection(
        &self,
        principal: &AuthenticatedPrincipal,
        playlist_id: UserPlaylistId,
    ) -> Result<Option<UserPlaylistSummaryProjection>>;

    async fn list_user_playlist_summary_projections(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistSummaryProjection>>;

    async fn load_media_item(&self, item_id: MediaItemId) -> Result<Option<MediaItem>>;
}

#[async_trait]
impl UserPlaylistStore for NakoDatabase {
    async fn create_user_playlist_record(
        &self,
        playlist: NewUserPlaylist,
    ) -> Result<UserPlaylistRecord> {
        UserPlaylistRepository::create_user_playlist(self, playlist).await
    }

    async fn load_user_playlist(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<Option<UserPlaylistRecord>> {
        UserPlaylistRepository::get_user_playlist(self, principal_id, playlist_id).await
    }

    async fn list_user_playlist_records(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistRecord>> {
        UserPlaylistRepository::list_user_playlists(self, principal_id, page).await
    }

    async fn update_user_playlist_name_record(
        &self,
        update: UserPlaylistNameUpdate,
    ) -> Result<Option<UserPlaylistRecord>> {
        UserPlaylistRepository::update_user_playlist_name(self, update).await
    }

    async fn delete_user_playlist_record(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<bool> {
        UserPlaylistRepository::delete_user_playlist(self, principal_id, playlist_id).await
    }

    async fn add_user_playlist_item_record(
        &self,
        write: UserPlaylistItemWrite,
    ) -> Result<Option<UserPlaylistRecord>> {
        UserPlaylistRepository::add_user_playlist_item(self, write).await
    }

    async fn remove_user_playlist_item_record(
        &self,
        removal: UserPlaylistItemRemoval,
    ) -> Result<Option<UserPlaylistRecord>> {
        UserPlaylistRepository::remove_user_playlist_item(self, removal).await
    }

    async fn replace_user_playlist_item_order_record(
        &self,
        reorder: UserPlaylistReorder,
    ) -> Result<Option<UserPlaylistRecord>> {
        UserPlaylistRepository::replace_user_playlist_item_order(self, reorder).await
    }

    async fn list_user_playlist_item_records(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistItemRecord>> {
        UserPlaylistRepository::list_user_playlist_items(self, principal_id, playlist_id, page)
            .await
    }

    async fn load_user_playlist_items_projection(
        &self,
        principal: &AuthenticatedPrincipal,
        playlist_id: UserPlaylistId,
        page: PageRequest,
    ) -> Result<Option<UserPlaylistItemsProjection>> {
        UserPlaylistRepository::get_user_playlist_items_projection(
            self,
            principal,
            playlist_id,
            page,
        )
        .await
    }

    async fn load_user_playlist_summary_projection(
        &self,
        principal: &AuthenticatedPrincipal,
        playlist_id: UserPlaylistId,
    ) -> Result<Option<UserPlaylistSummaryProjection>> {
        UserPlaylistRepository::get_user_playlist_summary_projection(self, principal, playlist_id)
            .await
    }

    async fn list_user_playlist_summary_projections(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistSummaryProjection>> {
        UserPlaylistRepository::list_user_playlist_summary_projections(self, principal, page).await
    }

    async fn load_media_item(&self, item_id: MediaItemId) -> Result<Option<MediaItem>> {
        MediaRepository::get_media_item(self, item_id).await
    }
}

#[derive(Clone, Debug)]
pub(crate) struct UserPlaylistAppService<S = NakoDatabase> {
    store: S,
}

impl<S> UserPlaylistAppService<S>
where
    S: UserPlaylistStore,
{
    pub(crate) fn new(store: S) -> Self {
        Self { store }
    }

    pub(crate) async fn create_playlist(
        &self,
        request: CreateUserPlaylistRequest,
    ) -> Result<UserPlaylistRecord> {
        let name = normalize_playlist_name(request.name)?;
        let now_ms = request.created_at_ms.unwrap_or(current_time_ms()?);

        self.store
            .create_user_playlist_record(NewUserPlaylist {
                id: UserPlaylistId::new(),
                principal_id: request.principal_id,
                name,
                created_at_ms: now_ms,
            })
            .await
    }

    pub(crate) async fn get_playlist(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<UserPlaylistRecord> {
        self.store
            .load_user_playlist(principal_id, playlist_id)
            .await?
            .ok_or_else(|| playlist_not_found(playlist_id))
    }

    pub(crate) async fn list_playlists(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistRecord>> {
        self.store
            .list_user_playlist_records(principal_id, page)
            .await
    }

    pub(crate) async fn get_playlist_summary(
        &self,
        principal: &AuthenticatedPrincipal,
        playlist_id: UserPlaylistId,
    ) -> Result<UserPlaylistSummaryProjection> {
        self.store
            .load_user_playlist_summary_projection(principal, playlist_id)
            .await?
            .ok_or_else(|| playlist_not_found(playlist_id))
    }

    pub(crate) async fn list_playlist_summaries(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistSummaryProjection>> {
        self.store
            .list_user_playlist_summary_projections(principal, page)
            .await
    }

    pub(crate) async fn rename_playlist(
        &self,
        request: RenameUserPlaylistRequest,
    ) -> Result<UserPlaylistRecord> {
        let playlist = self
            .get_playlist(&request.principal_id, request.playlist_id)
            .await?;
        ensure_expected_version(&playlist, request.expected_version)?;
        let name = normalize_playlist_name(request.name)?;
        let updated_at_ms = request.updated_at_ms.unwrap_or(current_time_ms()?);
        let result = self
            .store
            .update_user_playlist_name_record(UserPlaylistNameUpdate {
                playlist_id: request.playlist_id,
                principal_id: request.principal_id.clone(),
                name,
                expected_version: request.expected_version,
                updated_at_ms,
            })
            .await?;

        self.expect_mutation_result(&request.principal_id, request.playlist_id, result)
            .await
    }

    pub(crate) async fn delete_playlist(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<()> {
        if self
            .store
            .delete_user_playlist_record(principal_id, playlist_id)
            .await?
        {
            Ok(())
        } else {
            Err(playlist_not_found(playlist_id))
        }
    }

    pub(crate) async fn add_item(
        &self,
        request: AddUserPlaylistItemRequest,
    ) -> Result<UserPlaylistRecord> {
        let playlist = self
            .get_playlist(&request.principal_id, request.playlist_id)
            .await?;
        ensure_expected_version(&playlist, request.expected_version)?;
        self.ensure_item_exists(request.item_id).await?;
        let event_at_ms = request.added_at_ms.unwrap_or(current_time_ms()?);
        let result = self
            .store
            .add_user_playlist_item_record(UserPlaylistItemWrite {
                playlist_id: request.playlist_id,
                principal_id: request.principal_id.clone(),
                item_id: request.item_id,
                position: request.position,
                expected_version: request.expected_version,
                added_at_ms: event_at_ms,
                updated_at_ms: event_at_ms,
            })
            .await?;

        self.expect_mutation_result(&request.principal_id, request.playlist_id, result)
            .await
    }

    pub(crate) async fn remove_item(
        &self,
        request: RemoveUserPlaylistItemRequest,
    ) -> Result<UserPlaylistRecord> {
        let playlist = self
            .get_playlist(&request.principal_id, request.playlist_id)
            .await?;
        ensure_expected_version(&playlist, request.expected_version)?;
        let updated_at_ms = request.updated_at_ms.unwrap_or(current_time_ms()?);
        let result = self
            .store
            .remove_user_playlist_item_record(UserPlaylistItemRemoval {
                playlist_id: request.playlist_id,
                principal_id: request.principal_id.clone(),
                item_id: request.item_id,
                expected_version: request.expected_version,
                updated_at_ms,
            })
            .await?;

        self.expect_mutation_result(&request.principal_id, request.playlist_id, result)
            .await
    }

    pub(crate) async fn reorder_items(
        &self,
        request: ReorderUserPlaylistItemsRequest,
    ) -> Result<UserPlaylistRecord> {
        let playlist = self
            .get_playlist(&request.principal_id, request.playlist_id)
            .await?;
        ensure_expected_version(&playlist, request.expected_version)?;
        let current_items = self
            .list_all_items(&request.principal_id, request.playlist_id)
            .await?;
        validate_reorder_items(&current_items, &request.item_ids)?;
        let updated_at_ms = request.updated_at_ms.unwrap_or(current_time_ms()?);
        let result = self
            .store
            .replace_user_playlist_item_order_record(UserPlaylistReorder {
                playlist_id: request.playlist_id,
                principal_id: request.principal_id.clone(),
                item_ids: request.item_ids,
                expected_version: request.expected_version,
                updated_at_ms,
            })
            .await?;

        self.expect_mutation_result(&request.principal_id, request.playlist_id, result)
            .await
    }

    pub(crate) async fn list_items(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistItemRecord>> {
        self.get_playlist(principal_id, playlist_id).await?;
        self.store
            .list_user_playlist_item_records(principal_id, playlist_id, page)
            .await
    }

    pub(crate) async fn get_items_projection(
        &self,
        principal: &AuthenticatedPrincipal,
        playlist_id: UserPlaylistId,
        page: PageRequest,
    ) -> Result<UserPlaylistItemsProjection> {
        self.store
            .load_user_playlist_items_projection(principal, playlist_id, page)
            .await?
            .ok_or_else(|| playlist_not_found(playlist_id))
    }

    async fn ensure_item_exists(&self, item_id: MediaItemId) -> Result<()> {
        self.store
            .load_media_item(item_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })
            .map(|_| ())
    }

    async fn expect_mutation_result(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
        result: Option<UserPlaylistRecord>,
    ) -> Result<UserPlaylistRecord> {
        if let Some(record) = result {
            return Ok(record);
        }

        if self
            .store
            .load_user_playlist(principal_id, playlist_id)
            .await?
            .is_some()
        {
            Err(NakoError::Conflict {
                message: format!("user playlist {playlist_id} version changed"),
            })
        } else {
            Err(playlist_not_found(playlist_id))
        }
    }

    async fn list_all_items(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<Vec<UserPlaylistItemRecord>> {
        let mut items = Vec::new();
        let mut offset = 0;
        loop {
            let page = self
                .store
                .list_user_playlist_item_records(
                    principal_id,
                    playlist_id,
                    PageRequest::new(PageRequest::MAX_LIMIT, offset),
                )
                .await?;
            let page_len = page.len();
            items.extend(page);
            if page_len < PageRequest::MAX_LIMIT as usize {
                break;
            }
            offset += u64::from(PageRequest::MAX_LIMIT);
        }

        Ok(items)
    }
}

fn normalize_playlist_name(name: String) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "user playlist name cannot be empty".to_owned(),
        });
    }
    if name.len() > MAX_USER_PLAYLIST_NAME_LEN {
        return Err(NakoError::InvalidInput {
            message: format!("user playlist name cannot exceed {MAX_USER_PLAYLIST_NAME_LEN} bytes"),
        });
    }
    if name.chars().any(char::is_control) {
        return Err(NakoError::InvalidInput {
            message: "user playlist name cannot contain control characters".to_owned(),
        });
    }

    Ok(name.to_owned())
}

fn ensure_expected_version(
    playlist: &UserPlaylistRecord,
    expected_version: Option<u64>,
) -> Result<()> {
    if let Some(expected_version) = expected_version
        && playlist.version != expected_version
    {
        return Err(NakoError::Conflict {
            message: format!(
                "user playlist {} expected version {} but current version is {}",
                playlist.id, expected_version, playlist.version
            ),
        });
    }

    Ok(())
}

fn validate_reorder_items(
    current_items: &[UserPlaylistItemRecord],
    item_ids: &[MediaItemId],
) -> Result<()> {
    if current_items.len() != item_ids.len() {
        return Err(NakoError::InvalidInput {
            message: "playlist reorder must include every existing item exactly once".to_owned(),
        });
    }
    let unique = item_ids.iter().copied().collect::<BTreeSet<_>>();
    if unique.len() != item_ids.len() {
        return Err(NakoError::InvalidInput {
            message: "playlist reorder cannot contain duplicate item ids".to_owned(),
        });
    }
    for item_id in item_ids {
        if !current_items.iter().any(|item| item.item_id == *item_id) {
            return Err(NakoError::InvalidInput {
                message: format!("playlist reorder contains foreign item id {item_id}"),
            });
        }
    }

    Ok(())
}

fn playlist_not_found(playlist_id: UserPlaylistId) -> NakoError {
    NakoError::NotFound {
        entity: "user_playlist",
        id: playlist_id.to_string(),
    }
}
