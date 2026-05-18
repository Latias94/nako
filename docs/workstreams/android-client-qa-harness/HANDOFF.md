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

`ACQ-020` is complete. The smoke script and Android README updates have landed
locally, Android unit/build gates pass, and the smoke command now produces
repeatable emulator evidence.

## Next Task

Continue with `ACQ-030`: formalize repeatable fixture/state behavior for setup,
Home, Settings, Server Profile, and future detail/player smoke checks.

Recommended implementation order:

1. Decide what minimal seeded state the next emulator surfaces need.
2. Add fixture helpers or test support where they improve repeatability.
3. Keep the smoke command and evidence path documented for local reuse.
4. Record new evidence paths in `EVIDENCE_AND_GATES.md` without committing
   generated screenshots.

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
