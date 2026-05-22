# Addons and Automation Workstream

## Status

Completed for M5.

This workstream owns Nako's external extension surface: domain events, webhook
delivery, external automation jobs, HTTP addon manifests, addon resource
contracts, and reference addon behavior.

Top-level tracking:

- [Goal map](../../GOALS.md)
- [Roadmap](../../ROADMAP.md)
- [ADR 0014: durable event outbox](../../adr/0014-durable-event-outbox-for-webhooks-and-automation.md)
- [ADR 0015: capability-scoped addons and providers](../../adr/0015-capability-scoped-http-addons-and-automation-providers.md)
- [Phase 5.0 design baseline](PHASE5_0_EXTENSION_AUTOMATION_DESIGN_BASELINE.md)
- [Phase 5.1 event outbox foundation](PHASE5_1_EVENT_OUTBOX_FOUNDATION.md)
- [Phase 5.2 webhook delivery worker](PHASE5_2_WEBHOOK_DELIVERY_WORKER.md)
- [Phase 5.3 automation job model](PHASE5_3_AUTOMATION_JOB_MODEL.md)
- [Phase 5.4 addon manifest and resource contract](PHASE5_4_ADDON_MANIFEST_RESOURCE_CONTRACT.md)
- [Phase 5.5 reference addon and stabilization](PHASE5_5_REFERENCE_ADDON_STABILIZATION.md)

## Goals

- Let Nako emit durable domain events without coupling core workflows to
  external HTTP delivery.
- Deliver webhooks through bounded, retryable, inspectable jobs.
- Model automation as explicit jobs that call configured external providers.
- Define a Nako HTTP addon manifest and resource contract before SDKs.
- Keep provider credentials as secret references, not persisted plaintext.
- Store generated or external results as suggestions/artifacts unless accepted
  by explicit user or policy action.

## Non-Goals

- No in-process native plugin ABI.
- No embedded JavaScript runtime in M5.
- No local model runtime, vector database, or GPU model scheduling.
- No Stremio protocol compatibility implementation in the first addon slice.
- No remote storage backend work; that belongs to M6.

## Boundary Rules

- Domain services write outbox events; delivery workers consume events.
- HTTP handlers enqueue or inspect jobs; they do not call addon/provider HTTP
  endpoints inline.
- Addon and automation calls are bounded by resource class and timeout.
- Payloads use Nako IDs and API URLs, not raw local filesystem paths.
- Secrets are resolved at runtime from references and must not appear in jobs,
  logs, event payloads, manifests, or summaries.

## Resource Classes

M5 introduces or reserves these resource classes:

- `network.webhook`: webhook dispatch attempts.
- `network.addon`: addon manifest and resource calls.
- `automation.external_api`: API-key backed automation provider calls.

These classes are separate from scan, metadata, probe, and transcode budgets.
