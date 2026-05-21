# Android UniFFI Boundary Hardening — Evidence And Gates

Status: Closed
Last updated: 2026-05-21

## Gate Set

### Android Connection Encapsulation Gate

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon
```

Proves Android connection product logic still maps Rust-core probe outcomes,
transport failures, and diagnostics correctly after generated UniFFI types are
hidden behind the adapter seam.

### Rust Core Module Gate

```powershell
cargo fmt --package taru-client-core --check
cargo nextest run -p taru-client-core --no-fail-fast
cargo nextest run -p taru-client-uniffi --no-fail-fast
cargo nextest run -p taru-client --no-fail-fast
```

Proves the module split preserves `taru-client-core` behavior and current Rust
and UniFFI consumers.

### UniFFI Boundary Guard Gate

```powershell
./scripts/guard-uniffi-boundary.ps1
cargo nextest run -p taru-client-uniffi --no-fail-fast
```

Proves `taru-client-uniffi` remains a binding adapter and does not grow runtime
transport/platform dependencies.

### Android Native Smoke Gate

```powershell
apps/android/scripts/Validate-UniFfiNativeSmoke.ps1 -Serial <device> -Abi arm64-v8a
```

Proves selected Android ABI packaging, install, and packaged native UniFFI load
on a connected device/emulator.

### Closeout Gate

```powershell
python -m json.tool docs/workstreams/android-uniffi-boundary-hardening/WORKSTREAM.json > $null
git diff --check
```

Proves machine-readable workstream docs are valid JSON and no whitespace errors
remain in the final diff.

## Evidence Anchors

- `docs/workstreams/android-uniffi-boundary-hardening/DESIGN.md`
- `docs/workstreams/android-uniffi-boundary-hardening/TODO.md`
- `docs/workstreams/android-uniffi-boundary-hardening/MILESTONES.md`
- `crates/taru-client-core/src/*.rs`
- `crates/taru-client-uniffi/src/lib.rs`
- `apps/android/app/src/main/java/dev/taru/android/connection/RustConnectionCore.kt`
- `apps/android/scripts/Validate-UniFfiNativeSmoke.ps1`

## Evidence Log

### 2026-05-21 — UBF-010 scope freeze

- Wrote `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`,
  `HANDOFF.md`, and `WORKSTREAM.json` for this lane.
- Scope confirms the current ADR 0032 seam is correct and this lane only
  hardens adapter encapsulation, core locality, boundary drift detection, and
  device smoke repeatability.

### 2026-05-21 — UBF-020 Android adapter encapsulation

- Changed `apps/android/app/src/main/java/dev/taru/android/connection/RustConnectionCore.kt`
  so the connection adapter converts generated UniFFI probe outcomes and HTTP
  requests into Android-owned `ConnectionCoreOutcome`, `ConnectionCoreRequest`,
  and `ConnectionCoreSuccess` values.
- Changed `TaruConnectionClient` so connection product logic no longer imports
  or switches over generated UniFFI connection request/outcome types.
- Fresh gate:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon
```

Result: passed. This proves the connection flow still checks health, runs the
authenticated auth probe, maps public errors, preserves redaction, and applies
Android-owned transport/security diagnostics after generated UniFFI types were
hidden behind the adapter seam.

### 2026-05-21 — UBF-030 core module split

- Split `crates/taru-client-core/src/lib.rs` into focused modules:
  `ids`, `encoding`, `redaction`, `request`, `response`, `connection`, and
  `playback`.
- Kept `lib.rs` as the public re-export surface so existing Rust, UniFFI, and
  reqwest-client callers continue to use the same API.
- Fresh gates:

```powershell
cargo fmt --package taru-client-core --check
cargo nextest run -p taru-client-core --no-fail-fast
cargo nextest run -p taru-client-uniffi --no-fail-fast
cargo nextest run -p taru-client --no-fail-fast
```

Results: all passed. This proves the refactor preserved core request/response,
connection probe, playback target, UniFFI adapter, and Rust async client
behavior while improving module locality.

### 2026-05-21 — UBF-040 UniFFI boundary guard

- Added `scripts/guard-uniffi-boundary.ps1`.
- The guard validates that `taru-client-uniffi` direct dependencies stay within
  `taru-client-core` and `uniffi`.
- The guard also scans the dependency tree for forbidden runtime/platform
  dependencies: `reqwest`, `tokio`, `hyper`, `hyper-util`, `tower`, `axum`,
  `sqlx`, `rusqlite`, `jni`, `ndk`, and `android-activity`.
- Fresh gates:

```powershell
./scripts/guard-uniffi-boundary.ps1
cargo nextest run -p taru-client-uniffi --no-fail-fast
```

Results: both passed. The guard reported direct dependencies
`taru-client-core` and `uniffi`, checked 49 dependency-tree packages, and found
no forbidden runtime/platform dependencies.

### 2026-05-21 — UBF-050 native smoke script

- Added `apps/android/scripts/Validate-UniFfiNativeSmoke.ps1`.
- Documented native smoke usage in `apps/android/README.md`.
- The previously connected OPPO serial `3B15BC01DH500000` was no longer
  attached during this task:

```powershell
./apps/android/scripts/Validate-UniFfiNativeSmoke.ps1 -Serial 3B15BC01DH500000 -Abi arm64-v8a
```

Result: failed early with `device '3B15BC01DH500000' not found`, proving the
script does not silently run against the wrong device.

- Fresh connected-device gate:

```powershell
./apps/android/scripts/Validate-UniFfiNativeSmoke.ps1 -Serial emulator-5554 -Abi x86_64
```

Result: passed. The script verified `emulator-5554` reports ABI list
`x86_64,arm64-v8a`, built `:app:assembleDebug` and
`:app:assembleDebugAndroidTest` with `-PtaruRustAndroidAbis=x86_64`, installed
both APKs, and ran `dev.taru.android.uniffi.TaruUniFfiNativeSmokeTest` with
`OK (1 test)`.

### 2026-05-21 — UBF-090 closeout verification

- Fresh gates:

```powershell
cargo fmt --package taru-client-core --check
./scripts/guard-uniffi-boundary.ps1
cargo nextest run -p taru-client-core --no-fail-fast
cargo nextest run -p taru-client-uniffi --no-fail-fast
cargo nextest run -p taru-client --no-fail-fast
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon
python -m json.tool docs/workstreams/android-uniffi-boundary-hardening/WORKSTREAM.json > $null
git diff --check
```

Results: all passed. The native smoke script gate was already freshly run under
UBF-050 on `emulator-5554` with `-Abi x86_64`.
