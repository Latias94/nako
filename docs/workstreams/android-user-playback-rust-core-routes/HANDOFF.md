# Android User Playback Rust Core Routes — Handoff

Status: Closed
Last updated: 2026-05-21

## Current State

The lane is closed. UPC-010, UPC-020, UPC-030, UPC-040, UPC-050, and UPC-090
are complete.

## Completed Outcome

- `nako-client-core` owns explicit User Playback State request builders for the
  Android user-playback route family.
- `nako-client-uniffi` exposes thin FFI-safe user-playback builder
  records/functions.
- Android `NakoUserPlaybackClient` uses `UserPlaybackCore`/`RustUserPlaybackCore`
  for runtime route construction.
- Android still owns transport, generated SDK body serialization, generated SDK
  JSON decode, diagnostics, UI, and product mapping.
- `apps/android/README.md` documents generated SDK as DTO/body contract
  transition, not runtime user-playback route owner.

## Final Validation

Fresh gates passed on 2026-05-21:

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.userplayback.NakoUserPlaybackClientTest --no-daemon --rerun-tasks
python -m json.tool docs/workstreams/android-user-playback-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

## Residual Risks / Follow-ons

- User Playback State DTO/body mapping remains Kotlin SDK-owned by design;
  revisit only if cross-platform cache/offline requirements need Rust-owned
  read/write models.
- Remaining playback session/probe generated SDK route descriptors are a good
  next seam if we want all Android runtime routes to flow through Rust core.
- Add boundary guard and targeted user-playback JVM gate to CI when this
  invariant should become release-blocking.

## Next Recommended Action

Commit the closed lane, then open a follow-on workstream for remaining playback
session/probe route construction or DTO/body boundary strategy if desired.
