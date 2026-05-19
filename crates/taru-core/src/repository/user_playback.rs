use async_trait::async_trait;

use super::PageRequest;
use crate::{MediaItemId, Result, UserPlaybackState, UserPlaybackStateWrite, UserPrincipalId};

#[async_trait]
pub trait UserPlaybackStateRepository: Send + Sync {
    async fn upsert_user_playback_state(
        &self,
        state: UserPlaybackStateWrite,
    ) -> Result<UserPlaybackState>;

    async fn get_user_playback_state(
        &self,
        principal_id: &UserPrincipalId,
        item_id: MediaItemId,
    ) -> Result<Option<UserPlaybackState>>;

    async fn list_continue_watching_states(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaybackState>>;
}
