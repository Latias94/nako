# Android Browse/Catalog Rust Core Routes — Handoff

Status: Closed
Last updated: 2026-05-21

## Current State

The lane is closed. BCR-010, BCR-020, BCR-030, BCR-040, BCR-050, and BCR-090
are complete.

## Completed Outcome

- `nako-client-core` owns explicit browse/catalog request builders for the
  Android browse route family.
- `nako-client-uniffi` exposes thin FFI-safe browse builder records/functions.
- Android `NakoBrowseClient` uses `BrowseCore`/`RustBrowseCore` for runtime
  route construction.
- Android still owns transport, JSON decode through generated SDK DTOs,
  diagnostics, UI, and product mapping.
- `apps/android/README.md` documents generated SDK as DTO/contract transition,
  not runtime browse route owner.

## Final Validation

Fresh gates passed on 2026-05-21:

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.browse.NakoBrowseClientTest --no-daemon --rerun-tasks
python -m json.tool docs/workstreams/android-browse-catalog-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

## Residual Risks / Follow-ons

- User-playback route construction still has generated SDK helper usage and is a
  good next seam to migrate.
- Browse DTO decode remains Kotlin SDK-owned by design; revisit only if
  cross-platform cache/offline requirements need Rust-owned read models.
- Add the boundary guard and targeted browse JVM gate to CI when this invariant
  should become release-blocking.

## Next Recommended Action

Commit the closed lane, then open a follow-on workstream for user-playback route
construction or DTO boundary strategy if desired.
