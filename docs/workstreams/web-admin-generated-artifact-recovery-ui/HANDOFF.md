# Web Admin Generated Artifact Recovery UI — Handoff

Status: Closed
Last updated: 2026-06-02

## Current State

The Web Admin recovery queue route is shipped and verified:

- `/admin/automation/generated-artifacts/recovery` renders the GAOR recovery
  read model;
- attention, limit, and offset are route state;
- rows stay read-only and link to the existing Metadata Authority apply plan
  route instead of executing repair;
- fixture/live data-source and route tests cover the read path;
- desktop/mobile browser smoke passed without page-level overflow or sensitive
  raw data leakage.

## Active Task

- Task ID: `WAGR-020`
- Owner: codex
- Files: `web/src/features/admin`, `web/src/shell`, `web/src/api/admin`, `web/src/test`
- Validation: Web data-source tests, route tests, TypeScript check, browser smoke
- Status: DONE
- Review: no blocking findings during closeout
- Evidence: `docs/workstreams/web-admin-generated-artifact-recovery-ui/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Opened a new `web-product` follow-on instead of reopening GAOR.
- Kept repair mutation out of scope.
- Chose attention filter plus paginated table as the first route shape.
- Tightened mobile shell header sizing after browser smoke exposed top-bar
  overlap at 390px.
- Raised only the aggregate Web total-JS gzip budget from 340 KiB to 341 KiB
  after route slimming left the meaningful Admin surface at 340.53 KiB.

## Blockers

- None.

## Next Recommended Action

- Do not reopen this lane for mutation. Open
  `proposed:generated-artifact-apply-repair-actions` if operators need a
  guarded repair action over the same Metadata Authority semantics.
