# Addon Ecosystem Foundation — Handoff

Status: Active
Last updated: 2026-05-25

## Current State

The lane has been opened after the Addon ecosystem architecture review. The
accepted decision is that Nako should keep fine-grained Addon manifests,
permissions, tasks, events, and audit while allowing coarse-grained Addon
Package and Addon Suite deployment. Official addons should not force operators
to write one Docker Compose service per small capability.

AEF-010 is complete as the authority freeze. AEF-020 is complete: Addon Task
runs now persist deterministic request fingerprints and reject mismatched
idempotency-key reuse. AEF-030 is complete: Nako's built-in official source
catalog and the official metadata scraper runtime now share descriptor facts
through `nako-official-addon-catalog`. AEF-040 is complete: manifest-declared
Addon Event Subscriptions can be delivered from the durable outbox through a
host-owned runtime with durable attempts, retries, grant checks, protocol
envelopes, and redaction-safe admin responses.

## Active Task

- Task ID: AEF-050
- Owner: codex
- Files:
  - `F:\SourceCodes\Rust\nako-official-addons`
  - `scripts`
  - `docs`
- Validation:
  - focused official addon tests
  - a Nako-hosted smoke where feasible
- Status: READY
- Review: pending
- Evidence: AEF-020, AEF-030, and AEF-040 evidence is recorded in
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

## Blockers

- None currently.

## Next Recommended Action

Implement AEF-050 as a small official event-driven addon proof path before
adding broad notification, sync, MCP, Arr-stack, tunnel, or compatibility
provider breadth.
