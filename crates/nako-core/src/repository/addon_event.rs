use async_trait::async_trait;

use crate::{
    AddonEventDeliveryAttemptId, AddonEventDeliveryAttemptRecord, AddonId, EventId,
    NewAddonEventDeliveryAttempt, Result,
};

#[async_trait]
pub trait AddonEventDeliveryRepository: Send + Sync {
    async fn create_addon_event_delivery_attempt(
        &self,
        attempt: NewAddonEventDeliveryAttempt,
    ) -> Result<AddonEventDeliveryAttemptRecord>;

    async fn set_addon_event_delivery_attempt_result(
        &self,
        id: AddonEventDeliveryAttemptId,
        status: crate::AddonEventDeliveryStatus,
        http_status: Option<u16>,
        error: Option<String>,
        next_retry_at: Option<String>,
    ) -> Result<AddonEventDeliveryAttemptRecord>;

    async fn list_addon_event_delivery_attempts(
        &self,
        event_id: EventId,
    ) -> Result<Vec<AddonEventDeliveryAttemptRecord>>;

    async fn list_addon_event_delivery_attempts_for_addon(
        &self,
        addon_id: AddonId,
        event_id: EventId,
        declaration_id: &str,
    ) -> Result<Vec<AddonEventDeliveryAttemptRecord>>;
}
