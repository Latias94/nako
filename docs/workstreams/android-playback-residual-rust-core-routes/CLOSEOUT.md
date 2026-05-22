# Android Playback Residual Rust Core Routes — Closeout

Status: Closed
Closed on: 2026-05-21

## Outcome

This lane moved the remaining Android playback runtime route construction to the
shared Rust client core while preserving the ADR 0032 boundary: Android still
owns platform transport, generated SDK playback DTO decoding, diagnostics,
product mapping, Media3, player/session presentation, and profile/token state.

Completed slices:

1. `nako-client-core` now exposes explicit request builders for source probe,
   playback session inspection, and playback session cancellation.
2. `nako-client-uniffi` now exposes FFI-safe residual playback builder
   records/functions as a thin adapter over `nako-client-core`.
3. Android `PlaybackCore`/`RustPlaybackCore` now covers playback decision,
   streaming targets, HLS segments, source probe, playback session inspection,
   and playback session cancellation.
4. `NakoPlaybackClient` no longer calls generated Kotlin SDK route descriptors
   for runtime playback routes. The generated SDK remains in use for playback
   DTO decode and contract tests.
5. The confirmed-dead `PageRequest.toSdkPageQuery()` compatibility helper was
   deleted.
6. Android docs now state full playback runtime route ownership explicitly.

## Final Verification

Fresh closeout gates run on 2026-05-21:

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.playback.NakoPlaybackClientTest --no-daemon --rerun-tasks
if (rg -n "NakoPublicClientRequests|NakoRequestDescriptor|pathAndQuery|executeJson|PublicApiAuth" apps/android/app/src/main/java/dev/nako/android/playback/NakoPlaybackClient.kt apps/android/app/src/main/java/dev/nako/android/playback/RustPlaybackCore.kt) { exit 1 } else { 'PASS: playback runtime has no generated SDK route descriptor calls.' }
if (rg -n "toSdkPageQuery" apps/android/app/src/main/java apps/android/app/src/test/java) { exit 1 } else { 'PASS: removed dead toSdkPageQuery helper.' }
python -m json.tool docs/workstreams/android-playback-residual-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

All gates passed. Core ran 17 tests, UniFFI ran 5 tests, and Android playback
JVM tests passed with `--rerun-tasks`. The UniFFI boundary guard reported only
`nako-client-core` and `uniffi` as direct dependencies and no forbidden
runtime/platform dependency.

## Residual Risks

- Playback DTO decoding remains Kotlin-side through the generated public-client
  SDK. That is intentional for this lane; revisit only if Rust-owned read models
  become necessary for offline/cache or cross-platform parity.
- `PublicArtworkSource` still uses generated SDK request descriptors to validate
  public image URLs. That is not playback runtime route construction and should
  be split into an artwork/core route lane if we want zero Android runtime route
  descriptor usage outside previews/tests.
- Compose previews still use generated SDK descriptors to match fake transport
  URLs. Those are preview/test helpers, not runtime policy, but can be cleaned
  once preview fixtures have their own stable fake-route layer.
- Android playback JVM tests prove route construction, DTO decode, diagnostics,
  and session behavior. Device smoke remains covered by broader Android
  validation lanes and was not expanded here.

## Recommended Follow-ons

1. Move public artwork image request validation/building to Rust core or to an
   Android-owned non-SDK validator so artwork no longer depends on generated SDK
   route descriptors at runtime.
2. Replace Compose preview generated SDK route matching with fixture-owned route
   constants or Rust-built descriptors if we want previews to mirror production
   ownership exactly.
3. Add the UniFFI boundary guard and targeted playback JVM gate to CI if this
   seam becomes release-blocking.
4. Keep generated SDK request descriptors available for contract tests and API
   inventory, but avoid reintroducing them into Android runtime route policy.
