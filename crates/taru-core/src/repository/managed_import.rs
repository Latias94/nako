use async_trait::async_trait;

use super::PageRequest;
use crate::{
    LibraryId, ManagedImportArtifactId, ManagedImportArtifactListFilter,
    ManagedImportArtifactRecord, ManagedImportArtifactState, ManagedImportPromotionApplyId,
    ManagedImportPromotionApplyRecord, ManagedImportPromotionApplyState, ManagedImportSourceKind,
    NewManagedImportArtifact, NewManagedImportPromotionApply, Result,
};

#[async_trait]
pub trait ManagedImportRepository: Send + Sync {
    async fn upsert_managed_import_artifact(
        &self,
        artifact: NewManagedImportArtifact,
    ) -> Result<ManagedImportArtifactRecord>;

    async fn get_managed_import_artifact(
        &self,
        id: ManagedImportArtifactId,
    ) -> Result<Option<ManagedImportArtifactRecord>>;

    async fn find_managed_import_artifact_by_source(
        &self,
        target_library_id: LibraryId,
        source_kind: &ManagedImportSourceKind,
        source_uri: &str,
    ) -> Result<Option<ManagedImportArtifactRecord>>;

    async fn list_managed_import_artifacts(
        &self,
        filter: ManagedImportArtifactListFilter,
        page: PageRequest,
    ) -> Result<Vec<ManagedImportArtifactRecord>>;

    async fn set_managed_import_artifact_state(
        &self,
        id: ManagedImportArtifactId,
        state: ManagedImportArtifactState,
        updated_at_ms: i64,
        diagnostics_json: Option<String>,
    ) -> Result<Option<ManagedImportArtifactRecord>>;

    async fn upsert_managed_import_promotion_apply(
        &self,
        apply: NewManagedImportPromotionApply,
    ) -> Result<ManagedImportPromotionApplyRecord>;

    async fn get_managed_import_promotion_apply(
        &self,
        id: ManagedImportPromotionApplyId,
    ) -> Result<Option<ManagedImportPromotionApplyRecord>>;

    async fn find_managed_import_promotion_apply_by_idempotency_key(
        &self,
        target_library_id: LibraryId,
        idempotency_key: &str,
    ) -> Result<Option<ManagedImportPromotionApplyRecord>>;

    async fn list_managed_import_promotion_applies_for_artifact(
        &self,
        artifact_id: ManagedImportArtifactId,
        page: PageRequest,
    ) -> Result<Vec<ManagedImportPromotionApplyRecord>>;

    async fn set_managed_import_promotion_apply_state(
        &self,
        id: ManagedImportPromotionApplyId,
        state: ManagedImportPromotionApplyState,
        updated_at_ms: i64,
        outcome_json: Option<String>,
        safe_error_code: Option<String>,
        safe_message: Option<String>,
    ) -> Result<Option<ManagedImportPromotionApplyRecord>>;
}
