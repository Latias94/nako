# Android Client Follow-On Hardening — Closeout

Status: Ready For Commit Confirmation
Date: 2026-05-22

## Scope Closed

- ACFH-020: Android smoke evidence was run and recorded as
  DONE_WITH_CONCERNS. End-to-end smoke is not claimed PASS because local
  ADB/emulator instability blocked reliable completion.
- ACFH-030: Android TokenVault was migrated away from deprecated AndroidX
  Security `EncryptedSharedPreferences` for the default app build.
- ACFH-040: PlayerRuntime now owns the first platform capability slice:
  Android framework MediaSession plus a guarded Picture-in-Picture entrypoint.

## Validation

- PowerShell smoke scripts parse successfully.
- Deprecated token-vault crypto grep passed.
- Focused Android JVM tests passed:
  `apps\android\gradlew.bat -p apps\android --offline --no-daemon --no-parallel --console=plain :app:testDebugUnitTest --tests "dev.taru.android.connection.*" --tests "dev.taru.android.ui.screens.player.*"`
- Full Android JVM tests passed:
  `apps\android\gradlew.bat -p apps\android --offline --no-daemon --no-parallel --console=plain :app:testDebugUnitTest`
- x86_64 debug assemble passed:
  `apps\android\gradlew.bat -p apps\android --offline --no-daemon --no-parallel --console=plain :app:assembleDebug -PtaruRustAndroidAbis=x86_64`
- `git diff --check` passed with CRLF normalization warnings only.

## Residual Risks

- Re-run smoke on a stable emulator or physical Android device before treating
  smoke as release evidence.
- The default build does not retain deprecated AndroidX Security solely for old
  `EncryptedSharedPreferences` reads. Compatible legacy migration can be wired
  through `TokenVaultMigrationSource` if an app-distribution migration build is
  required later.

## Commit Guidance

Ask for user confirmation before committing. Suggested conventional commit:

```text
feat(android): harden token vault and player platform runtime
```
