# Android UI Visual Evidence Polish - Evidence And Gates

Status: Closed
Last updated: 2026-05-19

## Required Gates

- Focused player presentation test:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.player.PlayerPresentationTest --no-daemon`
- Focused smoke:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media -SkipBuild`
- Diff hygiene:
  `git diff --check`

## Evidence Ledger

### AUP-010 - Boundary Freeze

- Evidence: `docs/workstreams/android-ui-visual-evidence-polish/DESIGN.md`
- Result: Complete.
- Notes: Lane is scoped to Player chrome overlap shown in latest smoke
  screenshots.

### AUP-020 - Player Chrome Overlap Polish

- Evidence:
  - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.player.PlayerPresentationTest --no-daemon`
  - `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media`
  - Screenshot:
    `apps/android/build/smoke/20260519-120345-profile-with-media-emulator-5554/player.png`
- Result: PASS on 2026-05-19.
- Notes: Fresh Player screenshot shows Nako context chrome above the Media3
  progress bar, time labels, and settings control. The Media3 controller remains
  enabled.

### AUP-030 - Closeout

- Evidence: `git diff --check`
- Result: PASS on 2026-05-19.
- Notes: Broader Home/Detail visual immersion remains a separate follow-on.
