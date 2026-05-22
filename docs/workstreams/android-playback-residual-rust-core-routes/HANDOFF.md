# Android Playback Residual Rust Core Routes — Handoff

Status: Closed
Last updated: 2026-05-21

## Current State

The lane is closed. PRR-010, PRR-020, PRR-030, PRR-040, PRR-050, and PRR-090
are complete.

## Completed Outcome

- `nako-client-core` owns explicit request builders for source probe, playback
  session inspection, and playback session cancellation.
- `nako-client-uniffi` exposes thin FFI-safe residual playback builder
  records/functions.
- Android `NakoPlaybackClient` uses `PlaybackCore`/`RustPlaybackCore` for all
  runtime playback route construction.
- Android still owns transport, generated SDK playback JSON decode,
  diagnostics, Media3, UI, and product mapping.
- `PageRequest.toSdkPageQuery()` was deleted after all callers were removed.
- `apps/android/README.md` documents full playback runtime route ownership.

## Final Validation

Fresh gates passed on 2026-05-21:

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.playback.NakoPlaybackClientTest --no-daemon --rerun-tasks
python -m json.tool docs/workstreams/android-playback-residual-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

## Residual Risks / Follow-ons

- `PublicArtworkSource` still uses generated SDK route descriptors for image URL
  validation/building; split an artwork route lane if we want zero runtime SDK
  descriptor use outside previews/tests.
- Compose previews still use generated SDK descriptors for fake route matching;
  replace them with fixture-owned constants or Rust-built descriptors if needed.
- Playback DTO decode remains Kotlin SDK-owned by design.

## Next Recommended Action

Commit the closed lane, then open a follow-on workstream for public artwork route
ownership if desired.
