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

- [ ] MWF-020 [owner=unassigned] [deps=MWF-010] [scope=crates/nako-api,sdk/typescript,docs/workstreams/media-web-client-foundation]
  Goal: Audit the generated Public Client SDK/OpenAPI routes needed by the
  first Media Web routes and list gaps before UI code depends on them.
  Validation: `cargo test -p nako-api public_openapi -- --nocapture`; `cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts`; `git diff --check`
  Review: Public Client SDK must not expose Admin API paths, policy internals,
  raw Source Locators, local paths, provider payloads, or privileged playback
  URLs.
  Evidence: `EVIDENCE_AND_GATES.md` plus any SDK/OpenAPI parity diff.
  Handoff: If a route is missing, split the smallest Public Client API gap
  before scaffolding Media Web around a fake contract.

## M2 - App Scaffold And Connect Shell

- [ ] MWF-030 [owner=unassigned] [deps=MWF-020] [scope=apps/media-web,package.json]
  Goal: Add `apps/media-web` with a route-owned app shell, connect/login MVP,
  fixture/live data-source boundary, and no Admin API imports.
  Validation: `cd apps/media-web && npm run check && npm run test`; `rg -n "admin/v1|AdminApi|adminApi" apps/media-web` returns no runtime dependency.
  Review: The shell must be a viewer client, not a copied Admin Web console.
  Evidence: App tests, data-source tests, and browser smoke notes.
  Handoff: Account switching means replacing/clearing the active connection
  until real credential/session APIs exist.

## M3 - Browse, Search, And Detail

- [ ] MWF-040 [owner=unassigned] [deps=MWF-030] [scope=apps/media-web,apps/media-web/src/features]
  Goal: Implement accessible Media Libraries, Media Library detail, search,
  and Media Item detail from Public Client API data.
  Validation: `cd apps/media-web && npm run check && npm run test`; browser smoke for `/libraries`, `/libraries/:libraryId`, `/search`, and `/items/:itemId`.
  Review: UI must rely on server-filtered Library Access and must not expose
  admin diagnostics or unsafe source/provider/storage fields.
  Evidence: Route tests, redaction tests, and browser screenshots or notes.
  Handoff: Defer recommendations and storefront discovery.

## M4 - Source Selection, Player, And User Playback State

- [ ] MWF-050 [owner=unassigned] [deps=MWF-040] [scope=apps/media-web,crates/nako-api,crates/nako-server]
  Goal: Add Source/Version Picker, playback decision integration, browser
  player shell, playback error state, and User Playback State progress writes.
  Validation: Focused Media Web tests plus focused server/Public API tests for
  playback and `me/playback` routes touched by the slice.
  Review: Playback must use Public Client API auth and Library Access; it must
  not mint privileged URLs or bypass playback-session policy.
  Evidence: Tests and browser playback smoke against fixture or live server.
  Handoff: Split desktop/native player capability gaps instead of hiding them
  inside browser playback.

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

