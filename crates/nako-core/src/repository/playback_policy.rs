use async_trait::async_trait;

use crate::{
    EffectivePlaybackPolicy, LibraryId, PageRequest, PlaybackPolicy, PlaybackPolicyScope, Result,
    UserId, UserRole,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlaybackPolicyFilter {
    pub user_id: Option<UserId>,
    pub role: Option<UserRole>,
    pub library_id: Option<LibraryId>,
}

#[async_trait]
pub trait PlaybackPolicyRepository: Send + Sync {
    async fn upsert_playback_policy(&self, policy: &PlaybackPolicy) -> Result<()>;

    async fn delete_playback_policy(
        &self,
        scope: PlaybackPolicyScope,
        library_id: LibraryId,
    ) -> Result<()>;

    async fn list_playback_policies(
        &self,
        filter: PlaybackPolicyFilter,
        page: PageRequest,
    ) -> Result<Vec<PlaybackPolicy>>;

    async fn resolve_effective_playback_policy(
        &self,
        user_id: UserId,
        library_id: LibraryId,
    ) -> Result<EffectivePlaybackPolicy>;
}
