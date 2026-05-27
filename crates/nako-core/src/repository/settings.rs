use crate::{
    AdminMetadataRawCacheSettingsRecord, AdminSettingsDocumentKey, AdminSettingsDocumentRecord,
    Result,
};

#[async_trait::async_trait]
pub trait AdminSettingsRepository: Send + Sync {
    async fn upsert_admin_metadata_raw_cache_settings(
        &self,
        record: AdminMetadataRawCacheSettingsRecord,
    ) -> Result<AdminMetadataRawCacheSettingsRecord>;

    async fn get_admin_metadata_raw_cache_settings(
        &self,
    ) -> Result<Option<AdminMetadataRawCacheSettingsRecord>>;

    async fn upsert_admin_settings_document(
        &self,
        record: AdminSettingsDocumentRecord,
    ) -> Result<AdminSettingsDocumentRecord>;

    async fn get_admin_settings_document(
        &self,
        key: AdminSettingsDocumentKey,
    ) -> Result<Option<AdminSettingsDocumentRecord>>;
}
