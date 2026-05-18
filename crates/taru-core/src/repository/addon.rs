use async_trait::async_trait;

use crate::{
    AddonGrantRecord, AddonId, AddonRegistrationRecord, AddonStatus, AddonTokenId,
    AddonTokenRecord, NewAddonGrant, NewAddonRegistration, NewAddonToken, Result,
};

#[async_trait]
pub trait AddonRepository: Send + Sync {
    async fn upsert_addon_registration(
        &self,
        addon: NewAddonRegistration,
    ) -> Result<AddonRegistrationRecord>;

    async fn get_addon_registration(&self, id: AddonId) -> Result<Option<AddonRegistrationRecord>>;

    async fn find_addon_registration_by_manifest_id(
        &self,
        manifest_id: &str,
    ) -> Result<Option<AddonRegistrationRecord>>;

    async fn list_addon_registrations(
        &self,
        status: Option<AddonStatus>,
    ) -> Result<Vec<AddonRegistrationRecord>>;

    async fn create_addon_token(&self, token: NewAddonToken) -> Result<AddonTokenRecord>;

    async fn get_addon_token(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>>;

    async fn find_addon_token_by_hash(&self, token_hash: &str) -> Result<Option<AddonTokenRecord>>;

    async fn list_addon_tokens(&self, addon_id: AddonId) -> Result<Vec<AddonTokenRecord>>;

    async fn mark_addon_token_used(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>>;

    async fn rotate_addon_token(
        &self,
        rotated_token_id: AddonTokenId,
        new_token: NewAddonToken,
    ) -> Result<(AddonTokenRecord, AddonTokenRecord)>;

    async fn revoke_addon_token(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>>;

    async fn replace_addon_grants(
        &self,
        addon_id: AddonId,
        grants: Vec<NewAddonGrant>,
    ) -> Result<Vec<AddonGrantRecord>>;

    async fn list_addon_grants(&self, addon_id: AddonId) -> Result<Vec<AddonGrantRecord>>;
}
