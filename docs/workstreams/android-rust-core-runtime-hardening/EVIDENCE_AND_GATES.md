# Android Rust Core Runtime Hardening — Evidence And Gates

Status: Active
Last updated: 2026-05-21

## Evidence Anchors

- `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`
- `docs/workstreams/generated-sdk-runtime-ownership/CLOSEOUT.md`
- `apps/android/app/build.gradle.kts`
- `apps/android/README.md`
- `crates/taru-client-core/src/lib.rs`
- `crates/taru-client-uniffi/src/lib.rs`
- `crates/taru-client/src/lib.rs`
- `crates/taru-client-protocol/src/catalog.rs`
- `apps/android/app/src/main/java/dev/taru/android/connection`
- `apps/android/app/src/main/java/dev/taru/android/playback`

## Planning Gates

```powershell
python -m json.tool docs/workstreams/android-rust-core-runtime-hardening/WORKSTREAM.json > $null
git diff --check
```

## Task Gates

### RCR-020

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug -PtaruRustAndroidAbis=x86_64 --no-daemon
git diff --check
```

### RCR-030

```powershell
cargo fmt --package taru-client-core --package taru-client --check
cargo nextest run -p taru-client-core --no-fail-fast
cargo nextest run -p taru-client --no-fail-fast
```

### RCR-040

```powershell
cargo fmt --package taru-client-protocol --package taru-api --package taru-client --check
cargo nextest run -p taru-client-protocol --no-fail-fast
cargo nextest run -p taru-api kotlin_sdk --no-fail-fast
cargo nextest run -p taru-client --no-fail-fast
```

### RCR-050

```powershell
cargo fmt --package taru-client-core --package taru-client-uniffi --check
cargo nextest run -p taru-client-core --no-fail-fast
cargo nextest run -p taru-client-uniffi --no-fail-fast
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.* --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug -PtaruRustAndroidAbis=x86_64 --no-daemon
```

## Closeout Gates

```powershell
cargo fmt --all --check
cargo nextest run -p taru-client-core --no-fail-fast
cargo nextest run -p taru-client-uniffi --no-fail-fast
cargo nextest run -p taru-client-protocol --no-fail-fast
cargo nextest run -p taru-client --no-fail-fast
cargo nextest run -p taru-api kotlin_sdk --no-fail-fast
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug -PtaruRustAndroidAbis=x86_64 --no-daemon
python -m json.tool docs/workstreams/android-rust-core-runtime-hardening/WORKSTREAM.json > $null
git diff --check
```

## Evidence Log

- 2026-05-21: Opened lane and completed `RCR-010`.
  - Planning docs created for the serialized four-follow-on hardening lane.
  - `python -m json.tool docs/workstreams/android-rust-core-runtime-hardening/WORKSTREAM.json > $null`
    passed.
  - `git diff --check` passed.
- 2026-05-21: Completed `RCR-020` Android Rust/UniFFI build ergonomics.
  - Split `buildTaruClientUniFfiHost`, `generateTaruClientUniFfiKotlin`,
    `buildTaruClientUniFfiDebugAndroid`, and
    `buildTaruClientUniFfiReleaseAndroid`.
  - JVM test tasks now depend on host library plus generated Kotlin bindings,
    not Android ABI libraries.
  - APK JNI merge tasks build the packageable variant ABI libraries.
  - Added focused ABI selection with `-PtaruRustAndroidAbis=...`.
  - `apps/android/gradlew.bat -p apps/android :app:tasks --group "taru rust" --no-daemon`
    passed and listed the split tasks.
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon`
    passed. The output showed `buildTaruClientUniFfiHost` and
    `generateTaruClientUniFfiKotlin`, with no Android ABI build task.
  - `apps/android/gradlew.bat -p apps/android :app:assembleDebug -PtaruRustAndroidAbis=x86_64 --no-daemon`
    passed and ran `buildTaruClientUniFfiDebugAndroid` through the debug JNI
    packaging path.
  - `python -m json.tool docs/workstreams/android-rust-core-runtime-hardening/WORKSTREAM.json > $null`
    passed after command spelling was corrected.
- 2026-05-21: Completed `RCR-030` Rust client adapter reuse.
  - Added generic `CoreHttpRequestSpec`, `CoreQueryParam`,
    `build_core_request`, `interpret_core_response`, shared path encoding, and
    core request/response policy tests to `taru-client-core`.
  - `taru-client` now depends on `taru-client-core`, converts core request
    specs into reqwest requests, and maps core response-policy failures back
    into Rust client errors.
  - `cargo fmt --package taru-client-core --package taru-client --check`
    passed after formatting.
  - `cargo nextest run -p taru-client-core --no-fail-fast` passed with 9 tests.
  - `cargo nextest run -p taru-client --no-fail-fast` passed with 9 tests.
