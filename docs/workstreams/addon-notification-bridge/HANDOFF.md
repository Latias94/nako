# Addon Notification Bridge — Handoff

Status: Complete
Last updated: 2026-05-25

## Current State

This lane is closed. Nako now has an ACK-only official notification bridge
sidecar, built-in official catalog facts for it, host registration and
scheduler delivery proof, and a named follow-on for real provider adapters.

## Active Task

- Task ID: none
- Owner: planner
- Files:
  - `docs/workstreams/addon-notification-bridge`
  - `docs/workstreams/addon-notification-provider-adapters`
- Validation:
  - see `EVIDENCE_AND_GATES.md`
- Status: DONE
- Review: ANB-050 closeout found no blocking workstream-compliance or
  code-quality findings.
- Evidence: ANB-010 through ANB-050 evidence is recorded in
  `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Notification bridge is a separate lane, not part of AESR.
- Nako core owns event facts, scheduler delivery, grants, attempts, replay, and
  redaction-safe diagnostics.
- Notification provider credentials, templates, provider API calls, and
  provider-specific fan-out belong in the Addon sidecar.
- The first proof should subscribe to `library.scanned` and return a safe ACK
  before provider breadth.
- `nako-notification-bridge` now exists in `nako-official-addons` with manifest,
  health, diagnostics, ACK route, smoke script, Dockerfile, and Compose example.
- Event-only manifests currently fail validation with `EmptyResources`, so the
  bridge declares a narrow `webhook` resource on the same ACK path.
- Nako now exposes `nako.official.notification-bridge` from the built-in
  official Addon catalog and has a host test proving registration,
  health-check, routing-plan sync, and scheduled `library.scanned` delivery to
  the ACK path.
- The official sidecar health response now reports the declared webhook
  `resource_count` so Nako health-check validation can pass.
- Provider breadth is split into
  `docs/workstreams/addon-notification-provider-adapters/`; no real provider is
  implemented in this lane.
- Official addon suite packaging must remain possible; avoid a design that
  forces one Compose service per tiny notification capability.

## Blockers

- None currently known for ANB-050 closeout.

## Next Recommended Action

Start `docs/workstreams/addon-notification-provider-adapters/` at ANP-010 when
ready to select the first real provider. Do not restart provider work inside
this closed ACK-only bridge lane.
