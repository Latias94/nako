# Android Client QA Harness Handoff

Status: Active
Last updated: 2026-05-18

## Current State

The workstream is open. `ACQ-010` is complete: the lane is scoped to local
Android client testing, emulator smoke checks, screenshot evidence, and fixture
state strategy.

The Android Client Foundation and Android Material Expressive UI lanes are
closed. This lane should improve confidence for future parallel Android work,
not add product features.

`ACQ-020`, `ACQ-030`, and `ACQ-040` are complete. The smoke script and Android README
updates have landed locally, Android unit/build gates pass, and the smoke
command now produces repeatable emulator evidence. Fixture/state rules live in
`apps/android/SMOKE_FIXTURES.md`, with `current-state`, `empty-setup`, and
`profile-missing-token` modes documented.

`empty-setup` captures setup evidence. `profile-missing-token` seeds one local
Server Profile with no token value, then captures Home, Settings, and Server
Profile shell evidence without fake media or server-backed state.

## Next Task

Continue with `ACQ-050`: verify the harness and close or split follow-ons.

Recommended implementation order:

1. Run Android unit tests and debug assemble fresh.
2. Run both documented smoke fixture states on an emulator.
3. Run `git diff --check`.
4. Decide whether to close the lane or split CI/golden/server-backed demo
   fixture follow-ons.

## Constraints To Preserve

- Do not commit generated screenshots by default.
- Do not depend on AGPL server/internal crates from Android.
- Do not expose token values, token references, local server paths, FFmpeg
  commands, or unsafe diagnostics in fixture data or reports.
- Do not fake server-backed User Playback State or facets as real client data.
- Keep the harness local-friendly before considering CI or golden screenshots.

## Open Risks

- Emulator state can make smoke checks flaky. Mitigation: document state
  assumptions and add explicit setup/cleanup where practical.
- Detail/player smoke may require server-backed demo data. Mitigation: split
  server fixture work instead of hiding it in Android.
- Screenshot automation can become brittle. Mitigation: start with launch and
  named capture evidence before adding visual assertions.
