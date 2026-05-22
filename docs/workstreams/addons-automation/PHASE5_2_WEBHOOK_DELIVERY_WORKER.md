# Phase 5.2: Webhook Delivery Worker

Status: completed.

## Goal

Configure webhook endpoints and deliver selected durable outbox events through
a bounded, retryable HTTP delivery boundary without making domain workflows
perform external HTTP calls inline.

## Completed Shape

- Added webhook endpoint and delivery-attempt domain models in `nako-core`.
- Added SQLite migration `0010_webhooks.sql` for endpoint configuration and
  delivery attempt records.
- Added `WebhookRepository` and SQLite persistence for endpoint upsert, enabled
  endpoint listing, attempt creation, attempt result updates, and per-event
  attempt inspection.
- Added `nako-events` webhook delivery service with event envelopes, HMAC-SHA256
  signatures, request timeouts, retry backoff timestamps, safe error mapping,
  and a `reqwest` transport.
- Added server configuration `webhook_concurrency` and app-level semaphore
  budget for webhook dispatch.
- Added HTTP routes for endpoint configuration, endpoint inspection, per-event
  delivery attempt inspection, and explicit event dispatch.

## HTTP Surface

Initial routes:

- `POST /webhooks/endpoints`
- `GET /webhooks/endpoints`
- `GET /webhooks/endpoints/{endpoint_id}`
- `GET /events/{event_id}/webhook-attempts`
- `POST /events/{event_id}/webhooks/deliver`

Endpoint secrets are configured as environment variable references only. The
resolved secret is used to sign the webhook body and is never returned in API
responses or persisted in delivery records.

## Delivery Semantics

- Domain workflows only enqueue outbox events.
- Explicit dispatch reads an outbox event, filters enabled endpoints by event
  subscription, and attempts delivery under the webhook concurrency budget.
- Webhook bodies use a versioned envelope containing Nako IDs, event kind,
  subject, occurrence time, and the event payload JSON.
- Receivers can verify `x-nako-signature: sha256=<hex hmac>` when a secret
  reference is configured.
- Non-2xx responses and transport errors produce failed attempts with safe
  errors and `next_retry_at` when more attempts remain.
- Re-running dispatch for the same event creates the next attempt number per
  endpoint until `max_attempts` is exhausted.

## Non-Goals

- No automatic background scheduler loop yet; M5.2 exposes the durable worker
  service and explicit dispatch boundary.
- No webhook management UI.
- No dead-letter queue table beyond terminal failed attempt records.
- No addon or automation provider execution.

## Validation

Coverage:

- `nako-db` tests verify endpoint and delivery-attempt persistence.
- `nako-events` tests verify signing, success persistence, failed delivery
  retry timestamps, and real `reqwest` delivery to a mocked local webhook
  server.
- `nako-server` tests verify HTTP endpoint configuration/listing and per-event
  delivery-attempt inspection.
- Workspace gates pass: `cargo fmt --all -- --check`, `cargo check
  --workspace`, `cargo nextest run --workspace`, and `git diff --check`.
