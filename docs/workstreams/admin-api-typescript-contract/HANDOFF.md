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

## Active Task

- Task ID: AATC-030
- Owner: codex
- Files: `crates/taru-api/src`, `crates/taru-api/examples`,
  `apps/admin-web/src/adminApi`
- Validation: `cargo check -p taru-api --examples`, focused `taru-api`
  generator tests, generated artifact sync check, and admin-web type check
- Status: NEEDS_CONTEXT
- Review: run `review-workstream` before accepting implementation completion
- Evidence: generator source, example command, generated contract artifact,
  and Rust sync/leakage tests

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

## Blockers

- None for AATC-030.

## Next Recommended Action

Run AATC-030: add a focused `taru-api` Admin TypeScript contract generator,
emit `apps/admin-web/src/adminApi/generated/contract.ts`, and wire
`client.ts` to import generated route constants and response types.
