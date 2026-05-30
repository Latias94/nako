# Android Client Follow-On Hardening — Handoff

Status: Complete
Last updated: 2026-05-29

## Current State

Workstream closed after implementing the three requested follow-ons:

1. Android smoke evidence.
2. TokenVault migration away from deprecated token storage risk.
3. PlayerRuntime platform capability slice, prioritizing MediaSession/PiP if
   safe.

## Active Task

- None. ACFH-090 is complete.

## Next Recommended Action

No remaining task in this lane. Re-run end-to-end Android smoke on stable device
or emulator infrastructure before using smoke as release evidence.

## Completed Work

- ACFH-020: smoke evidence recorded as DONE_WITH_CONCERNS. Do not claim
  end-to-end smoke PASS; ADB/emulator instability is documented with report
  paths and script hardening.
- ACFH-030: TokenVault migrated away from deprecated AndroidX Security
  `EncryptedSharedPreferences` for the default app build. A migration-source
  seam preserves compatible read-through migration without retaining the
  deprecated dependency.
- ACFH-040: PlayerRuntime owns the platform session lifecycle. The safe vertical
  slice includes Android framework MediaSession and a guarded PiP entrypoint.
- ACFH-090: Closeout is complete. During final validation the Android playback
  SDK adapter was aligned to the current public contract: decision reasons use
  wire values, denied decisions are represented explicitly, and the removed
  client transcode hardware-acceleration field was deleted from the Android
  playback model.

## Fresh Validation

- `apps\android\gradlew.bat -p apps\android --no-daemon --no-parallel --console=plain :app:testDebugUnitTest --tests "dev.nako.android.connection.*" --tests "dev.nako.android.ui.screens.player.*"`:
  PASS on 2026-05-29.
- `apps\android\gradlew.bat -p apps\android --no-daemon --no-parallel --console=plain :app:testDebugUnitTest`:
  PASS on 2026-05-29.
- `apps\android\gradlew.bat -p apps\android --no-daemon --no-parallel --console=plain :app:assembleDebug -PnakoRustAndroidAbis=x86_64`:
  PASS on 2026-05-29 after installing the missing Rust
  `x86_64-linux-android` target.
- PowerShell parser check for smoke scripts: PASS.
- Deprecated token-vault crypto grep: PASS.
- `python -m json.tool docs\workstreams\android-client-follow-on-hardening\WORKSTREAM.json`:
  PASS on 2026-05-29.
- `git diff --check`: PASS on 2026-05-29 with CRLF normalization warnings only.

## Residual Risks

- End-to-end Android smoke remains environment-blocked by local ADB/emulator and
  Windows socket instability. Re-run smoke on a stable emulator or physical
  device before using smoke as release evidence.
- Legacy token migration can only migrate from a compatible provider injected
  through `TokenVaultMigrationSource`; the default build intentionally does not
  retain AndroidX Security crypto solely to read old `EncryptedSharedPreferences`.

## Guardrails

- Keep responses in Chinese; code/docs in English.
- Do not claim smoke passed unless the smoke script reports PASS.
- Do not commit generated `apps/android/build/` report artifacts.
- Keep bearer tokens out of UI, diagnostics, logs, saved state, and committed
  evidence.
- Do not use `git restore`, `git checkout`, `git reset`, stash, or destructive
  cleanup to remove changes that may belong to the user.
