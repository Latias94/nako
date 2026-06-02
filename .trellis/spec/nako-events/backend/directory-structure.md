# Directory Structure

`nako-events` currently lives in `src/lib.rs`. Split only around real event
contracts, repository orchestration, and transport adapters.

## Current Layout

- Event envelope and payload structs.
- Webhook request signing helpers.
- `WebhookTransport` trait.
- `ReqwestWebhookTransport` HTTP adapter.
- `WebhookDeliveryService` delivery orchestration.
- Retry and attempt-state helpers.

## Module Split Rules

- Move envelope and signing helpers together if they grow.
- Move reqwest adapter into a transport module before adding another adapter.
- Keep durable attempt orchestration separate from transport execution.
- Keep event producers outside this crate.

## Naming Rules

- Use `Webhook*` for webhook delivery concepts.
- Use `DeliveryAttempt` terminology for persisted attempt state.
- Use `EventEnvelope` terminology for transport-neutral event payloads.

## Anti-Patterns

- Do not add per-domain event producer modules here.
- Do not add server route handlers here.
- Do not mix retry scheduling with HTTP client implementation.
