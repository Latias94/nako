use async_trait::async_trait;

use super::PageRequest;
use crate::{
    AuthenticatedPrincipal, ManagedArtworkArtifactRecord, MediaItem, MediaItemId, Result,
    SelectedArtworkRecord, UserPlaybackProfilePreference, UserPlaybackProfilePreferenceWrite,
    UserPlaybackState, UserPlaybackStateWrite, UserPrincipalId,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ContinueWatchingEntry {
    pub state: UserPlaybackState,
    pub item: MediaItem,
    pub images: Vec<ContinueWatchingImageEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContinueWatchingImageEntry {
    pub selected: SelectedArtworkRecord,
    pub artifact: ManagedArtworkArtifactRecord,
}

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

    async fn list_continue_watching_entries(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<Vec<ContinueWatchingEntry>>;
}

#[async_trait]
pub trait UserPlaybackProfilePreferenceRepository: Send + Sync {
    async fn upsert_user_playback_profile_preference(
        &self,
        preference: UserPlaybackProfilePreferenceWrite,
    ) -> Result<UserPlaybackProfilePreference>;

    async fn get_user_playback_profile_preference(
        &self,
        principal_id: &UserPrincipalId,
    ) -> Result<Option<UserPlaybackProfilePreference>>;

    async fn delete_user_playback_profile_preference(
        &self,
        principal_id: &UserPrincipalId,
    ) -> Result<bool>;
}
