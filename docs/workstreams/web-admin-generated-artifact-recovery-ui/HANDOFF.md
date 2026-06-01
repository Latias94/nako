# Web Admin Generated Artifact Recovery UI — Handoff

Status: Active
Last updated: 2026-06-02

## Current State

GAOR closed with a backend/Admin API/Web data-source recovery queue:

- outcome list/detail and recovery queue Admin routes exist;
- `web/src/api/admin/read-models-data-source.ts` can load and map recovery
  entries;
- no route-level Web Admin UI renders that recovery queue yet.

## Active Task

- Task ID: `WAGR-020`
- Owner: codex
- Files: `web/src/features/admin`, `web/src/shell`, `web/src/api/admin`, `web/src/test`
- Validation: Web data-source tests, route tests, TypeScript check, browser smoke
- Status: READY
- Review: not started
- Evidence: `docs/workstreams/web-admin-generated-artifact-recovery-ui/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Opened a new `web-product` follow-on instead of reopening GAOR.
- Kept repair mutation out of scope.
- Chose attention filter plus paginated table as the first route shape.

## Blockers

- None yet.

## Next Recommended Action

- Implement `WAGR-020` as a read-only route and update route/data-source tests.
