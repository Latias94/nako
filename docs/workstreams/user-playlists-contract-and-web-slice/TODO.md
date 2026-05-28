# User Playlists Contract And Web Slice - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Open Lane

- [x] UPCW-010 [owner=planner] [deps=WDRP-050] [scope=docs/workstreams/user-playlists-contract-and-web-slice]
  Goal: Open the playlist contract lane and record the WDRP-050 readiness decision.
  Validation: `python -m json.tool docs/workstreams/user-playlists-contract-and-web-slice/WORKSTREAM.json`; `git diff --check -- docs/workstreams/user-playlists-contract-and-web-slice`.
  Evidence: Initial design, contract readiness decision, task ledger, and WDRP-050 update.
  Handoff: DONE. Next task is UPCW-020.

## M1 - Public Contract Freeze

- [ ] UPCW-020 [owner=Codex] [deps=UPCW-010] [scope=docs/api/HTTP_API.md,crates/nako-client-protocol,crates/nako-api,docs/workstreams/user-playlists-contract-and-web-slice]
  Goal: Freeze User Playlist vocabulary, route inventory, DTOs, access-filtering behavior, duplicate/order semantics, and SDK expectations.
  Validation: contract docs updated; focused protocol/API tests or snapshots added when code changes; `cargo fmt --all -- --check`; `git diff --check`.
  Review: verify User Playlist is distinct from catalog Collection, HLS transport playlist, and User Playback State progress.
  Evidence: `CONTRACT_READINESS.md`, public protocol DTO tests, and HTTP API notes.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M2 - Backend Persistence And App Service

- [ ] UPCW-030 [owner=Codex] [deps=UPCW-020] [scope=crates/nako-core,crates/nako-db,crates/nako-server/src/app]
  Goal: Implement principal-scoped playlist records, ordered membership persistence, and app-service validation.
  Validation: `cargo nextest run -p nako-db playlist --no-fail-fast`; focused app-service tests; `cargo fmt --all -- --check`.
  Review: no bearer tokens, canonical metadata writes, media source writes, NFO writes, or library-file writes.
  Evidence: repository and app-service tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M3 - Public API, SDKs, And Access Enforcement

- [ ] UPCW-040 [owner=Codex] [deps=UPCW-030] [scope=crates/nako-api,crates/nako-server/src/http,sdk/typescript,crates/nako-client]
  Goal: Expose `/users/me/playlists` routes through Public Client API, OpenAPI, TypeScript SDK, and Rust client with effective Library Access filtering.
  Validation: focused API/server route tests; SDK generation check; `cargo nextest run -p nako-api playlist --no-fail-fast`; `cargo nextest run -p nako-server user_playlist --no-fail-fast`.
  Review: Public DTOs must not expose admin policy rows, internal principal ids, source locators, or inaccessible item facts.
  Evidence: route tests, generated SDK diff, and HTTP API notes.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M4 - Web First Slice

- [ ] UPCW-050 [owner=Codex] [deps=UPCW-040] [scope=web/src/api/public,web/src/features/media,web/src/shell,web/src/test]
  Goal: Restore the first playlist UI in `web/` using live Public Client data with fixture fallback and route-owned state.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke.
  Review: no fixture-only playlist claims and no Admin API imports.
  Evidence: data-source contract tests, route tests, and browser smoke notes.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M5 - Closeout

- [ ] UPCW-060 [owner=planner] [deps=UPCW-050] [scope=docs/workstreams/user-playlists-contract-and-web-slice]
  Goal: Close the lane with backend/API/SDK/web evidence and split follow-ons for sharing, smart playlists, recommendation-generated lists, or offline sync.
  Validation: final backend and frontend gates recorded; JSON validation; `git diff --check`.
  Review: no blocking workstream or code-quality findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md`.
  Handoff: DONE. Return to WDRP or selected follow-on.
