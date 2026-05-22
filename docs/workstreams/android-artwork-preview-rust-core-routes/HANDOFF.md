# Android Artwork And Preview Rust Core Routes — Handoff

Status: Closed
Last updated: 2026-05-22

## Current State

Workstream is closed. APR-010 through APR-090 are complete. Rust core, UniFFI,
and Android artwork runtime now use Rust-owned selected artwork route
construction, and browse preview fake route matching no longer uses generated
SDK descriptors.

## Next Task

None. The lane is closed.

## Key Decisions

- Runtime selected artwork route construction should be Rust core owned.
- Android still owns profile/token lookup, image DTO selection, Coil/Compose
  image loading, and transport.
- Android should validate DTO `url` by comparing it to the canonical Rust-built
  path/query for the image id, not by calling generated SDK descriptors.
- Browse preview route matching should be fixture-owned local helper code, not
  generated SDK descriptor calls.

## Validation To Keep Fresh

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.artwork.PublicArtworkTest --tests dev.nako.android.ui.artwork.ArtworkRequestResolverTest --no-daemon --rerun-tasks
apps/android/gradlew.bat -p apps/android :app:compileDebugKotlin --no-daemon
git diff --check
```

## Known Non-blocking Residuals

- Generated SDK route descriptors may remain in tests that assert SDK contract
  inventory, such as connection tests. The route-owner scan for this lane is
  limited to Android `src/main`.
