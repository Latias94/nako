# Media Web Client Foundation - TODO

Status: Active
Last updated: 2026-05-26

Task IDs use the `MWF` prefix.

## M0 - Scope And Evidence Freeze

- [x] MWF-010 [owner=planner] [deps=none] [scope=docs/workstreams/media-web-client-foundation,docs/workstreams/client-surface-and-access-product-architecture,docs/workstreams/README.md]
  Goal: Split the Media Web foundation lane from the client-surface planning
  lane and freeze route, access, API, and non-goal boundaries.
  Validation: `python -m json.tool docs/workstreams/media-web-client-foundation/WORKSTREAM.json`; `git diff --check -- docs/workstreams/media-web-client-foundation docs/workstreams/client-surface-and-access-product-architecture docs/workstreams/README.md`
  Evidence: `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`
  Handoff: DONE. Start MWF-020 before scaffolding UI so the first app slice
  consumes Public Client API contracts intentionally.

## M1 - Public Client API And SDK Readiness

- [x] MWF-020 [owner=unassigned] [deps=MWF-010] [scope=crates/nako-api,sdk/typescript,docs/workstreams/media-web-client-foundation]
  Goal: Audit the generated Public Client SDK/OpenAPI routes needed by the
  first Media Web routes and list gaps before UI code depends on them.
  Validation: `cargo test -p nako-api public_openapi -- --nocapture`; `cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts`; `git diff --check`
  Review: Public Client SDK must not expose Admin API paths, policy internals,
  raw Source Locators, local paths, provider payloads, or privileged playback
  URLs.
  Evidence: `EVIDENCE_AND_GATES.md` plus any SDK/OpenAPI parity diff.
  Handoff: If a route is missing, split the smallest Public Client API gap
  before scaffolding Media Web around a fake contract.
  Result: DONE_WITH_CONCERNS 2026-05-26. Public OpenAPI tests pass and the
  TypeScript SDK regenerated without content changes. `ROUTE_API_READINESS.md`
  records ready coverage for libraries, items, search, images, playback
  decisions, playback sessions, and User Playback State, plus gaps for current
  principal/session summary, library-scoped item browse, Recently Added
  sort/feed, and browser playback auth transport.

## M2 - App Scaffold And Connect Shell

- [x] MWF-030 [owner=codex] [deps=MWF-020] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/components/layout/AdminShell.tsx,apps/admin-web/src/surfaces/media,apps/admin-web/src/styles.css]
  Goal: Add the first shared frontend app-shell boundary with explicit Admin
  and Media surfaces, a Media connect/login MVP, fixture/live Public Client
  data-source boundary, and no Admin API imports inside the Media surface.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx mediaSurface.test.tsx mediaDataSource.test.ts`; `rg -n "admin/v1|AdminApi|adminApi" apps/admin-web/src/surfaces/media` returns no runtime dependency.
  Review: The Media surface must be a viewer client that coexists with Admin
  Web through navigation and route namespaces, not a copied Admin console and
  not a separate package.
  Evidence: App tests, data-source tests, and browser smoke notes.
  Handoff: Account switching means replacing/clearing the active connection
  until real credential/session APIs exist.
  Result: DONE 2026-05-26. Admin and Media now coexist inside
  `apps/admin-web` with explicit route namespaces and symmetric surface
  switchers. Media has an in-memory connect shell, fixture/live Public Client
  data-source boundary, generated SDK dependency, focused tests, and browser
  smoke evidence.

## M3 - Browse, Search, And Detail

- [x] MWF-040 [owner=codex] [deps=MWF-030] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/surfaces/media,apps/admin-web/src/styles.css]
  Goal: Implement accessible Media Libraries, Media Library detail, search,
  and Media Item detail from Public Client API data.
  Validation: `cd apps/admin-web && npm run check && npm run test`; browser smoke for `/media/libraries`, `/media/libraries/:libraryId`, `/media/search`, and `/media/items/:itemId`.
  Review: UI must rely on server-filtered Library Access and must not expose
  admin diagnostics or unsafe source/provider/storage fields.
  Evidence: Route tests, redaction tests, and browser screenshots or notes.
  Handoff: Defer recommendations and storefront discovery.
  Result: DONE 2026-05-26. `/media/libraries`,
  `/media/libraries/:libraryId`, `/media/search`, and `/media/items/:itemId`
  now use Public Client SDK data-source methods with URL-owned pagination/search
  state. The library detail route shows a source evidence list, not a fake
  library-scoped item grid; MWF-GAP-002 remains open for a first-class public
  library item browse contract.

## M4 - Source Selection, Player, And User Playback State

- [x] MWF-050 [owner=codex] [deps=MWF-040] [scope=apps/admin-web/src/surfaces/media,crates/nako-api,crates/nako-server]
  Goal: Add Source/Version Picker, playback decision integration, browser
  player shell, playback error state, and User Playback State progress writes.
  Validation: Focused Media Web tests plus focused server/Public API tests for
  playback and `me/playback` routes touched by the slice.
  Review: Playback must use Public Client API auth and Library Access; it must
  not mint privileged URLs or bypass playback-session policy.
  Evidence: Tests and browser playback smoke against fixture or live server.
  Handoff: Split desktop/native player capability gaps instead of hiding them
  inside browser playback.
  Result: DONE_WITH_CONCERNS 2026-05-26. Media Item detail and
  `/media/watch/:itemId` now share a Source/Version Picker, URL-owned
  `source_id`, playback decision preview, safe player shell, and User Playback
  State watched/unwatched writes through the Public Client SDK. The browser
  shell intentionally does not render a real media element or stream URL until
  MWF-GAP-004 chooses a secure playback auth transport. Continuous time-based
  progress writes remain tied to that real player follow-on.

## M5 - Closeout And Follow-On Split

- [ ] MWF-060 [owner=planner] [deps=MWF-050] [scope=docs/workstreams/media-web-client-foundation]
  Goal: Verify final gates, close the lane, and split Management Context Links,
  credential/session UX, invitation onboarding, desktop Tauri/native playback,
  and recommendations as needed.
  Validation: Package-local Media Web check/test/build, relevant Rust gates,
  `git diff --check`, browser desktop/mobile smoke, and review-workstream.
  Review: close-workstream before completion claims.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`
  Handoff: Do not reopen this lane for broad product expansion after the first
  local-media browse/play path lands.
