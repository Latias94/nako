# Android Artwork And Preview Rust Core Routes — Evidence And Gates

Status: Closed
Last updated: 2026-05-22

## Gate Set

### Rust Core Gate

```powershell
cargo fmt --package taru-client-core --check
cargo nextest run -p taru-client-core --no-fail-fast
```

Proves selected artwork image route construction is encoded, authenticated, and
redaction-safe in Rust core.

### UniFFI Boundary Gate

```powershell
cargo fmt --package taru-client-uniffi --check
cargo nextest run -p taru-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
```

Proves the binding surface remains thin and dependency-safe.

### Android Artwork Gate

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.artwork.PublicArtworkTest --tests dev.taru.android.ui.artwork.ArtworkRequestResolverTest --no-daemon --rerun-tasks
```

Proves selected artwork request construction, active profile/token behavior,
safety rejection, and redaction behavior.

### Android Compile Gate

```powershell
apps/android/gradlew.bat -p apps/android :app:compileDebugKotlin --no-daemon
```

Proves main-source preview fixture cleanup and UniFFI generated Kotlin binding
integration compile.

### Android Runtime Route Owner Scan

```powershell
if (rg -n "TaruPublicClientRequests|TaruRequestDescriptor|PublicApiRequestDescriptors|urlOn\(" apps/android/app/src/main/java) { exit 1 } else { 'PASS: Android main has no generated SDK route descriptor use.' }
```

Proves Android `src/main` no longer uses generated SDK route descriptors as
runtime or preview route policy.

### Preview Fixture Scan

```powershell
if (rg -n "TaruPublicClientRequests|PageQuery|pathAndQuery" apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShellPreview.kt) { exit 1 } else { 'PASS: browse preview has no generated SDK route matching.' }
```

Proves the Compose preview fake transport does not teach generated SDK route
ownership.

### Closeout Gate

```powershell
python -m json.tool docs/workstreams/android-artwork-preview-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

## Evidence Anchors

- `crates/taru-client-core/src/artwork.rs`
- `crates/taru-client-uniffi/src/lib.rs`
- `apps/android/app/src/main/java/dev/taru/android/artwork/PublicArtwork.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShellPreview.kt`
- `apps/android/app/src/test/java/dev/taru/android/artwork/PublicArtworkTest.kt`
- `apps/android/app/src/test/java/dev/taru/android/ui/artwork/ArtworkRequestResolverTest.kt`

## Evidence Log

### 2026-05-22 — APR-010 scope freeze

- Opened the lane and froze scope around selected artwork image route
  construction and preview/test route fixture cleanup.
- Confirmed non-goals: no Rust-owned image transport, Coil/Compose image
  loading, DTO decode, server API shape, or generated SDK contract removal.

### 2026-05-22 — APR-020 Rust core and UniFFI artwork request builder

- Added `taru-client-core` selected artwork image request construction for
  `GET /images/{image_id}` with optional `width` and `height` query parameters.
- Added thin UniFFI binding over the core builder.
- Fresh gates:

```powershell
cargo fmt --package taru-client-core --check
cargo nextest run -p taru-client-core --no-fail-fast
cargo fmt --package taru-client-uniffi --check
cargo nextest run -p taru-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
```

Result: passed. Core ran 18 tests; UniFFI ran 6 tests. The boundary guard
reported only `taru-client-core` and `uniffi` as direct dependencies and no
forbidden runtime/platform dependency.

### 2026-05-22 — APR-030 Android runtime artwork migration

- Added Android `ArtworkCore` / `RustArtworkCore` over the UniFFI artwork image
  request builder.
- Migrated `PublicArtworkSource` off generated SDK descriptors. It now compares
  each DTO image URL to the canonical Rust-built path/query before returning a
  request.
- Added tests for stale-route/mismatched-image rejection and injectable core
  route construction.
- Fresh gate:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.artwork.PublicArtworkTest --tests dev.taru.android.ui.artwork.ArtworkRequestResolverTest --no-daemon --rerun-tasks
```

Result: passed. The gate also regenerated current UniFFI Kotlin bindings and
compiled the Android debug/test sources.

### 2026-05-22 — APR-040 preview fixture route cleanup

- Replaced `TaruBrowseShellPreview` generated SDK route matching with local
  preview fixture route helpers.
- Deleted now-dead `PublicApiRequestDescriptors.kt`.
- Fresh gates:

```powershell
apps/android/gradlew.bat -p apps/android :app:compileDebugKotlin --no-daemon
if (rg -n "TaruPublicClientRequests|TaruRequestDescriptor|PublicApiRequestDescriptors|urlOn\\(" apps/android/app/src/main/java) { exit 1 } else { 'PASS: Android main has no generated SDK route descriptor use.' }
if (rg -n "TaruPublicClientRequests|PageQuery|pathAndQuery" apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShellPreview.kt) { exit 1 } else { 'PASS: browse preview has no generated SDK route matching.' }
```

Result: passed after tightening the preview helper parameter name so the scan no
longer finds the old `pathAndQuery` concept in preview code.

### 2026-05-22 — APR-050 integration verification and docs

- Updated `apps/android/README.md` to document selected artwork Rust-core route
  ownership and preview fixture route-helper policy.
- Confirmed Android `src/main` no longer imports generated SDK route descriptor
  APIs.
- Fresh gates:

```powershell
cargo fmt --package taru-client-core --check
cargo nextest run -p taru-client-core --no-fail-fast
cargo fmt --package taru-client-uniffi --check
cargo nextest run -p taru-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.artwork.PublicArtworkTest --tests dev.taru.android.ui.artwork.ArtworkRequestResolverTest --no-daemon --rerun-tasks
apps/android/gradlew.bat -p apps/android :app:compileDebugKotlin --no-daemon
if (rg -n "TaruPublicClientRequests|TaruRequestDescriptor|PublicApiRequestDescriptors|urlOn\\(" apps/android/app/src/main/java) { exit 1 } else { 'PASS: Android main has no generated SDK route descriptor use.' }
if (rg -n "TaruPublicClientRequests|PageQuery|pathAndQuery" apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShellPreview.kt) { exit 1 } else { 'PASS: browse preview has no generated SDK route matching.' }
python -m json.tool docs/workstreams/android-artwork-preview-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

Result: passed. Core ran 18 tests; UniFFI ran 6 tests; targeted Android
artwork/resolver JVM tests passed; Android debug Kotlin compile passed; route
owner scans passed; workstream JSON parsed; `git diff --check` found no
whitespace errors.

### 2026-05-22 — APR-090 closeout verification

- Fresh closeout gates:

```powershell
python -m json.tool docs/workstreams/android-artwork-preview-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

Result: passed. Workstream JSON parsed successfully and `git diff --check`
reported no whitespace errors.
