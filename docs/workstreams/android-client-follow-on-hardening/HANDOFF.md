# Android Client Follow-On Hardening — Handoff

Status: Active
Last updated: 2026-05-22

## Current State

Workstream opened and implemented to handle the three requested follow-ons:

1. Android smoke evidence.
2. TokenVault migration away from deprecated token storage risk.
3. PlayerRuntime platform capability slice, prioritizing MediaSession/PiP if
   safe.

## Active Task

- Task ID: ACFH-090
- Owner: planner
- Scope: closeout, final evidence review, and commit confirmation.
- Status: READY

## Next Recommended Action

Run ACFH-090:

1. Validate `WORKSTREAM.json`.
2. Run `git diff --check`.
3. Confirm focused and broad Android gates remain green.
4. Write `CLOSEOUT.md`.
5. Ask for commit confirmation before committing.

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

## Fresh Validation

- `apps\android\gradlew.bat -p apps\android --offline --no-daemon --no-parallel --console=plain :app:testDebugUnitTest --tests "dev.taru.android.connection.*" --tests "dev.taru.android.ui.screens.player.*"`:
  PASS.
- `apps\android\gradlew.bat -p apps\android --offline --no-daemon --no-parallel --console=plain :app:testDebugUnitTest`:
  PASS.
- `apps\android\gradlew.bat -p apps\android --offline --no-daemon --no-parallel --console=plain :app:assembleDebug -PtaruRustAndroidAbis=x86_64`:
  PASS.
- PowerShell parser check for smoke scripts: PASS.
- Deprecated token-vault crypto grep: PASS.
- `git diff --check`: PASS with CRLF normalization warnings only.

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
