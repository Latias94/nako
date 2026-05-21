use taru_api::extension::{AddonSideEffectResponse, SubmitAddonSideEffectRequest};
use taru_core::Result;

use super::{AddonAppService, runtime::AddonSideEffectRuntime};

impl AddonAppService {
    pub async fn submit_addon_side_effect(
        &self,
        raw_token: &str,
        request: SubmitAddonSideEffectRequest,
    ) -> Result<AddonSideEffectResponse> {
        AddonSideEffectRuntime::new(
            self.store.clone(),
            self.permits.clone(),
            self.storage_backends.clone(),
        )
        .submit(raw_token, request)
        .await
    }
}
