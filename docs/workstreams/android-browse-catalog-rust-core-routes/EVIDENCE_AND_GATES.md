# Android Browse/Catalog Rust Core Routes — Evidence And Gates

Status: Closed
Last updated: 2026-05-21

## Gate Set

### Rust Core Gate

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
```

Proves browse/catalog route builders preserve core request construction,
encoding, auth header injection, and safe preview behavior.

### UniFFI Boundary Gate

```powershell
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
```

Proves the binding surface remains thin and dependency-safe.

### Android Browse Gate

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.browse.NakoBrowseClientTest --no-daemon
```

Proves migrated Android browse route construction, DTO decode, error mapping,
and diagnostics still behave as before.

### Closeout Gate

```powershell
python -m json.tool docs/workstreams/android-browse-catalog-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

## Evidence Anchors

- `crates/nako-client-core/src/browse.rs`
- `crates/nako-client-uniffi/src/lib.rs`
- `apps/android/app/src/main/java/dev/nako/android/browse/NakoBrowseClient.kt`
- `apps/android/app/src/main/java/dev/nako/android/browse/RustBrowseCore.kt`
- `apps/android/app/src/test/java/dev/nako/android/browse/NakoBrowseClientTest.kt`

## Evidence Log

### 2026-05-21 — BCR-010 scope freeze

- Opened this lane and froze request-construction-only scope.
- Browse/catalog DTO decode, Android transport, UI, and server API changes are
  explicitly out of scope.

### 2026-05-21 — BCR-020 Rust core browse request builders

- Added `crates/nako-client-core/src/browse.rs`.
- Builders cover libraries, library sources, items, item images, people, person
  item facets, genres, genre item facets, tags, tag item facets, and search.
- Fresh gates:

```powershell
cargo fmt --package nako-client-core
cargo nextest run -p nako-client-core --no-fail-fast
```

Result: passed with 13 tests. The new tests prove stable path/query encoding,
auth header injection, redaction-safe previews, facet ID encoding, and search
query/facet pagination behavior.

### 2026-05-21 — BCR-030 UniFFI browse binding surface

- Added FFI-safe browse request input records and explicit builder functions to
  `crates/nako-client-uniffi/src/lib.rs`.
- Fresh gates:

```powershell
cargo fmt --package nako-client-uniffi
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
```

Result: passed. UniFFI now has 3 tests, including browse builders for libraries,
search, and tag facet routes. The boundary guard still reports direct
dependencies `nako-client-core` and `uniffi` with no forbidden
runtime/platform dependency.

### 2026-05-21 — BCR-040 Android browse adapter migration

- Added `apps/android/app/src/main/java/dev/nako/android/browse/RustBrowseCore.kt`.
- Migrated `NakoBrowseClient` runtime request construction from
  `NakoPublicClientRequests` to `BrowseCore`/Rust UniFFI request descriptors.
- Kotlin SDK DTO aliases and `toAndroid` DTO mapping remain in place.
- Rust core now uses `%20` for search query spaces instead of the previous
  Kotlin `+` form; tests were updated to the new route owner output.
- Fresh gate:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.browse.NakoBrowseClientTest --no-daemon
```

Result: passed with 29 browse tests.

### 2026-05-21 — BCR-050 integration verification and docs

- Updated `apps/android/README.md` to document browse/catalog route ownership:
  Rust core owns request construction; Android owns transport and product
  diagnostics; generated Kotlin SDK remains the DTO/contract transition layer.
- Confirmed migrated Android browse runtime code no longer calls generated SDK
  route descriptors.
- Fresh gates:

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.browse.NakoBrowseClientTest --no-daemon --rerun-tasks
if (rg -n "NakoPublicClientRequests|pathAndQuery" apps/android/app/src/main/java/dev/nako/android/browse/NakoBrowseClient.kt apps/android/app/src/main/java/dev/nako/android/browse/RustBrowseCore.kt) { exit 1 } else { 'PASS: NakoBrowseClient/RustBrowseCore have no generated SDK route descriptor calls.' }
```

Result: passed. Core ran 13 tests; UniFFI ran 3 tests; Android browse ran
29 JVM tests with `--rerun-tasks`; the boundary guard remained `PASS` with
only `nako-client-core` and `uniffi` as direct dependencies; the route-owner
scan found no generated SDK descriptor calls in the migrated browse runtime
files.

### 2026-05-21 — BCR-090 closeout verification

- Fresh closeout gates:

```powershell
python -m json.tool docs/workstreams/android-browse-catalog-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

Result: passed. Workstream JSON parsed successfully and `git diff --check`
reported no whitespace errors.
