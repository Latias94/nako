# Android Developer Validation Entrypoint - Milestones

Status: Closed
Last updated: 2026-05-19

## M0 - Scope And Command Contract

Exit criteria:

- Problem and target command shape are explicit.
- CI, golden, and structured report non-goals are explicit.
- First proof target is chosen.

Status: Complete.

## M1 - Local Validation Entrypoint

Exit criteria:

- `Validate-AndroidLocal.ps1` exists under `apps/android/scripts`.
- It runs Android JVM tests by default.
- It can run or skip `:app:assembleDebug`.
- It can run or skip smoke regression.
- It writes a combined report with command, status, log, and smoke report
  references.
- Android README documents the command.

Primary gates:

- `pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1 -SkipSmoke`
- `pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1`
- `git diff --check`

Primary evidence:

- `apps/android/build/validation/20260519-094914/report.md`
- `apps/android/build/validation/20260519-095005/report.md`
- `apps/android/build/smoke-regression/20260519-095037/report.md`

Status: Complete.

## M2 - Closeout

Exit criteria:

- Gate evidence is recorded.
- Follow-ons are deferred or split.
- `WORKSTREAM.json` status is updated.

Status: Complete.
