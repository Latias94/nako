# Android Client Follow-On Hardening — Closeout

Status: Closed
Date: 2026-05-29

## Scope Closed

- ACFH-020: Android smoke evidence was run and recorded as
  DONE_WITH_CONCERNS. End-to-end smoke is not claimed PASS because local
  ADB/emulator instability blocked reliable completion.
- ACFH-030: Android TokenVault was migrated away from deprecated AndroidX
  Security `EncryptedSharedPreferences` for the default app build.
- ACFH-040: PlayerRuntime now owns the first platform capability slice:
  Android framework MediaSession plus a guarded Picture-in-Picture entrypoint.
- ACFH-090: Closeout completed with fresh gate evidence. The final validation
  pass also corrected Android public SDK drift in playback decision handling:
  denied decisions are explicit, decision reasons use wire values, and the
  removed client transcode hardware-acceleration field is no longer modeled on
  Android.

## Validation

- Focused Android JVM tests passed:
  `apps\android\gradlew.bat -p apps\android --no-daemon --no-parallel --console=plain :app:testDebugUnitTest --tests "dev.nako.android.connection.*" --tests "dev.nako.android.ui.screens.player.*"`
- Full Android JVM tests passed:
  `apps\android\gradlew.bat -p apps\android --no-daemon --no-parallel --console=plain :app:testDebugUnitTest`
- x86_64 debug assemble passed:
  `apps\android\gradlew.bat -p apps\android --no-daemon --no-parallel --console=plain :app:assembleDebug -PnakoRustAndroidAbis=x86_64`
- `python -m json.tool docs\workstreams\android-client-follow-on-hardening\WORKSTREAM.json`
  passed.
- `git diff --check` passed.

Notes:

- The first x86_64 assemble attempt failed because the local Rust toolchain did
  not have `x86_64-linux-android` installed. After
  `rustup target add x86_64-linux-android`, the gate passed.
- The debug assemble emitted non-blocking strip warnings for native libraries;
  APK packaging still completed successfully.

## Residual Risks

- Re-run smoke on a stable emulator or physical Android device before treating
  smoke as release evidence.
- The default build does not retain deprecated AndroidX Security solely for old
  `EncryptedSharedPreferences` reads. Compatible legacy migration can be wired
  through `TokenVaultMigrationSource` if an app-distribution migration build is
  required later.

## Commit Guidance

Suggested conventional commit:

```text
fix(android): align playback client with public SDK contract
```
