# Android Client Follow-On Hardening — TODO

Status: Complete
Last updated: 2026-05-29

## Task Ledger

### M0 — Lane Open

- [x] ACFH-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-client-follow-on-hardening]
  Goal: Open the follow-on lane for smoke validation, token-vault modernization,
  and PlayerRuntime capability growth.
  Validation: workstream docs exist and agree.
  Review: Confirm this lane follows `android-client-architecture-deepening`
  closeout instead of reopening it.
  Evidence: `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`.
  Handoff: DONE. Start with ACFH-020.

### M1 — Device Or Emulator Smoke Evidence

- [x] ACFH-020 [owner=codex] [deps=ACFH-010] [scope=apps/android/scripts,apps/android/build/validation,docs/workstreams/android-client-follow-on-hardening]
  Goal: Run available Android device/emulator smoke validation and record the
  result as PASS, SKIPPED, or BLOCKED with reproducible diagnostics.
  Validation: Prefer `apps/android/scripts/Validate-AndroidLocal.ps1` without
  `-SkipSmoke`; if no device/emulator is available, run device discovery and
  record the blocker. Keep JVM/build gates fresh.
  Review: Do not claim smoke passed unless the script reports PASS. Generated
  report paths may be referenced but not committed.
  Evidence: `EVIDENCE_AND_GATES.md`, smoke/validation report path references;
  script hardening in `apps/android/scripts`.
  Handoff: DONE_WITH_CONCERNS. Smoke did not pass end-to-end. ADB/emulator
  instability is recorded as BLOCKED, with JVM/build PASS and one state-level
  smoke PASS retained as partial evidence. Continue with ACFH-030.

### M2 — TokenVault Migration

- [x] ACFH-030 [owner=codex] [deps=ACFH-020] [scope=apps/android/app/src/main/java/dev/nako/android/connection,apps/android/app/src/test/java/dev/nako/android/connection,docs/workstreams/android-client-follow-on-hardening]
  Goal: Replace or wrap deprecated `EncryptedSharedPreferences` token storage
  for new installs behind `TokenVault`, preserving token safety and migration
  behavior where possible.
  Validation: focused token-vault tests; connection/runtime token-safety tests;
  full Android JVM tests when the storage seam changes.
  Review: Bearer tokens must not appear in UI, diagnostics, saved state,
  `toString`, or committed evidence. If platform APIs require instrumentation
  tests, split the instrumentation-only verification explicitly.
  Evidence: token-vault source/test paths and evidence log.
  Handoff: DONE. New installs use an Android Keystore AES-GCM backed
  SharedPreferences vault, deprecated AndroidX Security crypto references were
  removed, and the storage seam includes a no-deprecated read-through migration
  source for compatible legacy token providers.

### M3 — PlayerRuntime Platform Capability Slice

- [x] ACFH-040 [owner=codex] [deps=ACFH-020] [scope=apps/android/app/src/main/java/dev/nako/android/ui/screens/player,apps/android/app/src/main/java/dev/nako/android,apps/android/app/src/test/java/dev/nako/android/ui/screens/player,docs/workstreams/android-client-follow-on-hardening]
  Goal: Add the first safe PlayerRuntime platform capability slice, prioritizing
  MediaSession and Picture-in-Picture if they can be modeled without broad route
  churn.
  Validation: focused PlayerRuntime tests; player route tests; manifest/activity
  checks if PiP is added; full Android JVM tests; smoke when practical.
  Review: Keep Android ownership. Do not add Cast, Android TV, downloads,
  external player handoff, or broad track-selection UX in this task.
  Evidence: player runtime source/test paths and evidence log.
  Handoff: DONE. PlayerRuntime now owns a platform-session seam backed by
  Android framework `MediaSession`, the player route exposes an Android PiP
  entrypoint when available, and the manifest declares PiP support.

### M4 — Closeout

- [x] ACFH-090 [owner=planner] [deps=ACFH-020,ACFH-030,ACFH-040] [scope=docs/workstreams/android-client-follow-on-hardening]
  Goal: Close or split the lane with final evidence, residual risks, and commit
  guidance.
  Validation: JSON validation for `WORKSTREAM.json`; `git diff --check`; focused
  gates from completed tasks; broad Android JVM/build gates.
  Review: No blocking token-safety, workstream, or player-platform ownership
  findings remain.
  Evidence: `CLOSEOUT.md`, `EVIDENCE_AND_GATES.md`, `HANDOFF.md`.
  Handoff: DONE. The lane is closed with fresh focused JVM, full JVM,
  x86_64 assemble, JSON, and diff hygiene evidence. End-to-end smoke remains
  environment-blocked and is not claimed PASS.
