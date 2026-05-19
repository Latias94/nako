# Android Smoke Regression Harness - TODO

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

- [x] ASR-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-smoke-regression-harness]
  Goal: Open the Android smoke regression lane and freeze the local-only test
  boundary.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: `docs/workstreams/android-smoke-regression-harness/DESIGN.md`
  Handoff: Completed on 2026-05-19. The first execution slice is a local
  regression wrapper that composes existing smoke fixture states; it must not
  duplicate UI navigation logic or invent fake server-backed data.

## M1 - Local Regression Command

- [x] ASR-020 [owner=codex] [deps=ASR-010] [scope=apps/android/scripts,apps/android/README.md,apps/android/SMOKE_FIXTURES.md]
  Goal: Add one documented local regression command that builds once, runs the
  stable smoke state set, and writes a summary report linking each evidence
  directory.
  Validation:
  `pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media`
  plus `git diff --check`.
  Review: Use review-workstream before accepting completion if behavior changes
  beyond a thin wrapper.
  Evidence: `EVIDENCE_AND_GATES.md`, generated regression report path summary.
  Handoff: DONE on 2026-05-19. Added `Smoke-Regression.ps1`, documented it in
  Android README and smoke fixture docs, hardened UI hierarchy capture for
  UiAutomation transient empty-root failures, and validated the full stable
  state set. Latest report:
  `apps/android/build/smoke-regression/20260519-005118/report.md`.

## M2 - Failure Classification And Developer Handoff

- [ ] ASR-030 [owner=codex] [deps=ASR-020] [scope=apps/android/scripts,docs/workstreams/android-smoke-regression-harness]
  Goal: Ensure wrapper failures identify the failed state and preserve the
  evidence path so another developer or agent can continue diagnosis.
  Validation:
  run a successful local regression, or record a controlled environment failure
  with the exact state, command, and evidence directory.
  Review: Use review-workstream for workstream compliance and script hygiene.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`.
  Handoff: Split retry/ADB reconnect hardening if it grows beyond simple
  wrapper behavior.

## M3 - Closeout

- [ ] ASR-040 [owner=planner] [deps=ASR-030] [scope=docs/workstreams/android-smoke-regression-harness]
  Goal: Verify the harness, update evidence, and close or split CI/golden/deep
  playback follow-ons.
  Validation: fresh local regression command, Android unit/build gates if
  touched behavior warrants them, and `git diff --check`.
  Review: Use review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Summarize remaining risks in HANDOFF.md.
