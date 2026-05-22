# User Playback State Contract TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Contract And Principal Freeze

- [x] UPS-010 [owner=planner] [deps=none] [scope=docs/workstreams/user-playback-state-contract, docs/adr, CONTEXT.md]
  Goal: Freeze the first public **User Playback State** contract, including user principal strategy, DTOs, route inventory, progress semantics, watched threshold policy, and explicit non-goals.
  Validation: `git diff --check`
  Review: confirm the contract does not turn Single-Admin Mode into a permanent single-user domain model.
  Evidence: `CONTRACT.md`, `DESIGN.md`, `EVIDENCE_AND_GATES.md`, `docs/adr/0028-user-playback-state-principal-and-public-contract.md`.
  Handoff: Complete. UPS-020 can implement storage/service behavior against the frozen contract.

## M1 - Server Storage And App Service

- [x] UPS-020 [owner=codex] [deps=UPS-010] [scope=crates/nako-core, crates/nako-db, crates/nako-server]
  Goal: Implement user playback state repository traits, SQLite schema/migrations, principal resolution, and app-service behavior for lookup/report/mark-watched.
  Validation: `cargo nextest run -p nako-db -p nako-server user_playback --no-fail-fast`
  Review: verify idempotent progress writes, safe principal scoping, source/item validation, and watched threshold behavior.
  Evidence: `crates/nako-core/src/user_playback.rs`, `crates/nako-db/src/user_playback.rs`, `crates/nako-db/migrations/0030_user_playback_states.sql`, `crates/nako-server/src/app/user_playback.rs`, repository/server/auth tests.
  Handoff: Complete. UPS-030 can expose protocol DTOs, HTTP routes, OpenAPI, and SDK surface against the implemented service.

## M2 - Public API And SDK Surface

- [x] UPS-030 [owner=codex] [deps=UPS-020] [scope=crates/nako-client-protocol, crates/nako-api, crates/nako-client, sdk/typescript, docs/api, crates/nako-server]
  Goal: Expose the public routes through DTOs, OpenAPI, Rust SDK, TypeScript SDK, and HTTP API docs.
  Validation: `cargo nextest run -p nako-api -p nako-client --no-fail-fast`; `npm run check --prefix sdk/typescript`
  Review: ensure route schemas do not expose local paths, source locators, session internals, or token material.
  Evidence: `crates/nako-client-protocol/src/catalog.rs`, `crates/nako-api/src/openapi.rs`, `crates/nako-server/src/http/user_playback.rs`, `crates/nako-client/src/lib.rs`, `sdk/typescript/src/index.ts`, `docs/api/HTTP_API.md`, API/SDK/server route tests.
  Handoff: Complete. UPS-040 can implement Android client/UI behavior against the final `/users/me/playback-state/...` route names and DTOs.

## M3 - Android Authoritative Resume Integration

- [x] UPS-040 [owner=codex] [deps=UPS-030] [scope=apps/android/app/src/main/java/dev/nako/android, apps/android/app/src/test/java/dev/nako/android]
  Goal: Add Android client methods and UI integration for server-authoritative resume, progress reporting, watched transitions, and Continue Watching presentation.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  Review: device-local resume remains fallback/local cache and UI copy does not claim server state when routes fail.
  Evidence: `NakoUserPlaybackClientTest`, `BrowseResumeStateTest`, `UserPlaybackReportingTest`, `PlayerPresentationTest`, and full Android unit gate.
  Handoff: Complete. UPS-050 can add emulator smoke evidence for the server-backed Continue Watching path and then close or split follow-ons.

## M4 - Smoke Evidence And Closeout

- [x] UPS-050 [owner=planner] [deps=UPS-040] [scope=apps/android/scripts, docs/workstreams/user-playback-state-contract]
  Goal: Add or update smoke evidence proving Continue Watching is backed by server **User Playback State**, then close or split remaining follow-ons.
  Validation: `pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States profile-with-media`; `git diff --check`
  Review: review-workstream has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, smoke report path, `WORKSTREAM.json`.
  Handoff: Split multi-user account UI, offline sync, and recommendations into later lanes.
