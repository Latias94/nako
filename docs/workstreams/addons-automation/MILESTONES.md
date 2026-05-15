# Addons and Automation Milestones

## M5.0: Extension and Automation Design Baseline

Outcome: Taru has a documented execution plan for events, webhooks,
automation, addons, provider secrets, resource budgets, and trust boundaries.

Status: completed.

Deliverables:

- ADR for durable event outbox and webhook/automation trigger policy.
- ADR for capability-scoped HTTP addons and external automation providers.
- Workstream README, TODO, milestones, and phase note.
- Updated roadmap, goal map, and workstream index.

Exit criteria:

- Webhook, automation, addon, external API-key provider, resource-budget, and
  security boundaries are documented.
- The next implementation goal is M5.1 event outbox foundation.
- Docs-only validation passes.

## M5.1: Event Outbox Foundation

Outcome: Taru can persist domain events and query outbox state, but does not
deliver webhooks yet.

Status: completed.

Deliverables:

- Event domain model and event type registry in `taru-core`.
- SQLite event outbox migration in `taru-db`.
- Repository trait and SQLite implementation.
- Event write points for scan, metadata refresh, NFO import/export, and
  playback session completion.
- Tests for idempotency, payload safety, and persistence.

Exit criteria:

- Domain mutations can create durable events without external HTTP calls.
- Repeated/idempotent operations do not create duplicate logical events.
- Event payloads avoid secrets, raw local paths, and large binary fields.
- `cargo fmt`, `cargo check`, `cargo nextest run`, and `git diff --check`
  pass for the workspace.

## M5.2: Webhook Delivery Worker

Outcome: Taru can deliver selected outbox events to configured webhook
endpoints through bounded workers.

Status: completed.

Deliverables:

- Webhook endpoint configuration and secret reference model.
- Delivery attempt table with response status, safe error, next retry time, and
  terminal failure state.
- Worker that signs payloads, enforces timeout, retry, backoff, and resource
  budget.
- HTTP/API or CLI inspection path for webhook delivery state.
- Tests with mocked webhook servers.

Exit criteria:

- Delivery is retryable and restart-safe.
- Endpoint failures do not fail domain workflows.
- Payload signatures can be verified by receivers.
- Secrets do not appear in jobs, logs, or response envelopes.

## M5.3: Automation Job Model

Outcome: Taru can run explicit external automation jobs, such as
recommendation, metadata cleanup, summary generation, or title matching.

Deliverables:

- Automation provider configuration with secret references.
- Automation job kinds, inputs, summaries, and resource class.
- Bounded provider runner with timeout, retry, cancellation, and safe error
  mapping.
- Suggestion/artifact persistence model for generated outputs.
- Tests using mocked providers and no real network credentials.

Exit criteria:

- Automation jobs are auditable and retryable.
- Generated results do not mutate canonical metadata without explicit
  acceptance policy.
- Provider credentials are never persisted as plaintext.

## M5.4: Addon Manifest and Resource Contract

Outcome: Taru can register and validate an HTTP addon manifest and define the
first addon resource envelopes.

Deliverables:

- Addon manifest schema and versioning rules.
- Addon registration and validation model.
- Resource request/response envelopes for first resources.
- Timeout, retry, authentication, and scope policy.
- Tests for invalid manifests, scope denial, and response mapping.

Exit criteria:

- Addons are disabled by default and require explicit enablement.
- Taru can validate manifest resources and scopes before making calls.
- Handler code remains thin and does not embed addon HTTP behavior directly.

## M5.5: Reference Addon and Stabilization

Outcome: One minimal reference addon proves the protocol end to end.

Deliverables:

- Minimal reference addon implementation or fixture service.
- Server integration tests against the reference addon.
- Developer documentation for addon authors.
- API docs and limitations update.

Exit criteria:

- A local reference addon can be registered and queried.
- Protocol behavior is documented enough for another implementation.
- Validation gates pass for the workspace.
