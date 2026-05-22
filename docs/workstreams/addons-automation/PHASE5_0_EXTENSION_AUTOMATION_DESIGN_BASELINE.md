# Phase 5.0: Extension and Automation Design Baseline

Status: completed.

## Goal

Define Nako's M5 extension and automation surface before implementing outbox,
webhook delivery, automation jobs, addon manifests, or reference addons.

## Completed Shape

- Created a dedicated `addons-automation` workstream.
- Added ADR 0014 for durable event outbox and webhook/automation trigger
  policy.
- Added ADR 0015 for capability-scoped HTTP addons and external automation
  providers.
- Split M5 into event outbox, webhook delivery, automation jobs, addon
  manifest/resource contract, and reference addon stabilization.
- Documented resource classes for `network.webhook`, `network.addon`, and
  `automation.external_api`.
- Documented security boundaries for secrets, local paths, scopes, generated
  results, and inline HTTP calls.

## Design Commitments

- M5.1 implements event persistence first and does not deliver webhooks.
- Webhook delivery is a consumer of the event outbox, not inline domain logic.
- Automation jobs call external providers through bounded workers and store
  generated results as suggestions or artifacts by default.
- Addons are HTTP sidecars described by manifests, disabled by default, and
  scoped explicitly before use.
- Provider secrets use references and are resolved at runtime.
- Addon/provider payloads should use Nako IDs, public API URLs, and small
  snapshots rather than local filesystem paths.

## Non-Goals

- No runtime code changes in this phase.
- No webhook worker implementation.
- No automation provider implementation.
- No addon registration route or manifest validator implementation.
- No JavaScript SDK or embedded JavaScript runtime.
- No local model runtime or vector database.

## Validation

Expected coverage for this docs-only phase:

- ADR index links the new decisions.
- Workstream index marks `addons-automation` as active.
- Roadmap and goal map name M5 as the active goal and M5.1 as the next
  implementation goal.
- `git diff --check` passes.
