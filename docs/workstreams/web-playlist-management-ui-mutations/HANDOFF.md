# Web Playlist Management UI Mutations - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. The backend/Public Client User Playlist contract is already
closed in `docs/workstreams/user-playlists-contract-and-web-slice/`. The first
web slice can list playlists and items at `/media/my-list` through Public
Client live data with fixture fallback. WPMU-020 added the web mutation
boundary: Public Client-backed data-source methods and TanStack Query mutation
hooks now cover playlist create, rename, delete, add item, remove item, and
reorder.

The remaining lane work is UI integration. It must keep playlist management on
the Public Client boundary and must not import Admin API code into media
features.

## Active Task

- Task ID: WPMU-030
- Owner: Codex
- Files: `web/src/features/media/my-list-page.tsx`, `web/src/shell`,
  `web/src/test`
- Validation: `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`;
  `npm --prefix web run check`
- Status: READY
- Review: controls must be accessible, must not use modal-only dead ends, and
  must keep the active route valid after deletion.
- Evidence: route/state tests and screenshots

## Decisions Since Last Update

- This lane does not redesign the Public Client route contract.
- Fixture mode may preview forms/states, but cannot claim persisted mutation
  success.
- Fixture mutation payloads explicitly return `persisted: false`.
- Playlist mutation hooks invalidate the playlist list and affected item list;
  delete also removes the deleted playlist item query cache.
- Reorder starts with explicit accessible controls; drag-and-drop is optional
  and should be split if it expands cost.

## Blockers

- None known.

## Next Recommended Action

Start WPMU-030 with TDD: add create, rename, and delete controls on
`/media/my-list`, then wire them to the existing playlist mutation hooks with
loading, empty, error, and route-safe deletion states.
