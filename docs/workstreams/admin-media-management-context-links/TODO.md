# Admin Media Management Context Links - TODO

Status: Active
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

- [ ] AMCL-030 [owner=codex] [deps=AMCL-020] [scope=web/src/features/media,web/src/test]
  Goal: Render backend-computed management links in Media library, detail,
  source/version, watch, and playback-error contexts.
  Validation: media route tests cover enabled links, disabled reasons,
  ordinary viewer hidden/disabled states, and no unsafe text rendering.
  Review: Media UI must not call Admin mutations directly.
  Evidence: Media route/component tests and browser smoke screenshots.
  Handoff: Admin-owned command targets remain in AMCL-040.

### M3 - Admin Command And Return Links

- [ ] AMCL-040 [owner=codex] [deps=AMCL-020] [scope=web/src/api/admin,web/src/features/admin,web/src/test]
  Goal: Map accepted management links into Admin routes or confirmation flows
  and add safe Admin-to-Media return links where Library Access allows it.
  Validation: Admin tests cover library scan, item metadata refresh handoff,
  jobs/runtime/support/access route targets, and safe Media return links.
  Review: Broad or mutating actions remain Admin-owned and explicit.
  Evidence: Admin route tests and mutation data-source tests.
  Handoff: AMCL-050 verifies cross-surface behavior.

### M4 - Cross-Surface Verification

- [ ] AMCL-050 [owner=codex] [deps=AMCL-030,AMCL-040] [scope=web,docs/workstreams/admin-media-management-context-links]
  Goal: Verify the full matrix with administrator, library manager, and viewer
  states plus browser smoke for representative transitions.
  Validation: `npm --prefix web run test`, `npm --prefix web run check`,
  `npm --prefix web run build:budget`, and browser smoke.
  Review: Redaction and permission behavior must match backend link state.
  Evidence: `EVIDENCE_AND_GATES.md`, screenshots or smoke notes.
  Handoff: Ready for AMCL-090.

### M5 - Closeout

- [ ] AMCL-090 [owner=planner] [deps=AMCL-050] [scope=docs/workstreams/admin-media-management-context-links]
  Goal: Close the lane with final evidence, residual risks, and follow-ons.
  Validation: JSON validation, diff hygiene, web gates, and smoke evidence.
  Review: Confirm no broad Admin Web or Media Web product expansion was hidden
  inside this lane.
  Evidence: `CLOSEOUT.md`, `EVIDENCE_AND_GATES.md`, `HANDOFF.md`.
  Handoff: Split desktop/native playback or scoped manager job views separately.
