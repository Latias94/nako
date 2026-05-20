# Android End-To-End Validation Hardening - TODO

Status: Closed
Last updated: 2026-05-20

## Task Ledger

- [x] AEVH-010 - Freeze validation hardening boundary.
  - Owner: Codex
  - Scope: Workstream docs only.
  - Validation: Docs identify the existing harness layers and non-goals.

- [x] AEVH-020 - Add state-level structured smoke evidence.
  - Owner: Codex
  - Scope:
    - `apps/android/scripts/Smoke-Emulator.ps1`
  - Validation:
    - PowerShell parse check for `Smoke-Emulator.ps1`.
    - Focused `Smoke-Emulator.ps1` run when an emulator is available.
  - Evidence: Parse gate and focused `empty-setup` smoke regression passed on
    2026-05-20.

- [x] AEVH-030 - Link state evidence from regression reports.
  - Owner: Codex
  - Scope:
    - `apps/android/scripts/Smoke-Regression.ps1`
  - Validation:
    - PowerShell parse check for `Smoke-Regression.ps1`.
    - Controlled no-device failure or focused smoke regression run proves JSON
      shape stays valid on failure and success paths.
  - Evidence: Controlled invalid-serial failure and focused `empty-setup`
    success run produced stable `report_markdown` and `report_json` fields on
    2026-05-20.

- [x] AEVH-040 - Verify local validation evidence chain.
  - Owner: Codex
  - Scope:
    - `apps/android/scripts/Validate-AndroidLocal.ps1`
    - Workstream docs.
  - Validation:
    - `pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke`
    - Full default validation if emulator/device state is healthy.
    - `git diff --check`
  - Evidence: `Validate-AndroidLocal.ps1 -SkipSmoke`, focused media smoke, full
    default validation, and `git diff --check` passed on 2026-05-20.

- [x] AEVH-050 - Close lane.
  - Owner: Codex
  - Scope: Workstream docs and final evidence.
  - Validation:
    - Required gates are recorded in `EVIDENCE_AND_GATES.md`.
    - No generated evidence artifacts are staged.
  - Evidence: Full default validation passed on 2026-05-20.
