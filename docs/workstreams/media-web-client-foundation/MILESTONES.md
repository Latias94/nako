# Media Web Client Foundation - Milestones

Status: Active
Last updated: 2026-05-26

## M0 - Scope And Evidence Freeze

Exit criteria:

- Media Web boundary is split from Admin Web and the product architecture lane.
- Route map, auth assumptions, Public Client API dependency, and non-goals are
  explicit.
- First executable task is chosen.

Primary evidence:

- `docs/workstreams/media-web-client-foundation/DESIGN.md`
- `docs/workstreams/media-web-client-foundation/TODO.md`

## M1 - Public Client API And SDK Readiness

Exit criteria:

- The first Media Web route set is mapped to Public Client API/OpenAPI/SDK
  support.
- Gaps are listed before UI scaffolding depends on them.
- Admin API leakage checks remain clean.

Primary gates:

- `cargo test -p nako-api public_openapi -- --nocapture`
- `cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts`

## M2 - App Scaffold And Connect Shell

Exit criteria:

- The shared frontend has explicit Admin and Media surface boundaries.
- The Media surface has a route shell, connect/login MVP, data-source boundary,
  tests, and no Admin API dependency.
- The Media surface can run in fixture mode and has a path to live Public
  Client API mode.

Primary gates:

- `cd apps/admin-web && npm run check && npm run test -- App.test.tsx mediaSurface.test.tsx mediaDataSource.test.ts`
- `rg -n "admin/v1|AdminApi|adminApi" apps/admin-web/src/surfaces/media`

## M3 - Browse, Search, And Detail

Exit criteria:

- Libraries, Media Library detail, search, and Media Item detail are usable.
- Route state is URL-owned where useful.
- Library Access is treated as server authority.
- Unsafe internal fields are redacted from viewer UI.

Primary gates:

- `cd apps/admin-web && npm run check && npm run test`
- Browser smoke for the `/media/*` browse/detail routes.

## M4 - Source Selection, Player, And User Playback State

Exit criteria:

- Source/Version Picker and player shell use Public Client API playback
  decisions.
- User Playback State writes use `/users/me/playback-state` routes.
- Playback errors are client-safe.
- Browser stream auth and desktop/native playback limitations are recorded
  rather than hidden.
- A real browser media element is not required until a secure playback auth
  transport is accepted.

Primary gates:

- Focused Media Web source/playback shell tests.
- Focused Rust/Public API tests for any touched playback or `me/playback`
  behavior.
- Browser smoke for the fixture watch shell and User Playback State writes.

## M5 - Closeout And Follow-On Split

Exit criteria:

- Package-local Media Web gates pass.
- Relevant Rust gates pass or are explicitly scoped out.
- Browser desktop/mobile smoke is recorded.
- Management Context Links, credentials/invitations, desktop native playback,
  and richer recommendations are split or deferred.
- `WORKSTREAM.json` status is updated.
