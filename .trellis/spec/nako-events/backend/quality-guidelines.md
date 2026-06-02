# Quality Guidelines

Event changes must preserve signing correctness, durable delivery evidence, and
control-plane compatibility.

## Required Patterns

- Test HMAC SHA256 signatures against deterministic payloads.
- Test `x-nako-signature` header construction.
- Keep event envelope protocol version explicit.
- Keep delivery attempts auditable across success and failure.
- Keep retry math deterministic and bounded.

## Forbidden Patterns

- Do not use wall-clock sleeps in unit tests.
- Do not create fire-and-forget webhook tasks.
- Do not add event payload types that leak provider secrets.
- Do not make reqwest the only possible transport path.

## Tests Required

- Envelope serialization tests.
- Signature tests.
- Disabled subscription tests.
- Max-attempt/exhaustion tests.
- Transport success and failure tests with fake `WebhookTransport`.
- Retry delay tests.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-events --no-fail-fast`
- Control-plane compile:
  `cargo check -p nako-events -p nako-core --tests`
