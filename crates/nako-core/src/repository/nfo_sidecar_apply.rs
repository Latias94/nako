use async_trait::async_trait;

use super::PageRequest;
use crate::{
    LibraryId, MediaItemId, NewNfoSidecarApply, NfoSidecarApplyId, NfoSidecarApplyRecord,
    NfoSidecarApplyState, Result,
};

#[async_trait]
pub trait NfoSidecarApplyRepository: Send + Sync {
    async fn upsert_nfo_sidecar_apply(
        &self,
        apply: NewNfoSidecarApply,
    ) -> Result<NfoSidecarApplyRecord>;

    async fn get_nfo_sidecar_apply(
        &self,
        id: NfoSidecarApplyId,
    ) -> Result<Option<NfoSidecarApplyRecord>>;

    async fn find_nfo_sidecar_apply_by_idempotency_key(
        &self,
        target_library_id: LibraryId,
        idempotency_key: &str,
    ) -> Result<Option<NfoSidecarApplyRecord>>;

    async fn list_nfo_sidecar_applies_for_item(
        &self,
        media_item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<NfoSidecarApplyRecord>>;

    async fn set_nfo_sidecar_apply_state(
        &self,
        id: NfoSidecarApplyId,
        state: NfoSidecarApplyState,
        updated_at_ms: i64,
        outcome_json: Option<String>,
        safe_error_code: Option<String>,
        safe_message: Option<String>,
    ) -> Result<Option<NfoSidecarApplyRecord>>;
}
