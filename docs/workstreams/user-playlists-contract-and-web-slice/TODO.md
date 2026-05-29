# User Playlists Contract And Web Slice - TODO

Status: Active
Last updated: 2026-05-29

## M0 - Open Lane

- [x] UPCW-010 [owner=planner] [deps=WDRP-050] [scope=docs/workstreams/user-playlists-contract-and-web-slice]
  Goal: Open the playlist contract lane and record the WDRP-050 readiness decision.
  Validation: `python -m json.tool docs/workstreams/user-playlists-contract-and-web-slice/WORKSTREAM.json`; `git diff --check -- docs/workstreams/user-playlists-contract-and-web-slice`.
  Evidence: Initial design, contract readiness decision, task ledger, and WDRP-050 update.
  Handoff: DONE. Next task is UPCW-020.

## M1 - Public Contract Freeze

- [x] UPCW-020 [owner=Codex] [deps=UPCW-010] [scope=docs/api/HTTP_API.md,crates/nako-client-protocol,crates/nako-api,docs/workstreams/user-playlists-contract-and-web-slice]
  Goal: Freeze User Playlist vocabulary, route inventory, DTOs, access-filtering behavior, duplicate/order semantics, and SDK expectations.
  Validation: contract docs updated; focused protocol/API tests or snapshots added when code changes; `cargo fmt --all -- --check`; `git diff --check`.
  Review: verify User Playlist is distinct from catalog Collection, HLS transport playlist, and User Playback State progress.
  Evidence: DONE. `CONTRACT.md`, `CONTRACT_READINESS.md`, Public Client route
  inventory, protocol DTO tests, OpenAPI schemas/tests, generated
  TypeScript/Kotlin SDK entries, and HTTP API notes freeze current-user private
  playlist semantics.
  Handoff: DONE. Next task is UPCW-030.

## M2 - Backend Persistence And App Service

- [x] UPCW-030 [owner=Codex] [deps=UPCW-020] [scope=crates/nako-core,crates/nako-db,crates/nako-server/src/app]
  Goal: Implement principal-scoped playlist records, ordered membership persistence, and app-service validation.
  Validation: `cargo nextest run -p nako-db playlist --no-fail-fast`; focused app-service tests; `cargo fmt --all -- --check`.
  Review: no bearer tokens, canonical metadata writes, media source writes, NFO writes, or library-file writes.
  Evidence: DONE. Core User Playlist records/repository trait, SQLite/PostgreSQL
  baseline schema/adapters, NakoDatabase facade, database contract test, and
  `UserPlaylistAppService` tests cover principal scope, idempotent membership,
  ordering/reorder, stale version conflicts, name validation, and media item
  existence.
  Handoff: DONE. Next task is UPCW-040.

## M3 - Public API, SDKs, And Access Enforcement

- [x] UPCW-040 [owner=Codex] [deps=UPCW-030] [scope=crates/nako-api,crates/nako-server/src/http,sdk/typescript,crates/nako-client]
  Goal: Expose `/users/me/playlists` routes through Public Client API, OpenAPI, TypeScript SDK, and Rust client with effective Library Access filtering.
  Validation: focused API/server route tests; SDK generation check; `cargo nextest run -p nako-api playlist --no-fail-fast`; `cargo nextest run -p nako-server user_playlist --no-fail-fast`.
  Review: Public DTOs must not expose admin policy rows, internal principal ids, source locators, or inaccessible item facts.
  Evidence: DONE. Public Client `/users/me/playlists` HTTP routes, DTO
  mapping, access-filtered item responses/counts, Rust client methods, SDK
  inventory assertions, and TypeScript SDK check are implemented and validated.
  Handoff: DONE. Next task is UPCW-050.

## M4 - Web First Slice

- [x] UPCW-050 [owner=Codex] [deps=UPCW-040] [scope=web/src/api/public,web/src/features/media,web/src/shell,web/src/test]
  Goal: Restore the first playlist UI in `web/` using live Public Client data with fixture fallback and route-owned state.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke.
  Review: no fixture-only playlist claims and no Admin API imports.
  Evidence: DONE. Public media data source now maps live
  `/users/me/playlists` and playlist item responses with fixture fallback;
  `web/` exposes `/media/my-list` with `playlist`/`view` route-owned state,
  TanStack Query hooks, and desktop/mobile browser smoke evidence.
  Handoff: DONE. Next task is UPCW-060.

## M5 - Closeout

- [ ] UPCW-060 [owner=planner] [deps=UPCW-050] [scope=docs/workstreams/user-playlists-contract-and-web-slice]
  Goal: Close the lane with backend/API/SDK/web evidence and split follow-ons for sharing, smart playlists, recommendation-generated lists, or offline sync.
  Validation: final backend and frontend gates recorded; JSON validation; `git diff --check`.
  Review: no blocking workstream or code-quality findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md`.
  Handoff: DONE. Return to WDRP or selected follow-on.
