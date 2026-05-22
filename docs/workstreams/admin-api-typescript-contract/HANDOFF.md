# Admin API TypeScript Contract Handoff

Status: Completed
Last updated: 2026-05-20

## Current State

AATC-010 is complete as a planning/opening slice. The previous
`admin-web-console` lane is closed as the baseline that produced
`apps/admin-web`, the first live/mock Admin API data-source boundary, and the
AWC-070 read-model wiring.

AATC-020 is complete. `ADMIN_CONTRACT_INVENTORY.md` records the hand-written
wire DTO audit, the route/query inventory, and the chosen first artifact
shape.

AATC-030 is complete. `nako-api` owns an Admin API TypeScript contract
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

AATC-050 is complete. Public/Admin separation is documented and test-visible:
the Admin Contract test set proves generated Admin route constants stay out of
`nako-client-protocol` public route inventory, and the Public TypeScript SDK
guard still rejects Admin routes.

## Active Task

- None. The workstream is closed.

## Decisions Since Last Update

- Keep Admin API TypeScript contract separate from the Public Client SDK.
- Keep source ownership in `nako-api`.
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
- None.

## Next Recommended Action

Do not reopen this lane for product UI. Split follow-ons as separate
workstreams or issues:

- Admin npm package only if a second Admin API consumer appears.
- Jobs filters and job detail entry point.
- Catalog Governance filters and item review detail.
- Playback sessions filters and session detail.
- Settings diagnostics layout polish before editable settings.
