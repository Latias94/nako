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

- Task ID: AMCL-050
- Owner: codex
- Scope: `web/`, Admin/Media route transitions, browser smoke, and workstream
  evidence.
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
- AMCL-040 added sanitized Admin Management Context route state for libraries,
  tasks, transcoding, and users.
- AMCL-040 added Admin-owned scan confirmation, item metadata refresh task
  handoff, and safe Admin-to-Media return links from stable IDs.
- AMCL-040 split the pure Management Context model/normalizer away from the
  Public data source so Admin/Shell route parsing does not import Public client
  implementation.
- AMCL-040 adjusted the aggregate `total-js` gzip budget from 330 KiB to 335
  KiB after keeping route-level budgets unchanged and recording measured bundle
  output.

## Next Recommended Action

Implement AMCL-050:

1. Verify representative Media-to-Admin and Admin-to-Media transitions in the
   browser.
2. Confirm administrator/library-manager/viewer behavior follows backend link
   enabled/disabled state.
3. Re-run redaction/import guards for unsafe paths, tokens, Source Locators,
   provider payloads, storage handles, and Admin API imports from Media/Public.
4. Decide whether AMCL-090 can close the lane or whether role-specific UX
   follow-ons should be split.

## Guardrails

- `web/` is the product frontend. Do not implement new product UI in
  `apps/admin-web`.
- Backend link state is authoritative. Frontend should not recompute roles or
  Library Access.
- Mutating actions must enter Admin-owned confirmation/mutation paths.
- Media Web must not render raw paths, Source Locators, provider payloads,
  FFmpeg details, storage handles, tokens, or secrets.

## Parallelism

AMCL-050 is now the primary remaining execution task. AMCL-090 should wait
until cross-surface role/redaction verification is accepted.
