# Admin Media Management Context Links - TODO

Status: Closed
Last updated: 2026-05-30

## Task Ledger

### M0 - Lane Open

- [x] AMCL-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-media-management-context-links,docs/workstreams/client-surface-and-access-product-architecture]
  Goal: Split CSAPA-040 into a focused frontend execution lane for Management
  Context Links.
  Validation: Workstream docs exist, agree on target frontend, and record the
  backend contract as already implemented.
  Review: Confirm this lane targets `web/`, not the historical
  `apps/admin-web` validation console.
  Evidence: `DESIGN.md`, `ROUTE_MATRIX.md`, `WORKSTREAM.json`.
  Handoff: DONE. Start with AMCL-020.

### M1 - Route Resolver And Data Source

- [x] AMCL-020 [owner=codex] [deps=AMCL-010] [scope=web/src/api/public,web/src/shell,web/src/test]
  Goal: Add the frontend Management Context Link read boundary and one route
  resolver for backend `route_name` values.
  Validation: data-source contract tests cover live SDK calls, fixture
  fallback, known route mappings, unknown route handling, and safe query
  params.
  Review: No Media Web import of Admin API DTOs or mutation clients.
  Evidence: `web/src/api/public`, `web/src/test/data-source-contracts.test.ts`.
  Handoff: DONE. `createPublicManagementContextDataSource` wraps live and
  fixture reads; `resolveManagementContextLink(s)` maps known route names and
  rejects unknown, disabled, or unsafe targets. AMCL-030 and AMCL-040 can
  proceed from the accepted resolver contract.

### M2 - Media-to-Admin Link Rendering

- [x] AMCL-030 [owner=codex] [deps=AMCL-020] [scope=web/src/features/media,web/src/test]
  Goal: Render backend-computed management links in Media library, detail,
  source/version, watch, and playback-error contexts.
  Validation: media route tests cover enabled links, disabled reasons,
  ordinary viewer hidden/disabled states, and no unsafe text rendering.
  Review: Media UI must not call Admin mutations directly.
  Evidence: Media route/component tests and browser smoke screenshots.
  Handoff: DONE. Media detail and library routes render backend-computed
  Management Context Links through the Public Client boundary and shared
  resolver. Playback no-source diagnostics can receive the same link surface.
  Admin-owned command targets remain in AMCL-040.

### M3 - Admin Command And Return Links

- [x] AMCL-040 [owner=codex] [deps=AMCL-020] [scope=web/src/api/admin,web/src/features/admin,web/src/test]
  Goal: Map accepted management links into Admin routes or confirmation flows
  and add safe Admin-to-Media return links where Library Access allows it.
  Validation: Admin tests cover library scan, item metadata refresh handoff,
  jobs/runtime/support/access route targets, and safe Media return links.
  Review: Broad or mutating actions remain Admin-owned and explicit.
  Evidence: Admin route tests and mutation data-source tests.
  Handoff: DONE. Admin route search state now accepts sanitized Management
  Context Link params, renders Admin-owned context notices, keeps library scan
  behind Admin confirmation/mutation, hands item metadata refresh to Admin task
  context, and emits safe Media return links from stable IDs only. AMCL-050
  verified cross-surface roles and redaction behavior.

### M4 - Cross-Surface Verification

- [x] AMCL-050 [owner=codex] [deps=AMCL-030,AMCL-040] [scope=web,docs/workstreams/admin-media-management-context-links]
  Goal: Verify the full matrix with administrator, library manager, and viewer
  states plus browser smoke for representative transitions.
  Validation: `npm --prefix web run test`, `npm --prefix web run check`,
  `npm --prefix web run build:budget`, and browser smoke.
  Review: Redaction and permission behavior must match backend link state.
  Evidence: `EVIDENCE_AND_GATES.md`, screenshots or smoke notes.
  Result: DONE 2026-05-30. Web test/check/build:budget gates passed; browser
  smoke covered Media detail, Media library, Media-to-Admin, Admin-to-Media
  return links, disabled link states, and unsafe `source_id` redaction.
  Handoff: DONE. AMCL-090 can close the lane with this evidence.

### M5 - Closeout

- [x] AMCL-090 [owner=planner] [deps=AMCL-050] [scope=docs/workstreams/admin-media-management-context-links]
  Goal: Close the lane with final evidence, residual risks, and follow-ons.
  Validation: JSON validation, diff hygiene, web gates, and smoke evidence.
  Review: Confirm no broad Admin Web or Media Web product expansion was hidden
  inside this lane.
  Evidence: `CLOSEOUT.md`, `EVIDENCE_AND_GATES.md`, `HANDOFF.md`.
  Result: DONE 2026-05-30. Closeout accepted AMCL-050 web gate and browser
  smoke evidence, recorded residual follow-ons, and moved the lane to closed.
  Handoff: Split desktop/native playback, scoped manager job views, or
  role-specific UX polish separately.
