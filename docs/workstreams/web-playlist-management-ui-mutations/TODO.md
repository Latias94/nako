# Web Playlist Management UI Mutations - TODO

Status: Closed
Last updated: 2026-05-29

## M0 - Scope And Evidence Freeze

- [x] WPMU-010 [owner=planner] [deps=none] [scope=docs/workstreams/web-playlist-management-ui-mutations]
  Goal: Open the playlist management UI mutation lane and freeze scope, non-goals, and gates.
  Validation: `python -m json.tool docs/workstreams/web-playlist-management-ui-mutations/WORKSTREAM.json`; `git diff --check -- docs/workstreams/web-playlist-management-ui-mutations`.
  Evidence: `DESIGN.md`, task ledger, gates, and handoff.
  Handoff: DONE. Next task is WPMU-020.

## M1 - Public Client Mutation Boundary

- [x] WPMU-020 [owner=Codex] [deps=WPMU-010] [scope=web/src/api/public,web/lib/use-media.ts,web/src/test]
  Goal: Add web data-source methods and TanStack Query mutation hooks for create, rename, delete, add item, remove item, and reorder using the existing Public Client SDK.
  Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/use-media-contracts.test.tsx`; `npm --prefix web run check`.
  Review: no Admin API imports, no raw fetch calls from feature components, and no fixture mutation success claims.
  Evidence: data-source mutation contract tests and TanStack Query mutation hook cache-policy tests.
  Handoff: DONE. Public Client mutation boundary is ready for WPMU-030 CRUD controls.

## M2 - Playlist CRUD Controls

- [x] WPMU-030 [owner=Codex] [deps=WPMU-020] [scope=web/src/features/media/my-list-page.tsx,web/src/shell,web/src/test]
  Goal: Add create, rename, and delete playlist controls on `/media/my-list` with route-safe state, loading states, empty states, and error/conflict feedback.
  Validation: `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`; `npm --prefix web run check`.
  Review: controls must be accessible, must not use modal-only dead ends, and must keep the active route valid after deletion.
  Evidence: route/state tests and desktop/mobile screenshots.
  Handoff: DONE. Playlist CRUD controls are wired to Public Client mutation hooks; next task is WPMU-040.

## M3 - Item Add And Remove Flows

- [x] WPMU-040 [owner=Codex] [deps=WPMU-020,WPMU-030] [scope=web/src/features/media,web/src/test]
  Goal: Add Public Client-backed add/remove item flows from playlist items and a narrow add-to-playlist entry point from media browse/detail.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`.
  Review: inaccessible item facts must not leak, fixture fallback must be truthful, and no media source/library-file writes may be introduced.
  Evidence: route/data-source tests and browser smoke.
  Handoff: DONE. Playlist rows/cards can remove items; browse/detail can add items through the Public Client mutation hook. Next task is WPMU-050.

## M4 - Reorder And Conflict Handling

- [x] WPMU-050 [owner=Codex] [deps=WPMU-020,WPMU-030] [scope=web/src/features/media/my-list-page.tsx,web/src/test]
  Goal: Add explicit reorder behavior with stale-version/conflict recovery and keyboard-accessible controls before considering richer drag-and-drop.
  Validation: `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`; `npm --prefix web run check`.
  Review: reorder must submit full ordered `item_ids`, preserve route state, and refetch on conflicts.
  Evidence: state tests and browser smoke.
  Handoff: DONE. Explicit up/down controls reorder playlist items through the Public Client mutation hook and recover stale-version conflicts by refetching the current item order.

## M5 - Verification And Closeout

- [x] WPMU-060 [owner=planner] [deps=WPMU-030,WPMU-040,WPMU-050] [scope=docs/workstreams/web-playlist-management-ui-mutations]
  Goal: Verify the mutation UI lane, record evidence, close or split follow-ons, and commit closeout.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke desktop/mobile; `git diff --check`.
  Review: review-workstream has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and closeout notes.
  Handoff: DONE. Return to WDRP or selected follow-on.
