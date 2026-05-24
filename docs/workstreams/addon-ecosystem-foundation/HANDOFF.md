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
through `nako-official-addon-catalog`.

## Active Task

- Task ID: AEF-040
- Owner: codex
- Files:
  - `crates/nako-core`
  - `crates/nako-db`
  - `crates/nako-server`
  - `crates/nako-addon-client`
- Validation:
  - `cargo nextest run -p nako-server addon_event --no-fail-fast`
  - `cargo nextest run -p nako-db event --no-fail-fast`
- Status: READY
- Review: pending
- Evidence: AEF-020 and AEF-030 evidence is recorded in `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Addon is the authority and capability unit.
- Addon Package and Addon Suite are deployment/distribution units.
- Official addons should default to suite packaging when capabilities share
  trust, dependencies, and lifecycle.
- Network Tunnel Provider behavior stays outside core; core owns Remote Access
  Endpoint facts.
- MCP/AI outputs should enter through Generated Artifacts and Acceptance
  Workflow unless a specific Addon write grant applies.

## Blockers

- None currently.

## Next Recommended Action

Implement AEF-040 Addon Event Delivery before adding broad official addon
feature breadth.
