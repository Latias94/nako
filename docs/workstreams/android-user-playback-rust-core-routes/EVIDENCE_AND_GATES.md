# Android User Playback Rust Core Routes — Evidence And Gates

Status: Closed
Last updated: 2026-05-21

## Gate Set

### Rust Core Gate

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
```

Proves user-playback route builders preserve core request construction, item ID
encoding, pagination, methods, auth header injection, JSON content type, body
passthrough, and safe preview behavior.

### UniFFI Boundary Gate

```powershell
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
```

Proves the binding surface remains thin and dependency-safe.

### Android User Playback Gate

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.userplayback.NakoUserPlaybackClientTest --no-daemon --rerun-tasks
```

Proves migrated Android user-playback route construction, generated SDK DTO/body
mapping, error mapping, and diagnostics still behave as before.

### Route Owner Scan

```powershell
if (rg -n "NakoPublicClientRequests|NakoRequestDescriptor|pathAndQuery|executeJson" apps/android/app/src/main/java/dev/nako/android/userplayback/NakoUserPlaybackClient.kt apps/android/app/src/main/java/dev/nako/android/userplayback/RustUserPlaybackCore.kt) { exit 1 } else { 'PASS: user-playback runtime has no generated SDK route descriptor calls.' }
```

Proves migrated runtime user-playback code does not reintroduce generated SDK
route descriptors.

### Closeout Gate

```powershell
python -m json.tool docs/workstreams/android-user-playback-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

## Evidence Anchors

- `crates/nako-client-core/src/user_playback.rs`
- `crates/nako-client-uniffi/src/lib.rs`
- `apps/android/app/src/main/java/dev/nako/android/userplayback/NakoUserPlaybackClient.kt`
- `apps/android/app/src/main/java/dev/nako/android/userplayback/RustUserPlaybackCore.kt`
- `apps/android/app/src/test/java/dev/nako/android/userplayback/NakoUserPlaybackClientTest.kt`

## Evidence Log

### 2026-05-21 — UPC-010 scope freeze

- Opened this lane and froze request-construction-only scope.
- User Playback State DTO decode, request body DTO ownership, Android
  transport, UI, and server API changes are explicitly out of scope.

### 2026-05-21 — UPC-020 Rust core user-playback request builders

- Added `crates/nako-client-core/src/user_playback.rs`.
- Builders cover get state, Continue Watching, update progress, and set watched
  state.
- Fresh gates:

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
```

Result: passed with 15 tests. The new tests prove stable route paths,
pagination, item ID encoding, method selection, bearer auth injection,
redaction-safe previews, JSON content type on writes, and body passthrough.

### 2026-05-21 — UPC-030 UniFFI user-playback binding surface

- Added FFI-safe user-playback request input records and explicit builder
  functions to `crates/nako-client-uniffi/src/lib.rs`.
- Fresh gates:

```powershell
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
```

Result: passed. UniFFI now has 4 tests, including user-playback builders for
Continue Watching and write routes. The boundary guard still reports direct
dependencies `nako-client-core` and `uniffi` with no forbidden
runtime/platform dependency.

### 2026-05-21 — UPC-040 Android user-playback adapter migration

- Added
  `apps/android/app/src/main/java/dev/nako/android/userplayback/RustUserPlaybackCore.kt`.
- Migrated `NakoUserPlaybackClient` runtime request construction from
  `NakoPublicClientRequests` to `UserPlaybackCore`/Rust UniFFI request
  descriptors.
- Generated SDK DTO aliases and body DTO mapping remain in place.
- Preserved local missing-token behavior before transport, because Rust request
  builders are infallible descriptors and Android still owns product validation.
- Fresh gates:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.userplayback.NakoUserPlaybackClientTest --no-daemon --rerun-tasks
if (rg -n "NakoPublicClientRequests|NakoRequestDescriptor|pathAndQuery|executeJson" apps/android/app/src/main/java/dev/nako/android/userplayback/NakoUserPlaybackClient.kt apps/android/app/src/main/java/dev/nako/android/userplayback/RustUserPlaybackCore.kt) { exit 1 } else { 'PASS: user-playback runtime has no generated SDK route descriptor calls.' }
```

Result: passed with 4 user-playback JVM tests. The route-owner scan found no
generated SDK route descriptor calls in the migrated runtime user-playback
files.

### 2026-05-21 — UPC-050 integration verification and docs

- Updated `apps/android/README.md` to document User Playback State route
  ownership: Rust core owns request construction; Android owns request-body
  serialization, DTO decode, transport, and product diagnostics; generated
  Kotlin SDK remains the DTO/body contract transition layer.
- Confirmed migrated Android user-playback runtime code no longer calls
  generated SDK route descriptors.
- Fresh gates:

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.userplayback.NakoUserPlaybackClientTest --no-daemon --rerun-tasks
if (rg -n "NakoPublicClientRequests|NakoRequestDescriptor|pathAndQuery|executeJson" apps/android/app/src/main/java/dev/nako/android/userplayback/NakoUserPlaybackClient.kt apps/android/app/src/main/java/dev/nako/android/userplayback/RustUserPlaybackCore.kt) { exit 1 } else { 'PASS: user-playback runtime has no generated SDK route descriptor calls.' }
```

Result: passed. Core ran 15 tests; UniFFI ran 4 tests; Android user-playback
ran 4 JVM tests with `--rerun-tasks`; the boundary guard remained `PASS` with
only `nako-client-core` and `uniffi` as direct dependencies; the route-owner
scan found no generated SDK descriptor calls in the migrated user-playback
runtime files.

### 2026-05-21 — UPC-090 closeout verification

- Fresh closeout gates:

```powershell
python -m json.tool docs/workstreams/android-user-playback-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

Result: passed. Workstream JSON parsed successfully and `git diff --check`
reported no whitespace errors.
