use std::{collections::HashSet, env};

use nako_api::extension::{
    UpsertWebhookEndpointRequest, WebhookDeliveryAttemptsResponse, WebhookDispatchResponse,
    WebhookEndpointResponse, WebhookEndpointsResponse,
};
use nako_core::{
    DomainEventKind, EventId, EventOutboxRepository, NakoError, NewWebhookEndpoint,
    OutboxEventListFilter, OutboxEventRecord, PageRequest, Result, WebhookDeliveryStatus,
    WebhookEndpointId, WebhookEndpointRecord, WebhookRepository,
};
use nako_db::NakoDatabase;
use nako_events::{ReqwestWebhookTransport, WebhookDeliveryService, endpoint_subscribes_to};
use tokio::sync::Semaphore;
use tracing::warn;

fn resolve_webhook_secret(endpoint: &WebhookEndpointRecord) -> Result<Option<String>> {
    let Some(name) = endpoint.secret_env.as_deref() else {
        return Ok(None);
    };

    env::var(name).map(Some).map_err(|err| NakoError::InvalidInput {
        message: format!(
            "webhook endpoint {} references unavailable secret environment variable {name}: {err}",
            endpoint.id
        ),
    })
}

#[derive(Clone, Debug)]
pub(crate) struct WebhookAppService {
    store: NakoDatabase,
    permits: std::sync::Arc<Semaphore>,
}

impl WebhookAppService {
    pub(crate) fn new(store: NakoDatabase, permits: std::sync::Arc<Semaphore>) -> Self {
        Self { store, permits }
    }

