# Admin Media Management Context Links - Handoff

Status: Closed
Last updated: 2026-05-30

## Current State

This lane is closed for the current `web/` product frontend.

The backend contract is already complete:

- Public Client route: `GET /management/context-links`
- SDK method: `NakoClient.managementContextLinks(query)`
- Backend route names:
  `library.scan`, `library.metadata_profile`, `item.metadata_refresh`,
  `jobs.filtered`, `playback.support`, `playback.runtime`,
  `access.library_policies`

## Active Task

- Task ID: none
- Lane: `web-product`
- Owner: none
- Scope: none.
- Status: CLOSED

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
- AMCL-050 verified web gates and representative browser transitions:
  `npm --prefix web run test`, `npm --prefix web run check`,
  `npm --prefix web run build:budget`, Media/Public import guard, and
  Playwright CLI smoke all passed.
- AMCL-050 browser smoke covered Media detail links, Media library links,
  Media-to-Admin refresh handoff, Admin-to-Media return links, library scan
  Admin routing, disabled link states, and unsafe `source_id` redaction.

## Next Recommended Action

Do not reopen this lane for broad product expansion. Split follow-on work into
new lanes:

1. Desktop/native playback strategy remains owned by CSAPA-050 or a follow-on
   desktop playback spike.
2. Role-specific UX polish or scoped manager job views should get a bounded
   web-product workstream.
3. Generated Artifact Metadata Authority apply workflow remains separate under
   the GAMA lane.

## Guardrails

- `web/` is the product frontend. Do not implement new product UI in
  `apps/admin-web`.
- Backend link state is authoritative. Frontend should not recompute roles or
  Library Access.
- Mutating actions must enter Admin-owned confirmation/mutation paths.
- Media Web must not render raw paths, Source Locators, provider payloads,
  FFmpeg details, storage handles, tokens, or secrets.

## Parallelism

AMCL is closed. The `web-product` lane is free for the next planner-approved
workstream.
