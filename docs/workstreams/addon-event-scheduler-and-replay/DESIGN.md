# Addon Event Scheduler And Replay

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

Addon Event Delivery now works when an operator manually asks Nako to deliver a
specific outbox event. That proves the protocol and persistence boundary, but
it is not yet an operational event system. Notification bridge, watch-state
sync, MCP automation, Arr-stack integration, and compatibility sidecars need
Nako to schedule Addon Event delivery automatically, retry safely, and expose a
separate forced replay story.

## Relevant Authority

- ADRs:
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/guides/ADDON_AUTHOR_GUIDE.md`
- Related workstreams:
  - `docs/workstreams/addon-ecosystem-foundation/`
  - `docs/workstreams/addon-task-runtime-contract/`
  - `docs/workstreams/addon-runtime-and-distribution/`

## Problem

Current Addon Event Delivery has three deliberate gaps:

- delivery is manually triggered through an admin endpoint;
- failed attempts record `next_retry_at`, but no scheduler consumes it;
- a repeated manual dispatch skips already succeeded subscriptions, but there
  is no explicit forced replay API with audit and operator intent.

If broad event-driven addons land before this is fixed, every addon feature
would need its own polling or retry story, and notification-like integrations
could duplicate events under retry pressure.

## Target State

When this workstream closes:

- Nako has an Addon Event scheduler loop that discovers due outbox events and
  deliverable Addon Event subscriptions.
- Scheduler execution is bounded by the existing resource budget and cannot
  concurrently deliver the same addon/event/subscription tuple twice.
- Retry uses durable `next_retry_at` and attempt limits.
- Already succeeded subscriptions are skipped by default.
- Forced replay is explicit, audited, and separate from normal scheduler
  delivery.
- Admin diagnostics can explain pending, skipped, succeeded, failed, due, and
  exhausted delivery state without exposing outbox payload values.

## In Scope

- Scheduler candidate selection and wake-up loop.
- Durable in-flight guard or lease semantics for addon/event/subscription
  delivery attempts.
- Retry/backoff consumption from `addon_event_delivery_attempts.next_retry_at`.
- Explicit forced replay API and audit-safe response shape.
- Event subscription filter execution for persisted routing plan filters.
- Focused SQLite/PostgreSQL contract parity for new persistence behavior.
- Admin/operator diagnostics and tests.

## Out Of Scope

- Notification provider matrix.
- Watch-state cloud sync.
- MCP media steward behavior.
- Arr-stack integration.
- DLNA/UPnP/WebDAV compatibility surfaces.
- Network tunnel provider behavior.
- Changing Addon manifest protocol version unless the scheduler requires a
  public wire contract change.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The durable outbox remains the source of event facts. | High | ADR 0014 and AEF-040. | Reopen event authority before scheduler work. |
| Normal delivery should be idempotent after success. | High | AEF-040 tests and handoff. | Forced replay must become default, increasing duplicate risk. |
| Forced replay needs separate operator intent. | High | Notification/watch-sync duplicate side effects are costly. | Admin deliver endpoints may need stronger warnings. |
| Existing semaphore resource budget is sufficient for first scheduler slice. | Medium | Webhook and Addon Event manual delivery share this pattern. | Add a dedicated scheduler budget before broad fan-out. |
| Event filters can start as deterministic local JSON predicates. | Medium | Routing plans persist filter metadata today. | Split a filter-language ADR before execution. |

## Architecture Direction

Keep Addon Event Delivery as a host-owned runtime adapter over the durable
event outbox. The scheduler should not become a second event bus or a feature
provider. It should select due work, acquire bounded execution authority, call
the existing delivery runtime, and record outcomes through the repository.

Normal scheduling and manual delivery should share the same safe default:
deliver only subscriptions without an existing succeeded attempt. Forced replay
should be a separate command with a reason, a new attempt row, and a response
that clearly identifies replayed subscriptions.

Event subscription filters should be evaluated before scheduling a sidecar
call. If the filter language expands beyond simple event facts such as
`library_id`, `source_id`, subject kind, or event kind, split an ADR instead of
embedding a bespoke query engine.

## Closeout Condition

This lane can close when:

- scheduler and retry behavior are implemented and tested;
- forced replay is explicit and tested;
- event filter execution is either implemented or split into a narrower lane;
- SQLite and PostgreSQL persistence contracts agree;
- admin diagnostics are redaction-safe;
- final evidence is recorded and follow-ons are named.
