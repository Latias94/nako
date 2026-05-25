# Addon Notification Provider Attempt History

Status: Complete
Last updated: 2026-05-25

## Why This Lane Exists

When a provider send fails or succeeds, the current sidecar ACK response is the
only immediate observation surface. Operators need a small sidecar-owned history
to inspect recent provider outcomes without exposing provider URLs, secrets, or
raw payload values.

## Relevant Authority

- ADRs:
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- Related workstreams:
  - `docs/workstreams/addon-notification-provider-adapters/`

## Problem

Nako records Addon Event delivery attempts at the sidecar boundary, but it does
not know provider-specific send outcomes beyond safe sidecar HTTP status. The
sidecar has no recent-attempt view for provider-level troubleshooting.

## Target State

The lane closes when `nako-notification-bridge` records a bounded,
redaction-safe in-memory history of recent provider outcomes and exposes it
through diagnostics or a safe local endpoint.

## In Scope

- Bounded in-memory attempt ring buffer.
- Provider id, event id, event kind, subject facts, attempt number, status,
  retryable flag, provider HTTP status, and timestamp.
- Redaction-safe diagnostics/JSON output.
- Tests proving URLs, secrets, raw payload values, and message body are absent.

## Out Of Scope

- Persistent sidecar database.
- Provider retry queue.
- Nako core schema changes.
- Admin Web provider history UI.
- Full request/response body capture.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| A bounded in-memory history is enough for the first operator proof. | Medium | Sidecar currently has no storage layer. | Split persistent history into a later lane. |
| Nako core attempt records should remain unchanged. | High | Existing event scheduler owns host-side attempts. | Stop and write an ADR before host schema changes. |
| Safe provider facts can diagnose most fixture/live failures. | Medium | Current provider error body has safe status fields. | Add one safe field at a time with tests. |

## Architecture Direction

Keep provider attempt history inside `nako-notification-bridge` state. Record
only derived facts and safe statuses. Expose history through sidecar diagnostics
without changing the Addon Protocol or host attempt schema.

## Closeout Condition

This lane can close when recent attempt history is recorded, bounded, exposed
safely, tested, documented, and verified through package gates.

Closeout result: complete on 2026-05-25. Bounded in-memory provider attempt
history is implemented and exposed through redaction-safe diagnostics.
