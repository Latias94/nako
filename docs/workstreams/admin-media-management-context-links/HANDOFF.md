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

- Task ID: AMCL-040
- Owner: codex
- Scope: `web/src/api/admin`, `web/src/features/admin`, Admin route state, and
  focused Admin tests.
- Status: READY

## Completed

- AMCL-020 added `createPublicManagementContextDataSource`, a live/fixture
  Public Client read boundary for `GET /management/context-links`.
- AMCL-020 added `resolveManagementContextLink(s)` in `web/src/shell`.
- Contract tests cover live SDK query params, fixture fallback, unsafe ID
  omission, known route mappings, disabled links, and unknown route names.
- Focused validation passed: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`
  and `npm --prefix web run check`.
- AMCL-030 added `ManagementContextLinks` in Media Web and renders backend
  links on media detail, library, selected source, and playback diagnostic
  contexts.
- AMCL-030 route/component tests cover enabled links, backend disabled reasons,
  unsafe target omission, and playback diagnostic actions.

## Next Recommended Action

Implement AMCL-040:

1. Teach Admin routes to accept safe Management Context Link search params
   emitted by `resolveManagementContextLink`.
2. Keep mutating targets as Admin-owned confirmation or mutation surfaces.
3. Add return links from Admin surfaces back to Media only when Public Client
   context is available.
4. Cover library scan, item metadata refresh handoff, jobs/runtime/support,
   access policy targets, and safe Media return links in tests.

## Guardrails

- `web/` is the product frontend. Do not implement new product UI in
  `apps/admin-web`.
- Backend link state is authoritative. Frontend should not recompute roles or
  Library Access.
- Mutating actions must enter Admin-owned confirmation/mutation paths.
- Media Web must not render raw paths, Source Locators, provider payloads,
  FFmpeg details, storage handles, tokens, or secrets.

## Parallelism

AMCL-040 is now the primary remaining execution task. AMCL-050 should wait
until Admin target handling is accepted.
