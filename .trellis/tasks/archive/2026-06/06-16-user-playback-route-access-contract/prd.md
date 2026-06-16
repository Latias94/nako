# User Playback Route Access Contract

## Goal

Harden U5 access and playback policy enforcement by proving Public Client user
playback state routes deny no-access principals at the HTTP boundary, matching
the app-service access contract.

## Requirements

- Add a focused HTTP route test for
  `GET /users/me/playback-state/items/{item_id}` with a principal that has
  `LibraryAccessLevel::None`.
- Preserve the existing behavior that Browse-only principals may read state but
  cannot update progress or watched state.
- Preserve the existing app-service ownership of Browse/Play checks; do not add
  route-local access helpers.
- Do not change public DTOs, route paths, generated contracts, persistence, or
  playback runtime behavior.

## Acceptance Criteria

- No-access state read returns `403 forbidden` with the standard Library Access
  `browse` message.
- Browse-only read and write-denial tests still pass.
- Focused `nako-server user_playback` gate passes.
- Task context validates and the work lands as a conventional commit.

## Scope Boundaries

- No production code change is expected unless the test reveals a real contract
  violation.
- No changes to continue-watching projection, pagination, or backfill behavior.
- No changes to playback ticket, Direct/Remux/HLS, or renderer policy paths.

## Technical Notes

- Primary package: `nako-server`.
- Follow `.trellis/spec/nako-server/backend/http-api-patterns.md` User Playback
  State Access Boundary.
- Existing app-service test already proves no-access reads are forbidden; this
  slice adds the missing HTTP route evidence.
