# Android End-To-End Validation Hardening - Milestones

Status: Active
Last updated: 2026-05-20

## M1 - Boundary Frozen

Status: Complete

Exit criteria:

- Existing harness layers and non-goals are documented.

## M2 - State Evidence Structured

Status: Complete

Exit criteria:

- `Smoke-Emulator.ps1` writes token-safe `report.json`.
- The state report includes surface evidence, readback artifacts, device, state,
  APK, fixture server metadata, and report paths.

## M3 - Regression Evidence Linked

Status: Complete

Exit criteria:

- `Smoke-Regression.ps1` links each state's `report.md` and `report.json`.
- Failure and not-run rows keep deterministic JSON shape.

## M4 - Local Gate Verified

Status: Active

Exit criteria:

- No-emulator local validation gate passes.
- Emulator-backed validation is run when available or explicitly recorded as a
  skipped gate with reason.
- Workstream evidence is closed.
