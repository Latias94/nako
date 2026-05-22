# Android User Playback Rust Core Routes — Closeout

Status: Closed
Closed on: 2026-05-21

## Outcome

This lane moved Android User Playback State runtime route construction to the
shared Rust client core while preserving the ADR 0032 boundary: Android still
owns platform transport, generated SDK request-body serialization, generated
SDK DTO decoding, diagnostics, product mapping, profile/token state, UI, and
Media3.

Completed slices:

1. `nako-client-core` now exposes explicit User Playback State request builders
   for get state, Continue Watching, update progress, and set watched state.
2. `nako-client-uniffi` now exposes FFI-safe user-playback builder
   records/functions as a thin adapter over `nako-client-core`.
3. Android now uses `UserPlaybackCore`/`RustUserPlaybackCore` to convert product
   user-playback inputs into Rust-built request descriptors before executing
   Android-owned transport.
4. `NakoUserPlaybackClient` no longer calls generated Kotlin SDK route
   descriptors for migrated runtime user-playback routes. The generated SDK
   remains in use for DTO decode and request-body contract mapping.
5. Android docs now state the route ownership split explicitly.

## Final Verification

Fresh closeout gates run on 2026-05-21:

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.userplayback.NakoUserPlaybackClientTest --no-daemon --rerun-tasks
if (rg -n "NakoPublicClientRequests|NakoRequestDescriptor|pathAndQuery|executeJson" apps/android/app/src/main/java/dev/nako/android/userplayback/NakoUserPlaybackClient.kt apps/android/app/src/main/java/dev/nako/android/userplayback/RustUserPlaybackCore.kt) { exit 1 } else { 'PASS: user-playback runtime has no generated SDK route descriptor calls.' }
python -m json.tool docs/workstreams/android-user-playback-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

All gates passed. Core ran 15 tests, UniFFI ran 4 tests, and Android
user-playback ran 4 JVM tests with `--rerun-tasks`. The UniFFI boundary guard
reported only `nako-client-core` and `uniffi` as direct dependencies and no
forbidden runtime/platform dependency.

## Residual Risks

- User Playback State DTO decoding and request-body serialization remain
  Kotlin-side through the generated public-client SDK. That is intentional for
  this lane, but a future cache/offline lane may want Rust-owned read/write
  models for cross-platform consistency.
- The Rust builders are infallible request descriptor builders. Android still
  performs product validation such as missing item and missing token before
  transport.
- Android user-playback JVM tests prove route construction, DTO/body mapping,
  diagnostics, and error mapping. Device smoke remains covered by broader
  Android validation lanes and was not expanded here.
- Newly added User Playback State routes must add core + UniFFI builders instead
  of reintroducing generated Kotlin runtime route descriptors.

## Recommended Follow-ons

1. Decide whether generated SDK DTO/body ownership should remain Kotlin-side or
   move selected User Playback State wire models into Rust for offline/cache
   semantics.
2. Consider migrating remaining playback session/probe route descriptors to Rust
   core so all Android runtime Public Client API route construction follows one
   owner.
3. Add the UniFFI boundary guard and targeted user-playback JVM gate to CI if
   this seam becomes release-blocking.
4. Keep generated SDK request descriptors available for contract tests and API
   inventory, but avoid reintroducing them into Android runtime route policy.
