use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use hmac::{Hmac, Mac};
pub use nako_core::{
    DomainEventKind, DomainEventSubject, NewOutboxEvent as DomainEvent, OutboxEventRecord,
    OutboxEventStatus,
};
use nako_core::{
    EventOutboxRepository, NakoError, Result, WebhookDeliveryAttemptId,
    WebhookDeliveryAttemptRecord, WebhookDeliveryStatus, WebhookEndpointRecord, WebhookRepository,
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use time::{Duration as TimeDuration, OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WebhookEventEnvelope {
    pub protocol_version: String,
    pub event_id: String,
    pub event_kind: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub occurred_at: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebhookHttpRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebhookHttpResponse {
    pub status: u16,
}

#[async_trait]
pub trait WebhookTransport: Send + Sync {
    async fn post(&self, request: WebhookHttpRequest) -> Result<WebhookHttpResponse>;
}

#[derive(Clone, Debug)]
pub struct ReqwestWebhookTransport {
    client: reqwest::Client,
}

impl Default for ReqwestWebhookTransport {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl ReqwestWebhookTransport {
    #[must_use]
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl WebhookTransport for ReqwestWebhookTransport {
    async fn post(&self, request: WebhookHttpRequest) -> Result<WebhookHttpResponse> {
        let mut builder = self
            .client
            .post(&request.url)
            .timeout(Duration::from_millis(request.timeout_ms))
            .body(request.body);

        for (name, value) in request.headers {
            builder = builder.header(name, value);
        }

        let response = builder.send().await.map_err(webhook_error)?;
        Ok(WebhookHttpResponse {
            status: response.status().as_u16(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct WebhookDeliveryService<T> {
    transport: Arc<T>,
}

impl<T> WebhookDeliveryService<T>
where
    T: WebhookTransport,
{
    #[must_use]
    pub fn new(transport: T) -> Self {
        Self {
            transport: Arc::new(transport),
        }
    }

    pub async fn deliver_once<R>(
        &self,
        repository: &R,
        event: &OutboxEventRecord,
        endpoint: &WebhookEndpointRecord,
        secret: Option<&str>,
    ) -> Result<WebhookDeliveryAttemptRecord>
    where
        R: EventOutboxRepository + WebhookRepository,
    {
        if !endpoint_subscribes_to(endpoint, event.kind) {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "webhook endpoint {} does not subscribe to {}",
                    endpoint.id,
                    event.kind.as_str()
                ),
            });
        }

        let existing_attempts = repository.list_webhook_delivery_attempts(event.id).await?;
        let attempt_number = existing_attempts
            .iter()
            .filter(|attempt| attempt.endpoint_id == endpoint.id)
            .map(|attempt| attempt.attempt_number)
            .max()
            .unwrap_or(0)
            + 1;

        if attempt_number > endpoint.max_attempts {
            return Err(NakoError::Conflict {
                message: format!(
                    "webhook endpoint {} exhausted attempts for event {}",
                    endpoint.id, event.id
                ),
            });
        }

        let attempt = repository
            .create_webhook_delivery_attempt(nako_core::NewWebhookDeliveryAttempt {
                id: WebhookDeliveryAttemptId::new(),
                endpoint_id: endpoint.id,
                event_id: event.id,
                attempt_number,
            })
            .await?;
        let request = build_webhook_request(event, endpoint, secret)?;
        let result = self.transport.post(request).await;

        match result {
            Ok(response) if (200..300).contains(&response.status) => {
                repository
                    .set_webhook_delivery_attempt_result(
                        attempt.id,
                        WebhookDeliveryStatus::Succeeded,
                        Some(response.status),
                        None,
                        None,
                    )
                    .await
            }
            Ok(response) => {
                let next_retry_at = next_retry_at(attempt_number, endpoint.max_attempts)?;
                repository
                    .set_webhook_delivery_attempt_result(
                        attempt.id,
                        WebhookDeliveryStatus::Failed,
                        Some(response.status),
                        Some(format!(
                            "webhook receiver returned HTTP {}",
                            response.status
                        )),
                        next_retry_at,
                    )
                    .await
            }
            Err(err) => {
                let next_retry_at = next_retry_at(attempt_number, endpoint.max_attempts)?;
                repository
                    .set_webhook_delivery_attempt_result(
                        attempt.id,
                        WebhookDeliveryStatus::Failed,
                        None,
                        Some(err.to_string()),
                        next_retry_at,
                    )
                    .await
            }
        }
    }
}

pub fn build_webhook_request(
    event: &OutboxEventRecord,
    endpoint: &WebhookEndpointRecord,
    secret: Option<&str>,
) -> Result<WebhookHttpRequest> {
    let payload =
        serde_json::from_str(&event.payload_json).map_err(|err| NakoError::InvalidInput {
            message: format!("failed to parse outbox event payload JSON: {err}"),
        })?;
    let envelope = WebhookEventEnvelope {
        protocol_version: "2026-05-15".to_owned(),
        event_id: event.id.to_string(),
        event_kind: event.kind.as_str().to_owned(),
        subject_kind: event.subject.kind().to_owned(),
        subject_id: event.subject.id(),
        occurred_at: event.occurred_at.clone(),
        payload,
    };
    let body = serde_json::to_string(&envelope).map_err(|err| NakoError::InvalidInput {
        message: format!("failed to serialize webhook event envelope: {err}"),
    })?;
    let mut headers = vec![
        ("content-type".to_owned(), "application/json".to_owned()),
        ("x-nako-event-id".to_owned(), event.id.to_string()),
        (
            "x-nako-event-kind".to_owned(),
            event.kind.as_str().to_owned(),
        ),
    ];

    if let Some(secret) = secret {
        headers.push((
            "x-nako-signature".to_owned(),
            format!("sha256={}", sign_body(secret, &body)?),
        ));
    }

    Ok(WebhookHttpRequest {
        url: endpoint.url.clone(),
        headers,
        body,
        timeout_ms: endpoint.timeout_ms,
    })
}

pub fn endpoint_subscribes_to(endpoint: &WebhookEndpointRecord, kind: DomainEventKind) -> bool {
    endpoint
        .subscribed_event_kinds
        .iter()
        .any(|candidate| candidate == "*" || candidate == kind.as_str())
}

pub fn sign_body(secret: &str, body: &str) -> Result<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|err| {
        NakoError::InvalidInput {
            message: format!("invalid webhook signing secret: {err}"),
        }
    })?;
    mac.update(body.as_bytes());
    Ok(hex_encode(&mac.finalize().into_bytes()))
}

fn next_retry_at(attempt_number: u32, max_attempts: u32) -> Result<Option<String>> {
    if attempt_number >= max_attempts {
        return Ok(None);
    }

    let exponent = attempt_number.saturating_sub(1).min(6);
    let delay_seconds = 30_i64 * 2_i64.pow(exponent);
    let retry_at = OffsetDateTime::now_utc() + TimeDuration::seconds(delay_seconds);
    retry_at
        .format(&Rfc3339)
        .map(Some)
        .map_err(|err| NakoError::InvalidInput {
            message: format!("failed to format webhook retry timestamp: {err}"),
        })
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn webhook_error(error: reqwest::Error) -> NakoError {
    NakoError::Provider {
        provider: "webhook".to_owned(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::{
        Router,
        body::Bytes,
        extract::State,
        http::{HeaderMap, StatusCode},
        routing::post,
    };
    use nako_core::{
        DatabaseLifecycle, Library, LibraryId, LibraryOptions, LibraryPreset, LibraryRepository,
    };
    use nako_core::{
        DomainEventSubject, EventId, EventOutboxRepository, NewOutboxEvent, NewWebhookEndpoint,
        WebhookDeliveryStatus, WebhookEndpointId, WebhookEndpointStatus, WebhookRepository,
    };
    use nako_db::NakoDatabase;
    use tokio::net::TcpListener;

    use super::*;

    #[derive(Default)]
    struct FakeTransport {
        response_status: u16,
        requests: Mutex<Vec<WebhookHttpRequest>>,
    }

    #[async_trait]
    impl WebhookTransport for FakeTransport {
        async fn post(&self, request: WebhookHttpRequest) -> Result<WebhookHttpResponse> {
            self.requests.lock().unwrap().push(request);
            Ok(WebhookHttpResponse {
                status: self.response_status,
            })
        }
    }

    #[derive(Clone, Default)]
    struct MockWebhookState {
        requests: Arc<Mutex<Vec<ReceivedWebhookRequest>>>,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct ReceivedWebhookRequest {
        event_id: Option<String>,
        signature: Option<String>,
        body: String,
    }

    async fn capture_webhook(
        State(state): State<MockWebhookState>,
        headers: HeaderMap,
        body: Bytes,
    ) -> StatusCode {
        let event_id = header_value(&headers, "x-nako-event-id");
        let signature = header_value(&headers, "x-nako-signature");
        let body = String::from_utf8(body.to_vec()).unwrap();
        state.requests.lock().unwrap().push(ReceivedWebhookRequest {
            event_id,
            signature,
            body,
        });

        StatusCode::ACCEPTED
    }

    fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned)
    }

    #[tokio::test]
    async fn reqwest_transport_posts_to_mocked_webhook_server() {
        let state = MockWebhookState::default();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let router = Router::new()
            .route("/webhook", post(capture_webhook))
            .with_state(state.clone());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let transport = ReqwestWebhookTransport::default();

        let response = transport
            .post(WebhookHttpRequest {
                url: format!("http://{addr}/webhook"),
                headers: vec![
                    ("content-type".to_owned(), "application/json".to_owned()),
                    (
                        "x-nako-event-id".to_owned(),
                        "018f0000-0000-7000-8000-000000000001".to_owned(),
                    ),
                    ("x-nako-signature".to_owned(), "sha256=test".to_owned()),
                ],
                body: r#"{"ok":true}"#.to_owned(),
                timeout_ms: 5_000,
            })
            .await
            .unwrap();

        assert_eq!(response.status, StatusCode::ACCEPTED.as_u16());
        let requests = state.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].event_id,
            Some("018f0000-0000-7000-8000-000000000001".to_owned())
        );
        assert_eq!(requests[0].signature, Some("sha256=test".to_owned()));
        assert_eq!(requests[0].body, r#"{"ok":true}"#);
        server.abort();
    }

    #[tokio::test]
    async fn webhook_delivery_signs_and_persists_successful_attempt() {
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        store.upsert_library(&library).await.unwrap();
        let event = store
            .enqueue_outbox_event(NewOutboxEvent {
                id: EventId::new(),
                kind: DomainEventKind::LibraryScanned,
                subject: DomainEventSubject::Library(library.id),
                library_id: Some(library.id),
                source_id: None,
                idempotency_key: format!("library.scanned:{}", library.id),
                payload_json: format!(r#"{{"library_id":"{}"}}"#, library.id),
            })
            .await
            .unwrap();
        let endpoint = store
            .upsert_webhook_endpoint(NewWebhookEndpoint {
                id: WebhookEndpointId::new(),
                name: "receiver".to_owned(),
                url: "https://example.test/webhook".to_owned(),
                secret_env: Some("NAKO_WEBHOOK_SECRET".to_owned()),
                subscribed_event_kinds: vec![DomainEventKind::LibraryScanned.as_str().to_owned()],
                timeout_ms: 5_000,
                max_attempts: 3,
                status: WebhookEndpointStatus::Enabled,
            })
            .await
            .unwrap();
        let transport = FakeTransport {
            response_status: 204,
            requests: Mutex::new(Vec::new()),
        };
        let service = WebhookDeliveryService::new(transport);

        let attempt = service
            .deliver_once(&store, &event, &endpoint, Some("secret"))
            .await
            .unwrap();

        assert_eq!(attempt.status, WebhookDeliveryStatus::Succeeded);
        assert_eq!(attempt.http_status, Some(204));
        assert_eq!(
            store
                .list_webhook_delivery_attempts(event.id)
                .await
                .unwrap(),
            vec![attempt]
        );
        let request = build_webhook_request(&event, &endpoint, Some("secret")).unwrap();
        assert!(
            request
                .headers
                .iter()
                .any(|(name, value)| name == "x-nako-signature" && value.starts_with("sha256="))
        );
        assert!(!request.body.contains("secret"));
        assert!(!request.body.contains("NAKO_WEBHOOK_SECRET"));
    }

    #[tokio::test]
    async fn webhook_delivery_records_failed_attempt_with_retry_time() {
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        store.upsert_library(&library).await.unwrap();
        let event = store
            .enqueue_outbox_event(NewOutboxEvent {
                id: EventId::new(),
                kind: DomainEventKind::LibraryScanned,
                subject: DomainEventSubject::Library(library.id),
                library_id: Some(library.id),
                source_id: None,
                idempotency_key: format!("library.scanned:{}", library.id),
                payload_json: format!(r#"{{"library_id":"{}"}}"#, library.id),
            })
            .await
            .unwrap();
        let endpoint = store
            .upsert_webhook_endpoint(NewWebhookEndpoint {
                id: WebhookEndpointId::new(),
                name: "receiver".to_owned(),
                url: "https://example.test/webhook".to_owned(),
                secret_env: None,
                subscribed_event_kinds: vec![DomainEventKind::LibraryScanned.as_str().to_owned()],
                timeout_ms: 5_000,
                max_attempts: 3,
                status: WebhookEndpointStatus::Enabled,
            })
            .await
            .unwrap();
        let service = WebhookDeliveryService::new(FakeTransport {
            response_status: 503,
            requests: Mutex::new(Vec::new()),
        });

        let attempt = service
            .deliver_once(&store, &event, &endpoint, None)
            .await
            .unwrap();

        assert_eq!(attempt.status, WebhookDeliveryStatus::Failed);
        assert_eq!(attempt.http_status, Some(503));
        assert!(attempt.next_retry_at.is_some());
        assert_eq!(attempt.attempt_number, 1);
    }
}
