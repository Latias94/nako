# Android Smoke Regression Harness - Milestones

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- Local-only scope and CI/golden non-goals are explicit.
- Existing Android foundation, QA harness, and server-backed fixture lanes are
  linked.
- First proof target is chosen.

Primary evidence:

- `docs/workstreams/android-smoke-regression-harness/DESIGN.md`
- `docs/workstreams/android-smoke-regression-harness/TODO.md`

Status: Complete.

## M1 - Local Regression Command

Exit criteria:

- A single local command composes stable smoke fixture states.
- The command builds once by default and reuses `Smoke-Emulator.ps1` with
  `-SkipBuild` for individual states.
- The generated summary reports each state result and evidence directory.
- Android README and smoke fixture docs mention the command.

Primary gates:

- `pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media`
- `git diff --check`

Status: Complete.

## M2 - Failure Classification And Developer Handoff

Exit criteria:

- A failed state is visible in wrapper output and the regression report.
- The last known evidence path is preserved when a state starts and then fails.
- Handoff explains how to rerun one failed state directly.

Primary gates:

- successful local regression, or a recorded environment failure with exact
  command and state.

## M3 - Closeout

Exit criteria:

- Gate set is recorded.
- Remaining CI, golden screenshot, and deeper playback work is either deferred
  or split into follow-on workstreams.
- `WORKSTREAM.json` status is updated.
