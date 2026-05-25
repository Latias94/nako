# Addon Ecosystem Foundation — Handoff

Status: Complete
Last updated: 2026-05-25

## Current State

This lane is complete. The accepted decision is that Nako keeps fine-grained
Addon manifests, permissions, tasks, events, and audit while allowing
coarse-grained Addon Package and Addon Suite deployment. Official addons should
not force operators to write one Docker Compose service per small capability.

AEF-010 is complete as the authority freeze. AEF-020 is complete: Addon Task
runs now persist deterministic request fingerprints and reject mismatched
idempotency-key reuse. AEF-030 is complete: Nako's built-in official source
catalog and the official metadata scraper runtime now share descriptor facts
through `nako-official-addon-catalog`. AEF-040 is complete: manifest-declared
Addon Event Subscriptions can be delivered from the durable outbox through a
host-owned runtime with durable attempts, retries, grant checks, protocol
envelopes, and redaction-safe admin responses. AEF-050 is complete: the
official metadata scraper sidecar declares and serves a minimal
`library.scanned` event proof path. AEF-060 is complete: the immediate follow-on
is split into `docs/workstreams/addon-event-scheduler-and-replay/`.

## Active Task

- Task ID: none
- Owner: planner
- Files:
  - `docs/workstreams/addon-event-scheduler-and-replay/`
- Validation:
  - see the follow-on workstream gate set
- Status: DONE
- Review: no blocking closeout finding recorded
- Evidence: AEF-020 through AEF-060 evidence is recorded in
  `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Addon is the authority and capability unit.
- Addon Package and Addon Suite are deployment/distribution units.
- Official addons should default to suite packaging when capabilities share
  trust, dependencies, and lifecycle.
- Network Tunnel Provider behavior stays outside core; core owns Remote Access
  Endpoint facts.
- MCP/AI outputs should enter through Generated Artifacts and Acceptance
  Workflow unless a specific Addon write grant applies.
- Addon Event dispatch admin responses must expose redacted event summaries,
  not raw outbox payloads.
- Addon Event manual dispatch is idempotent after success; forced replay should
  be a separate explicit API, not the default deliver path.
- Event subscription filters are metadata today; evaluate them in the runtime
  before broad provider fan-out.
- The first official event proof is intentionally an ACK path, not a
  notification bridge.

## Blockers

- None currently.

## Follow-Ons

- Immediate: `docs/workstreams/addon-event-scheduler-and-replay/`
- Future named lanes: notification bridge, watch-state sync, MCP media steward,
  Arr-stack integration, DLNA/UPnP/WebDAV compatibility, Network Tunnel
  Provider.
