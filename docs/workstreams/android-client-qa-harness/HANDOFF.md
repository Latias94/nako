# Android Client QA Harness Handoff

Status: Completed
Last updated: 2026-05-18

## Current State

The workstream is closed. `ACQ-010`, `ACQ-020`, `ACQ-030`, `ACQ-040`, and
`ACQ-050` are complete.

The Android Client Foundation and Android Material Expressive UI lanes are
closed. This lane should improve confidence for future parallel Android work,
not add product features.

The smoke script and Android README updates have landed locally. Android
unit/build gates pass, and the smoke command now produces repeatable emulator
evidence. Fixture/state rules live in `apps/android/SMOKE_FIXTURES.md`, with
`current-state`, `empty-setup`, and `profile-missing-token` modes documented.

`empty-setup` captures setup evidence. `profile-missing-token` seeds one local
Server Profile with no token value, then captures Home, Settings, and Server
Profile shell evidence without fake media or server-backed state.

## Closeout Result

`ACQ-050` verified and closed the lane.

Fresh closeout gates passed on 2026-05-18:

- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
- `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`
- `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState empty-setup`
- `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-missing-token`
- `git diff --check`

Final smoke evidence:

- `apps/android/build/smoke/20260518-215542-empty-setup-emulator-5554/`
- `apps/android/build/smoke/20260518-215751-profile-missing-token-emulator-5554/`

## Constraints To Preserve

- Do not commit generated screenshots by default.
- Do not depend on AGPL server/internal crates from Android.
- Do not expose token values, token references, local server paths, FFmpeg
  commands, or unsafe diagnostics in fixture data or reports.
- Do not fake server-backed User Playback State or facets as real client data.
- Keep the harness local-friendly before considering CI or golden screenshots.

## Residual Risks

- Emulator state can make smoke checks flaky. Mitigation: document state
  assumptions and add explicit setup/cleanup where practical.
- Detail/player smoke may require server-backed demo data. Mitigation: split
  server fixture work instead of hiding it in Android.
- Screenshot automation can become brittle. Mitigation: start with launch and
  named capture evidence before adding visual assertions.

## Follow-Ons

- CI device execution for the smoke harness.
- Golden visual diffing policy and reference image storage.
- Server-backed demo fixtures for Home media content, detail, source picker,
  and player coverage.
- Instrumentation test migration when UI routes stabilize.
- Optional retry hardening around ADB daemon reconnects if closeout-style runs
  keep seeing daemon restarts.
