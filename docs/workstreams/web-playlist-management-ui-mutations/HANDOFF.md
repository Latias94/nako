# Web Playlist Management UI Mutations - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. The backend/Public Client User Playlist contract is already
closed in `docs/workstreams/user-playlists-contract-and-web-slice/`. The first
web slice can list playlists and items at `/media/my-list` through Public
Client live data with fixture fallback.

This lane starts the mutation UI work. It must keep playlist management on the
Public Client boundary and must not import Admin API code into media features.

## Active Task

- Task ID: WPMU-020
- Owner: Codex
- Files: `web/src/api/public/media-data-source.ts`, `web/lib/use-media.ts`,
  `web/src/test/data-source-contracts.test.ts`
- Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`;
  `npm --prefix web run check`
- Status: READY
- Review: no Admin API imports, no raw feature-level fetch calls, no fixture
  mutation success claims
- Evidence: data-source and mutation hook tests

## Decisions Since Last Update

- This lane does not redesign the Public Client route contract.
- Fixture mode may preview forms/states, but cannot claim persisted mutation
  success.
- Reorder starts with explicit accessible controls; drag-and-drop is optional
  and should be split if it expands cost.

## Blockers

- None known.

## Next Recommended Action

Start WPMU-020 with TDD: add failing data-source/hook tests for playlist
mutations, then implement Public Client mutation methods and TanStack Query
cache behavior.
