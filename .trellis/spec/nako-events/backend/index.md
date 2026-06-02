# nako-events Backend Guidelines

`nako-events` owns event envelopes, webhook signing, delivery attempts, and
transport helpers. It should stay aligned with the control-plane direction for
durable background work.

## Current Evidence

- `crates/nako-events/src/lib.rs`
- `docs/architecture/REALTIME_SYNC.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/adr/0053-control-plane-runtime-baseline.md`

## Boundaries

- Build `WebhookEventEnvelope` values and delivery requests.
- Use repository traits for subscription and delivery-attempt persistence.
- Use `WebhookTransport` for transport abstraction.
- Use `ReqwestWebhookTransport` only as the HTTP adapter.
- Keep event producer domain logic in the producing crates.

## Required Patterns

- Use protocol version `2026-05-15` for current webhook envelopes.
- Sign webhook payloads with HMAC SHA256 and `x-nako-signature`.
- Persist delivery attempts before reporting success or failure.
- Respect subscription enabled state and max-attempt limits.
- Use exponential retry delay based on attempt number.

## Forbidden Patterns

- Do not deliver disabled subscriptions.
- Do not log webhook secrets or full signed payloads.
- Do not add raw `tokio::spawn` delivery loops outside the control-plane model.
- Do not hide HTTP transport failures as successful attempts.

## Validation

- Focused:
  `cargo nextest run -p nako-events --no-fail-fast`
- Cross-layer control-plane checks:
  `cargo check -p nako-events -p nako-core --tests`
