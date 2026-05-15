# Webhook Receiver Guide

Taru webhooks deliver persisted outbox events to configured HTTP endpoints.

## Configure An Endpoint

Use `POST /webhooks/endpoints` with a secret reference:

```json
{
  "id": null,
  "name": "receiver",
  "url": "https://example.test/taru-webhook",
  "secret_env": "TARU_WEBHOOK_SECRET",
  "subscribed_event_kinds": ["library.scanned"],
  "timeout_ms": 5000,
  "max_attempts": 3,
  "status": "enabled"
}
```

`secret_env` is the environment variable name Taru resolves at delivery time.
The secret value is not stored in endpoint records, jobs, or event payloads.

## Receive Events

Webhook bodies use a versioned JSON envelope. Receivers should persist event
IDs and handle duplicate deliveries idempotently.

Relevant headers:

```text
content-type: application/json
x-taru-event-id: <event id>
x-taru-event-kind: <event kind>
x-taru-signature: sha256=<hmac hex>
```

`x-taru-signature` is present when `secret_env` is configured. Verify it with
HMAC-SHA256 over the request body.

## Retry Behavior

Webhook delivery is bounded by endpoint timeout and max attempts. Failures are
recorded as delivery attempts and can be inspected through:

```text
GET /events/{event_id}/webhook-attempts
```

Domain workflows do not fail when webhook receivers are unavailable.
