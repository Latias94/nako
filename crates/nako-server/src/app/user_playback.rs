use async_trait::async_trait;
use nako_core::{
    AuthenticatedPrincipal, ContinueWatchingEntry, MediaItemId, MediaRepository, MediaSourceId,
    NakoError, Result, UserPlaybackState, UserPlaybackStateRepository, UserPlaybackStateWrite,
    UserPrincipalId,
};
use nako_db::NakoDatabase;

use super::current_time_ms;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UpdateUserPlaybackProgressRequest {
    pub principal_id: UserPrincipalId,
    pub item_id: MediaItemId,
    pub source_id: Option<MediaSourceId>,
    pub position_ms: u64,
    pub duration_ms: Option<u64>,
    pub reported_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SetUserWatchedStateRequest {
    pub principal_id: UserPrincipalId,
    pub item_id: MediaItemId,
    pub watched: bool,
    pub source_id: Option<MediaSourceId>,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub marked_at_ms: Option<i64>,
}

#[async_trait]
pub(crate) trait UserPlaybackStore: Clone + Send + Sync + std::fmt::Debug {
    async fn load_user_playback_state(
        &self,
        principal_id: &UserPrincipalId,
        item_id: MediaItemId,
    ) -> Result<Option<UserPlaybackState>>;

    async fn store_user_playback_state(
        &self,
        write: UserPlaybackStateWrite,
    ) -> Result<UserPlaybackState>;

    async fn list_continue_watching_user_playback_states(
        &self,
        principal_id: &UserPrincipalId,
        page: nako_core::PageRequest,
    ) -> Result<Vec<UserPlaybackState>>;

    async fn list_continue_watching_entries(
        &self,
        principal: &AuthenticatedPrincipal,
        page: nako_core::PageRequest,
    ) -> Result<Vec<ContinueWatchingEntry>>;

    async fn load_media_item(&self, item_id: MediaItemId) -> Result<Option<nako_core::MediaItem>>;

    async fn load_media_source(
        &self,
        source_id: MediaSourceId,
    ) -> Result<Option<nako_core::MediaSource>>;
}

#[async_trait]
impl UserPlaybackStore for NakoDatabase {
    async fn load_user_playback_state(
        &self,
        principal_id: &UserPrincipalId,
        item_id: MediaItemId,
    ) -> Result<Option<UserPlaybackState>> {
        UserPlaybackStateRepository::get_user_playback_state(self, principal_id, item_id).await
    }

    async fn store_user_playback_state(
        &self,
        write: UserPlaybackStateWrite,
    ) -> Result<UserPlaybackState> {
        UserPlaybackStateRepository::upsert_user_playback_state(self, write).await
    }

    async fn list_continue_watching_user_playback_states(
        &self,
        principal_id: &UserPrincipalId,
        page: nako_core::PageRequest,
    ) -> Result<Vec<UserPlaybackState>> {
        UserPlaybackStateRepository::list_continue_watching_states(self, principal_id, page).await
    }

    async fn list_continue_watching_entries(
        &self,
        principal: &AuthenticatedPrincipal,
        page: nako_core::PageRequest,
    ) -> Result<Vec<ContinueWatchingEntry>> {
        UserPlaybackStateRepository::list_continue_watching_entries(self, principal, page).await
    }

    async fn load_media_item(&self, item_id: MediaItemId) -> Result<Option<nako_core::MediaItem>> {
        MediaRepository::get_media_item(self, item_id).await
    }

    async fn load_media_source(
        &self,
        source_id: MediaSourceId,
    ) -> Result<Option<nako_core::MediaSource>> {
        MediaRepository::get_media_source(self, source_id).await
    }
}

#[derive(Clone, Debug)]
pub(crate) struct UserPlaybackAppService<S = NakoDatabase> {
    store: S,
}

impl<S> UserPlaybackAppService<S>
where
    S: UserPlaybackStore,
{
    pub(crate) fn new(store: S) -> Self {
        Self { store }
    }

    pub(crate) async fn get_state(
        &self,
        principal_id: &UserPrincipalId,
        item_id: MediaItemId,
    ) -> Result<UserPlaybackState> {
        self.ensure_item_exists(item_id).await?;

        Ok(self
            .store
            .load_user_playback_state(principal_id, item_id)
            .await?
            .unwrap_or_else(|| default_user_playback_state(principal_id.clone(), item_id)))
    }

    pub(crate) async fn update_progress(
        &self,
        request: UpdateUserPlaybackProgressRequest,
    ) -> Result<UserPlaybackState> {
        self.ensure_item_exists(request.item_id).await?;
        if let Some(source_id) = request.source_id {
            self.ensure_source_belongs_to_item(source_id, request.item_id)
                .await?;
        }

        let event_at_ms = request.reported_at_ms.unwrap_or(current_time_ms()?);
        let existing = self
            .store
            .load_user_playback_state(&request.principal_id, request.item_id)
            .await?;
        if let Some(existing) = &existing {
            if existing.updated_at_ms > event_at_ms {
                return Ok(existing.clone());
            }
        }

        let watched_by_policy = is_watched_by_policy(request.position_ms, request.duration_ms);
        let already_watched = existing.as_ref().is_some_and(|state| state.watched);
        let watched = already_watched || watched_by_policy;
        let resume_position_ms = if watched || request.position_ms == 0 {
            None
        } else {
            Some(request.position_ms)
        };
        let watched_at_ms = if already_watched {
            existing.as_ref().and_then(|state| state.watched_at_ms)
        } else {
            watched_by_policy.then_some(event_at_ms)
        };
        let last_played_at_ms = (request.position_ms > 0).then_some(event_at_ms);
        let write = UserPlaybackStateWrite {
            principal_id: request.principal_id,
            item_id: request.item_id,
            source_id: request.source_id,
            resume_position_ms,
            duration_ms: request.duration_ms,
            watched,
            watched_at_ms,
            last_played_at_ms,
            updated_at_ms: event_at_ms,
        };

        if let Some(existing) = existing {
            if state_matches_write(&existing, &write) {
                return Ok(existing);
            }
        }

        self.store.store_user_playback_state(write).await
    }

    pub(crate) async fn set_watched_state(
        &self,
        request: SetUserWatchedStateRequest,
    ) -> Result<UserPlaybackState> {
        self.ensure_item_exists(request.item_id).await?;
        if let Some(source_id) = request.source_id {
            self.ensure_source_belongs_to_item(source_id, request.item_id)
                .await?;
        }

        let event_at_ms = request.marked_at_ms.unwrap_or(current_time_ms()?);
        let resume_position_ms = match (request.watched, request.position_ms) {
            (true, _) | (false, None | Some(0)) => None,
            (false, Some(position_ms))
                if is_watched_by_policy(position_ms, request.duration_ms) =>
            {
                None
            }
            (false, Some(position_ms)) => Some(position_ms),
        };

        self.store
            .store_user_playback_state(UserPlaybackStateWrite {
                principal_id: request.principal_id,
                item_id: request.item_id,
                source_id: request.source_id,
                resume_position_ms,
                duration_ms: request.duration_ms,
                watched: request.watched,
                watched_at_ms: request.watched.then_some(event_at_ms),
                last_played_at_ms: request
                    .position_ms
                    .filter(|position| *position > 0)
                    .map(|_| event_at_ms),
                updated_at_ms: event_at_ms,
            })
            .await
    }

    pub(crate) async fn list_continue_watching(
        &self,
        principal_id: &UserPrincipalId,
        page: nako_core::PageRequest,
    ) -> Result<Vec<UserPlaybackState>> {
        self.store
            .list_continue_watching_user_playback_states(principal_id, page)
            .await
    }

    pub(crate) async fn list_continue_watching_entries(
        &self,
        principal: &AuthenticatedPrincipal,
        page: nako_core::PageRequest,
    ) -> Result<Vec<ContinueWatchingEntry>> {
        self.store
            .list_continue_watching_entries(principal, page)
            .await
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

    async fn ensure_source_belongs_to_item(
        &self,
        source_id: MediaSourceId,
        item_id: MediaItemId,
    ) -> Result<()> {
        let source = self
            .store
            .load_media_source(source_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_source",
                id: source_id.to_string(),
            })?;

        if source.item_id != item_id {
            return Err(NakoError::InvalidInput {
                message: format!("media source {source_id} does not belong to item {item_id}"),
            });
        }

        Ok(())
    }
}

fn state_matches_write(state: &UserPlaybackState, write: &UserPlaybackStateWrite) -> bool {
    state.principal_id == write.principal_id
        && state.item_id == write.item_id
        && state.source_id == write.source_id
        && state.resume_position_ms == write.resume_position_ms
        && state.duration_ms == write.duration_ms
        && state.watched == write.watched
        && state.watched_at_ms == write.watched_at_ms
        && state.last_played_at_ms == write.last_played_at_ms
        && state.updated_at_ms == write.updated_at_ms
}

fn default_user_playback_state(
    principal_id: UserPrincipalId,
    item_id: MediaItemId,
) -> UserPlaybackState {
    UserPlaybackState {
        principal_id,
        item_id,
        source_id: None,
        resume_position_ms: None,
        duration_ms: None,
        watched: false,
        watched_at_ms: None,
        last_played_at_ms: None,
        updated_at_ms: 0,
        version: 0,
    }
}

fn is_watched_by_policy(position_ms: u64, duration_ms: Option<u64>) -> bool {
    let Some(duration_ms) = duration_ms else {
        return false;
    };
    if duration_ms == 0 {
        return false;
    }

    (duration_ms >= 60_000 && position_ms.saturating_mul(100) >= duration_ms.saturating_mul(90))
        || (duration_ms >= 20 * 60_000 && duration_ms.saturating_sub(position_ms) <= 120_000)
}
