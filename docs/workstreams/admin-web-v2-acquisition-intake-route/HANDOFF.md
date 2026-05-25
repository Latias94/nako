# Admin Web V2 Acquisition Intake Route Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

The lane is closed. `/acquisition/intake` is now a route-first, read-only V2
page backed by `AdminAcquisitionIntakeCandidatesQuery` and
`AdminAcquisitionIntakeCandidateListResponse`.

## Active Task

- Task ID: none
- Owner: codex
- Files:
  - `apps/admin-web/src/App.tsx`
  - `apps/admin-web/src/features/acquisition/AcquisitionIntakePage.tsx`
  - `apps/admin-web/src/adminApi/client.ts`
  - `apps/admin-web/src/adminApi/dataSource.ts`
  - `apps/admin-web/src/App.test.tsx`
  - `apps/admin-web/src/adminApi/dataSource.test.ts`
- Status: DONE
- Validation: complete; see `EVIDENCE_AND_GATES.md`

## Decisions

- Route path is `/acquisition/intake`.
- The route is read-only.
- Watch-folder discovery and promotion mutations are follow-ons.
- `/legacy` remains available during this lane.
- The route renders no raw source refs, source URIs, locators, paths, tokens,
  or credentials.

## Blockers

- None.

## Next Recommended Action

Continue Admin Web V2 migration with another legacy workflow or close the
current V2 batch after final repository review.
