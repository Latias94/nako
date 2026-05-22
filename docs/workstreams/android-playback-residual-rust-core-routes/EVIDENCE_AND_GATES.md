# Android Playback Residual Rust Core Routes — Evidence And Gates

Status: Closed
Last updated: 2026-05-21

## Gate Set

### Rust Core Gate

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
```

Proves residual playback route builders preserve core request construction,
source/session ID encoding, methods, auth header injection, and safe preview
behavior.

### UniFFI Boundary Gate

```powershell
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
```

Proves the binding surface remains thin and dependency-safe.

### Android Playback Gate

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.playback.NakoPlaybackClientTest --no-daemon --rerun-tasks
```

Proves migrated Android playback route construction, DTO decode, error mapping,
streaming/session behavior, and diagnostics still behave as before.

### Route Owner Scan

```powershell
if (rg -n "NakoPublicClientRequests|NakoRequestDescriptor|pathAndQuery|executeJson|PublicApiAuth" apps/android/app/src/main/java/dev/nako/android/playback/NakoPlaybackClient.kt apps/android/app/src/main/java/dev/nako/android/playback/RustPlaybackCore.kt) { exit 1 } else { 'PASS: playback runtime has no generated SDK route descriptor calls.' }
```

Proves migrated runtime playback code does not reintroduce generated SDK route
descriptors or the old generated-SDK executor path.

### Dead Helper Scan

```powershell
if (rg -n "toSdkPageQuery" apps/android/app/src/main/java apps/android/app/src/test/java) { exit 1 } else { 'PASS: removed dead toSdkPageQuery helper.' }
```

Proves confirmed-dead compatibility helper removal.

### Closeout Gate

```powershell
python -m json.tool docs/workstreams/android-playback-residual-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

## Evidence Anchors

- `crates/nako-client-core/src/playback.rs`
- `crates/nako-client-uniffi/src/lib.rs`
- `apps/android/app/src/main/java/dev/nako/android/playback/NakoPlaybackClient.kt`
- `apps/android/app/src/main/java/dev/nako/android/playback/RustPlaybackCore.kt`
- `apps/android/app/src/main/java/dev/nako/android/browse/BrowseSdkAdapters.kt`
- `apps/android/app/src/test/java/dev/nako/android/playback/NakoPlaybackClientTest.kt`

## Evidence Log

### 2026-05-21 — PRR-010 scope freeze

- Opened this lane and froze residual playback request-construction-only scope.
- Playback DTO decode, Android transport, UI, and server API changes are
  explicitly out of scope.
- Confirmed cleanup candidate: `PageRequest.toSdkPageQuery()` has no remaining
  callers after browse and user-playback migrations.

### 2026-05-21 — PRR-020 Rust core residual playback request builders

- Added core request builders and request IDs for source probe, playback
  session inspection, and playback session cancellation.
- Fresh gates:

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
```

Result: passed with 17 tests. The new tests prove stable methods, encoded
source/session IDs, bearer auth injection, and redaction-safe previews for the
residual playback route family.

### 2026-05-21 — PRR-030 UniFFI residual playback binding surface

- Added FFI-safe residual playback request input records and explicit builder
  functions to `crates/nako-client-uniffi/src/lib.rs`.
- Fresh gates:

```powershell
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
```

Result: passed. UniFFI now has 5 tests, including residual playback builders
for source probe, session inspection, and session cancellation. The boundary
guard still reports direct dependencies `nako-client-core` and `uniffi` with no
forbidden runtime/platform dependency.

### 2026-05-21 — PRR-040 Android playback migration and cleanup

- Extended Android `PlaybackCore`/`RustPlaybackCore` with source probe,
  playback session inspection, and playback session cancellation request
  descriptors.
- Migrated `NakoPlaybackClient` residual runtime request construction from
  `NakoPublicClientRequests` to Rust/UniFFI request descriptors.
- Deleted the confirmed-dead `PageRequest.toSdkPageQuery()` helper from
  `BrowseSdkAdapters.kt`.
- Fresh gates:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.playback.NakoPlaybackClientTest --no-daemon --rerun-tasks
if (rg -n "NakoPublicClientRequests|NakoRequestDescriptor|pathAndQuery|executeJson|PublicApiAuth" apps/android/app/src/main/java/dev/nako/android/playback/NakoPlaybackClient.kt apps/android/app/src/main/java/dev/nako/android/playback/RustPlaybackCore.kt) { exit 1 } else { 'PASS: playback runtime has no generated SDK route descriptor calls.' }
if (rg -n "toSdkPageQuery" apps/android/app/src/main/java apps/android/app/src/test/java) { exit 1 } else { 'PASS: removed dead toSdkPageQuery helper.' }
```

Result: passed. Android playback JVM tests passed; route-owner scan found no
generated SDK route descriptor calls in playback runtime files; dead-helper
scan found no `toSdkPageQuery` references. Missing-token local validation is
preserved before Rust descriptor authentication.

### 2026-05-21 — PRR-050 integration verification and docs

- Updated `apps/android/README.md` to document playback runtime route ownership:
  Rust core owns playback decision, source probe, streaming targets, HLS
  segments, and playback session inspection/cancellation request construction;
  Android owns generated SDK DTO decode, Media3, player/session presentation,
  and platform transport.
- Confirmed migrated Android playback runtime code no longer calls generated SDK
  route descriptors or the old generated-SDK executor path.
- Confirmed the dead `toSdkPageQuery` helper is removed.
- Fresh gates:

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.playback.NakoPlaybackClientTest --no-daemon --rerun-tasks
if (rg -n "NakoPublicClientRequests|NakoRequestDescriptor|pathAndQuery|executeJson|PublicApiAuth" apps/android/app/src/main/java/dev/nako/android/playback/NakoPlaybackClient.kt apps/android/app/src/main/java/dev/nako/android/playback/RustPlaybackCore.kt) { exit 1 } else { 'PASS: playback runtime has no generated SDK route descriptor calls.' }
if (rg -n "toSdkPageQuery" apps/android/app/src/main/java apps/android/app/src/test/java) { exit 1 } else { 'PASS: removed dead toSdkPageQuery helper.' }
```

Result: passed. Core ran 17 tests; UniFFI ran 5 tests; Android playback JVM
tests passed with `--rerun-tasks`; the boundary guard remained `PASS` with only
`nako-client-core` and `uniffi` as direct dependencies; route-owner and
dead-helper scans passed. Missing-token local validation is preserved before
Rust descriptor authentication.

### 2026-05-21 — PRR-090 closeout verification

- Fresh closeout gates:

```powershell
python -m json.tool docs/workstreams/android-playback-residual-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

Result: passed. Workstream JSON parsed successfully and `git diff --check` reported no whitespace errors.
