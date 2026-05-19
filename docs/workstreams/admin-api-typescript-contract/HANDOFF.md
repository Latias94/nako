# Admin API TypeScript Contract Handoff

Status: Active
Last updated: 2026-05-19

## Current State

AATC-010 is complete as a planning/opening slice. The previous
`admin-web-console` lane is closed as the baseline that produced
`apps/admin-web`, the first live/mock Admin API data-source boundary, and the
AWC-070 read-model wiring.

This lane now owns the contract gap: admin-web has hand-written wire DTOs for
existing `/admin/v1/*` read models, and those DTOs need a generated or
mechanically synchronized source before deeper UI workflows are added.

## Active Task

- Task ID: AATC-020
- Owner: codex
- Files: `crates/taru-api`, `apps/admin-web/src/adminApi`,
  `docs/workstreams/admin-api-typescript-contract`
- Validation: documented route/DTO inventory and artifact-shape decision
- Status: NEEDS_CONTEXT
- Review: run `review-workstream` before implementation tasks
- Evidence: updated `DESIGN.md` and `HANDOFF.md`

## Decisions Since Last Update

- Keep Admin API TypeScript contract separate from the Public Client SDK.
- Keep source ownership in `taru-api`.
- Default artifact location is app-local under `apps/admin-web` until a real
  second admin client creates package pressure.
- First route coverage should match AWC-070 read models.

## Blockers

- None for AATC-020.

## Next Recommended Action

Run AATC-020: inventory the current hand-written admin-web wire DTOs and choose
whether AATC-030 should generate interfaces only, route constants plus
interfaces, or a tiny generated admin client runtime.
