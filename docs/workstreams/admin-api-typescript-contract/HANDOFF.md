# Admin API TypeScript Contract Handoff

Status: Active
Last updated: 2026-05-19

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

## Active Task

- Task ID: AATC-040
- Owner: codex
- Files: `apps/admin-web/src/adminApi`, `apps/admin-web/src`
- Validation: `cd apps/admin-web && npm run check`, `npm run test`, and
  `npm run build`
- Status: READY
- Review: review-workstream for DTO duplication and redaction fixture safety
- Evidence: admin-web imports generated contract for covered wire DTOs; tests
  keep source fallback and redaction behavior

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

## Blockers

- None for AATC-030.
- None for AATC-040.

## Next Recommended Action

Run AATC-040: remove durable wire DTO duplication from
`apps/admin-web/src/adminApi/types.ts` by importing/re-exporting generated
contract DTOs, while keeping UI-only view models local to admin-web.
