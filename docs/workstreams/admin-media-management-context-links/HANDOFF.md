# Admin Media Management Context Links - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

This lane is a frontend execution lane for the current `web/` product
frontend.

The backend contract is already complete:

- Public Client route: `GET /management/context-links`
- SDK method: `NakoClient.managementContextLinks(query)`
- Backend route names:
  `library.scan`, `library.metadata_profile`, `item.metadata_refresh`,
  `jobs.filtered`, `playback.support`, `playback.runtime`,
  `access.library_policies`

## Active Task

- Task ID: AMCL-030
- Owner: codex
- Scope: `web/src/features/media`, Management Context Link rendering, and
  focused media tests.
- Status: READY

## Completed

- AMCL-020 added `createPublicManagementContextDataSource`, a live/fixture
  Public Client read boundary for `GET /management/context-links`.
- AMCL-020 added `resolveManagementContextLink(s)` in `web/src/shell`.
- Contract tests cover live SDK query params, fixture fallback, unsafe ID
  omission, known route mappings, disabled links, and unknown route names.
- Focused validation passed: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`
  and `npm --prefix web run check`.

## Next Recommended Action

Implement AMCL-030:

1. Load Management Context Links from Media library/detail/source/watch
   contexts using `createPublicManagementContextDataSource`.
2. Render only backend-provided links and use `resolveManagementContextLink`
   for navigation targets.
3. Keep disabled states authoritative from `disabledReason`; do not recompute
   roles or Library Access in Media Web.
4. Add media route/component tests for enabled links, disabled reasons,
   ordinary viewer behavior, and unsafe text redaction.

## Guardrails

- `web/` is the product frontend. Do not implement new product UI in
  `apps/admin-web`.
- Backend link state is authoritative. Frontend should not recompute roles or
  Library Access.
- Mutating actions must enter Admin-owned confirmation/mutation paths.
- Media Web must not render raw paths, Source Locators, provider payloads,
  FFmpeg details, storage handles, tokens, or secrets.

## Parallelism

AMCL-030 Media rendering and AMCL-040 Admin handoffs can now be split between
workers because AMCL-020 stabilized the shared resolver contract.
