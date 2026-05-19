# User Playback State Contract Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Smallest Current Repro

Current gap: no public route exists for server-authoritative **User Playback
State** lookup/reporting, and Android only has device-local resume.

Useful reads:

```powershell
rg -n "User Playback State|Continue Watching|DevicePlayback|resume|watched|progress" CONTEXT.md docs apps/android crates -g '*.md' -g '*.kt' -g '*.rs'
```

## Gate Set

### Contract Gate

```powershell
git diff --check
```

Proves workstream docs and API contract edits are clean before implementation.

### Server Gate

```powershell
cargo nextest run -p taru-db -p taru-server user_playback --no-fail-fast
```

Proves storage, repository, principal scoping, and app-service behavior.

### API And SDK Gate

```powershell
cargo nextest run -p taru-api -p taru-client --no-fail-fast
npm run check --prefix sdk/typescript
```

Proves public DTO/OpenAPI/SDK surfaces agree.

### Android Gate

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
```

Proves Android client/UI behavior and local fallback boundaries.

### Smoke Gate

```powershell
pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States profile-with-media
git diff --check
```

Proves the emulator/server-backed user-facing path.

## Evidence Anchors

- `CONTEXT.md`
- `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- `docs/api/HTTP_API.md`
- `docs/workstreams/user-playback-state-contract/CONTRACT.md`
- `docs/workstreams/android-client-foundation/CLIENT_INTERFACE_DESIGN.md`
- `docs/workstreams/android-device-local-playback-position/`
- `docs/workstreams/android-public-client-api-coverage/`
- `crates/taru-client-protocol/`
- `crates/taru-api/`
- `crates/taru-client/`
- `crates/taru-server/`
- `apps/android/`

## Notes

- Do not mark Continue Watching complete from Android local storage.
- Do not store raw source locators, filesystem paths, tokens, or playback
  session internals in user playback state DTOs.
- Fresh verification is required before marking a task, Codex goal, or lane
  complete.

## UPS-010 Evidence

Claim: the first public **User Playback State** contract and **Single-Admin
Mode** principal semantics are frozen for implementation.

Evidence:

- `CONTRACT.md` defines `/users/me/playback-state/...` routes, DTO names,
  progress semantics, watched threshold policy, Continue Watching semantics, and
  first-slice deferrals.
- ADR-0028 defines explicit principal resolution and forbids treating bearer
  tokens or global item rows as user playback state.
- `DESIGN.md` links the frozen contract and updates the workstream from draft
  planning to active implementation.

Fresh gate evidence:

- 2026-05-19: `git diff --check` - PASS. This proves the UPS-010 ADR,
  contract, task ledger, and handoff edits have no whitespace errors.

## UPS-020 Evidence

Claim: principal-aware **User Playback State** storage and server app-service
behavior are implemented, while public HTTP/API/SDK exposure remains deferred
to UPS-030.

Evidence:

- `crates/taru-core/src/user_playback.rs` defines `UserPrincipalId`,
  `UserPlaybackState`, and write records.
- `crates/taru-core/src/repository/user_playback.rs` defines the repository
  contract for upsert, lookup, and Continue Watching state listing.
- `crates/taru-db/migrations/0029_user_playback_states.sql` persists rows by
  principal, item, and optional source.
- `crates/taru-db/src/user_playback.rs` implements SQLite persistence and
  Continue Watching filtering/sorting.
- `crates/taru-server/src/app/user_playback.rs` implements default lookup,
  progress reporting, watched/unwatched transitions, watched threshold policy,
  idempotent identical progress writes, stale progress rejection, and source
  ownership validation.
- `crates/taru-server/src/http/auth.rs` resolves accepted or explicitly
  disabled auth requests to the internal `local-admin` principal extension.

Fresh gate evidence:

- 2026-05-19: `cargo nextest run -p taru-db -p taru-server user_playback_state --no-fail-fast` - PASS, 2 tests passed. This original narrow UPS-020 gate proves repository and idempotent app-service behavior matching that filter.
- 2026-05-19: `cargo nextest run -p taru-db -p taru-server user_playback --no-fail-fast` - PASS, 8 tests passed. This is the corrected UPS-020 gate and covers DB round-trip, Continue Watching filtering/sorting, default lookup, progress, watched/unwatched, stale events, and idempotency.
- 2026-05-19: `cargo test -p taru-server require_auth_inserts_local_admin_principal --no-default-features` - PASS, 2 tests passed. This proves the inbound auth middleware resolves accepted or disabled local/test requests to the internal `local-admin` principal extension.
- 2026-05-19: `cargo fmt --all --check` - PASS.
- 2026-05-19: `git diff --check` - PASS.

## UPS-030 Evidence

Claim: public **User Playback State** lookup, Continue Watching, progress
reporting, and watched-state updates are exposed through protocol DTOs, HTTP
routes, OpenAPI, Rust SDK, TypeScript SDK, and HTTP API docs without exposing
internal principal IDs, source locators, local paths, token material, or
playback session internals.

Evidence:

- `crates/taru-client-protocol/src/catalog.rs` defines
  `UserPlaybackStateDto`, `UserPlaybackStateResponse`,
  `ContinueWatchingResponse`, `UpdatePlaybackProgressRequest`, and
  `SetWatchedStateRequest`.
- `crates/taru-client-protocol/src/lib.rs` adds the four
  `/users/me/playback-state/...` public route inventory entries.
- `crates/taru-api/src/public_client.rs` maps internal
  `UserPlaybackState` records to public DTOs, formatting millisecond timestamps
  as RFC3339 UTC strings and omitting principal IDs.
- `crates/taru-api/src/openapi.rs` exposes the same routes and schemas in the
  public OpenAPI v1 contract, with regression tests for `/users/me` semantics
  and principal redaction.
- `crates/taru-server/src/http/user_playback.rs` wires the public routes to
  `UserPlaybackAppService` using the authenticated `UserPrincipalId` request
  extension.
- `crates/taru-client/src/lib.rs` adds Rust SDK methods for state lookup,
  Continue Watching, progress updates, watched-state updates, and JSON PUT
  request bodies.
- `crates/taru-api/src/sdk.rs` and `sdk/typescript/src/index.ts` expose the
  generated TypeScript SDK methods and JSON body runtime support.
- `docs/api/HTTP_API.md` documents the route inventory, request/response
  shapes, `/users/me` scoping, and first-slice deferrals.

Fresh gate evidence:

- 2026-05-19: `cargo nextest run -p taru-client-protocol -p taru-api -p taru-client --no-fail-fast` - PASS, 47 tests passed. This proves protocol DTOs, OpenAPI contract, generated TypeScript SDK drift checks, and Rust SDK route/body behavior.
- 2026-05-19: `cargo nextest run -p taru-server user_playback --no-fail-fast` - PASS, 9 tests passed. This proves server app-service behavior plus HTTP route integration for state update/read/list and source/item validation.
- 2026-05-19: `npm run check --prefix sdk/typescript` - PASS after installing the locked TypeScript dev dependency with `npm ci --prefix sdk/typescript`. This proves the generated TypeScript SDK type-checks with `exactOptionalPropertyTypes`.
