# Admin API TypeScript Contract Handoff

Status: Active
Last updated: 2026-05-20

## Current State

AATC-010 is complete as a planning/opening slice. The previous
`admin-web-console` lane is closed as the baseline that produced
`apps/admin-web`, the first live/mock Admin API data-source boundary, and the
AWC-070 read-model wiring.

AATC-020 is complete. `ADMIN_CONTRACT_INVENTORY.md` records the hand-written
wire DTO audit, the route/query inventory, and the chosen first artifact
shape.

AATC-030 is complete. `taru-api` owns an Admin API TypeScript contract
generator and emit example. The committed app-local artifact exports the
AWC-070 route constants, query interfaces, and wire interfaces under
`apps/admin-web/src/adminApi/generated/contract.ts`. `client.ts` now imports
generated response types and route constants while keeping its hand-written
fetch/runtime boundary.

AATC-040 is complete. Admin-web no longer owns duplicate hand-written wire
DTO definitions for the covered AWC-070 routes. `types.ts` re-exports
generated wire DTOs and keeps UI-only view/data-source types local;
`dataSource.ts`, `mockData.ts`, and data-source tests consume generated
contract types/routes directly.

## Active Task

- Task ID: AATC-050
- Owner: codex
- Files: `crates/taru-api`, `sdk/typescript`, `docs/api`,
  `docs/workstreams/admin-api-typescript-contract`
- Validation: focused `taru-api` nextest tests, public TypeScript SDK sync
  test, admin-web gates, and `git diff --check`
- Status: READY
- Review: review-workstream and verify-rust-workstream before closeout
- Evidence: Public Client SDK separation tests and Admin API contract docs

## Decisions Since Last Update

- Keep Admin API TypeScript contract separate from the Public Client SDK.
- Keep source ownership in `taru-api`.
- Default artifact location is app-local under `apps/admin-web` until a real
  second admin client creates package pressure.
- First route coverage should match AWC-070 read models.
- AATC-020 chooses route constants plus wire/query interfaces, not interfaces
  only and not a generated fetch client.
- `client.ts` should remain hand-written for base URL normalization, bearer
  auth, request failure behavior, and future live/mock fallback policy.
- The AATC-030 generator intentionally emits a narrow explicit contract rather
  than a generated fetch client or a public/admin combined SDK.
- AATC-040 keeps compatibility re-exports from `types.ts` but moves source
  ownership of covered wire DTOs to `generated/contract`.

## Blockers

- None for AATC-030.
- None for AATC-040.
- None for AATC-050.

## Next Recommended Action

Run AATC-050: document generation and separation commands, refresh Public
Client SDK/Admin API separation evidence, and close or split the workstream
before starting Jobs/Catalog/Playback detail-page UI work.
