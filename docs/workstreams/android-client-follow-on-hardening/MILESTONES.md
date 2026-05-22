# Android Client Follow-On Hardening — Milestones

Status: Active
Last updated: 2026-05-22

## M0 — Lane Open

Exit criteria:

- Workstream docs exist and agree on scope, task order, and validation gates.
- First executable task is ACFH-020.

## M1 — Smoke Evidence

Exit criteria:

- DONE_WITH_CONCERNS. Android smoke status is recorded as
  environment-blocked with partial PASS evidence.
- Report paths and blocker diagnostics are recorded.
- No unverified smoke success claim is made.

## M2 — TokenVault Migration

Exit criteria:

- DONE. Token storage avoids deprecated `EncryptedSharedPreferences` for new
  installs.
- DONE. `TokenVault` remains the only app-facing token storage interface.
- DONE. Focused token-safety tests pass.

## M3 — PlayerRuntime Capability Slice

Exit criteria:

- DONE. MediaSession is modeled behind the PlayerRuntime seam, and PiP is
  exposed through a guarded gateway.
- DONE. Broad Composable route bodies do not regain platform orchestration
  ownership.
- DONE. Focused player tests pass.

## M4 — Closeout

Exit criteria:

- Accepted tasks are complete or explicitly split.
- Fresh validation evidence is recorded.
- Residual risks and follow-ons are documented.