    async fn get_outbox_event_or_not_found(&self, event_id: EventId) -> Result<OutboxEventRecord> {
        self.store
            .get_outbox_event(event_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "outbox_event",
                id: event_id.to_string(),
            })
    }

    fn normalize_webhook_endpoint(
        &self,
        request: UpsertWebhookEndpointRequest,
    ) -> Result<NewWebhookEndpoint> {
        let name = request.name.trim().to_owned();
        if name.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "webhook endpoint name cannot be empty".to_owned(),
            });
        }

        let url = request.url.trim().to_owned();
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            return Err(NakoError::InvalidInput {
                message: "webhook endpoint URL must use http or https".to_owned(),
            });
        }

        let mut seen = HashSet::new();
        let mut subscribed_event_kinds = Vec::new();
        for value in request.subscribed_event_kinds {
            let value = value.trim().to_owned();
            if value.is_empty() || !seen.insert(value.clone()) {
                continue;
            }
            if value != "*" && DomainEventKind::parse(&value).is_err() {
                return Err(NakoError::InvalidInput {
                    message: format!("unsupported webhook event kind: {value}"),
                });
            }
            subscribed_event_kinds.push(value);
        }
        if subscribed_event_kinds.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "webhook endpoint must subscribe to at least one event kind".to_owned(),
            });
        }

        let timeout_ms = request.timeout_ms.unwrap_or(10_000);
        if !(100..=60_000).contains(&timeout_ms) {
            return Err(NakoError::InvalidInput {
                message: "webhook timeout_ms must be between 100 and 60000".to_owned(),
            });
        }

        let max_attempts = request.max_attempts.unwrap_or(3);
        if !(1..=10).contains(&max_attempts) {
            return Err(NakoError::InvalidInput {
                message: "webhook max_attempts must be between 1 and 10".to_owned(),
            });
        }

        let secret_env = request.secret_env.and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        });

        Ok(NewWebhookEndpoint {
            id: request.id.unwrap_or_else(WebhookEndpointId::new),
            name,
            url,
            secret_env,
            subscribed_event_kinds,
            timeout_ms,
            max_attempts,
            status: request.status,
        })
    }

    pub async fn upsert_webhook_endpoint(
        &self,
        request: UpsertWebhookEndpointRequest,
    ) -> Result<WebhookEndpointResponse> {
        let endpoint = self.normalize_webhook_endpoint(request)?;
        let endpoint = self.store.upsert_webhook_endpoint(endpoint).await?;

        Ok(WebhookEndpointResponse { endpoint })
    }

    pub async fn get_webhook_endpoint(
        &self,
        endpoint_id: WebhookEndpointId,
    ) -> Result<WebhookEndpointResponse> {
        let endpoint = self
            .store
            .get_webhook_endpoint(endpoint_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "webhook_endpoint",
                id: endpoint_id.to_string(),
            })?;

        Ok(WebhookEndpointResponse { endpoint })
    }

    pub async fn list_enabled_webhook_endpoints(&self) -> Result<WebhookEndpointsResponse> {
        let endpoints = self.store.list_enabled_webhook_endpoints().await?;

        Ok(WebhookEndpointsResponse { endpoints })
    }

    pub async fn list_webhook_delivery_attempts(
        &self,
        event_id: EventId,
    ) -> Result<WebhookDeliveryAttemptsResponse> {
        self.get_outbox_event_or_not_found(event_id).await?;
        let attempts = self.store.list_webhook_delivery_attempts(event_id).await?;

        Ok(WebhookDeliveryAttemptsResponse { event_id, attempts })
    }

    pub async fn list_outbox_events(
        &self,
        filter: OutboxEventListFilter,
        page: PageRequest,
    ) -> Result<Vec<OutboxEventRecord>> {
        self.store.list_outbox_events(filter, page).await
    }

    pub async fn deliver_webhooks_for_event(
        &self,
        event_id: EventId,
    ) -> Result<WebhookDispatchResponse> {
        let event = self.get_outbox_event_or_not_found(event_id).await?;
        let endpoints = self.store.list_enabled_webhook_endpoints().await?;
        let service = WebhookDeliveryService::new(ReqwestWebhookTransport::default());
        let mut workers = tokio::task::JoinSet::new();
        let mut attempted_endpoints = 0_u32;
        let mut delivered = 0_u32;
        let mut failed = 0_u32;
        let mut skipped_endpoints = 0_u32;
        let mut attempts = Vec::new();
        let mut errors = Vec::new();

        for endpoint in endpoints {
            if !endpoint_subscribes_to(&endpoint, event.kind) {
                skipped_endpoints += 1;
                continue;
            }

            let permit =
                self.permits
                    .clone()
                    .acquire_owned()
                    .await
                    .map_err(|err| NakoError::Provider {
                        provider: "webhook".to_owned(),
                        message: format!("webhook resource budget was closed: {err}"),
                    })?;
            attempted_endpoints += 1;
            let endpoint_id = endpoint.id;
            let event = event.clone();
            let service = service.clone();
            let store = self.store.clone();

            workers.spawn(async move {
                let _permit = permit;
                let secret = resolve_webhook_secret(&endpoint).map_err(|err| (endpoint_id, err))?;
                service
                    .deliver_once(&store, &event, &endpoint, secret.as_deref())
                    .await
                    .map(|attempt| (endpoint_id, attempt))
                    .map_err(|err| (endpoint_id, err))
            });
        }

        while let Some(result) = workers.join_next().await {
            match result {
                Ok(Ok((_, attempt))) => {
                    match attempt.status {
                        WebhookDeliveryStatus::Succeeded => delivered += 1,
                        WebhookDeliveryStatus::Failed => failed += 1,
                        WebhookDeliveryStatus::Pending | WebhookDeliveryStatus::Running => {}
                    }
                    attempts.push(attempt);
                }
                Ok(Err((endpoint_id, err))) => {
                    failed += 1;
                    warn!(
                        endpoint_id = %endpoint_id,
                        event_id = %event.id,
                        error = %err,
                        "webhook delivery failed before attempt completion"
                    );
                    errors.push(format!("endpoint {endpoint_id}: {err}"));
                }
                Err(err) => {
                    failed += 1;
                    warn!(
                        event_id = %event.id,
                        error = %err,
                        "webhook delivery worker join failed"
                    );
                    errors.push(format!("webhook delivery worker join failed: {err}"));
                }
            }
        }
        attempts.sort_by_key(|attempt| (attempt.endpoint_id, attempt.attempt_number));

        Ok(WebhookDispatchResponse {
            event,
            attempted_endpoints,
            delivered,
            failed,
            skipped_endpoints,
            attempts,
            errors,
        })
    }
}
