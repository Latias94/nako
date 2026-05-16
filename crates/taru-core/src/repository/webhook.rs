use async_trait::async_trait;

use crate::{
    EventId, NewWebhookDeliveryAttempt, NewWebhookEndpoint, Result, WebhookDeliveryAttemptId,
    WebhookDeliveryAttemptRecord, WebhookDeliveryStatus, WebhookEndpointId, WebhookEndpointRecord,
};

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn upsert_webhook_endpoint(
        &self,
        endpoint: NewWebhookEndpoint,
    ) -> Result<WebhookEndpointRecord>;

    async fn get_webhook_endpoint(
        &self,
        id: WebhookEndpointId,
    ) -> Result<Option<WebhookEndpointRecord>>;

    async fn list_enabled_webhook_endpoints(&self) -> Result<Vec<WebhookEndpointRecord>>;

    async fn create_webhook_delivery_attempt(
        &self,
        attempt: NewWebhookDeliveryAttempt,
    ) -> Result<WebhookDeliveryAttemptRecord>;

    async fn set_webhook_delivery_attempt_result(
        &self,
        id: WebhookDeliveryAttemptId,
        status: WebhookDeliveryStatus,
        http_status: Option<u16>,
        error: Option<String>,
        next_retry_at: Option<String>,
    ) -> Result<WebhookDeliveryAttemptRecord>;

    async fn list_webhook_delivery_attempts(
        &self,
        event_id: EventId,
    ) -> Result<Vec<WebhookDeliveryAttemptRecord>>;
}
