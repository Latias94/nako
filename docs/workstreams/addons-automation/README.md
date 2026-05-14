# Addons and Automation Workstream

## Status

Active for M5.

This workstream owns Taru's external extension surface: domain events, webhook
delivery, external automation jobs, HTTP addon manifests, addon resource
contracts, and reference addon behavior.

Top-level tracking:

- [Goal map](../../GOALS.md)
- [Roadmap](../../ROADMAP.md)
- [ADR 0014: durable event outbox](../../adr/0014-durable-event-outbox-for-webhooks-and-automation.md)
- [ADR 0015: capability-scoped addons and providers](../../adr/0015-capability-scoped-http-addons-and-automation-providers.md)
- [Phase 5.0 design baseline](PHASE5_0_EXTENSION_AUTOMATION_DESIGN_BASELINE.md)

## Goals

- Let Taru emit durable domain events without coupling core workflows to
  external HTTP delivery.
- Deliver webhooks through bounded, retryable, inspectable jobs.
- Model automation as explicit jobs that call configured external providers.
- Define a Taru HTTP addon manifest and resource contract before SDKs.
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
- Payloads use Taru IDs and API URLs, not raw local filesystem paths.
- Secrets are resolved at runtime from references and must not appear in jobs,
  logs, event payloads, manifests, or summaries.

## Resource Classes

M5 introduces or reserves these resource classes:

- `network.webhook`: webhook dispatch attempts.
- `network.addon`: addon manifest and resource calls.
- `automation.external_api`: API-key backed automation provider calls.

These classes are separate from scan, metadata, probe, and transcode budgets.
