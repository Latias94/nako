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

## Next Task

Run `ACQ-020`: add a documented local smoke command for build, install, launch,
and basic evidence capture against an already running emulator.

Recommended implementation order:

1. Inspect current `apps/android` build outputs, package name, activity, and
   existing README guidance.
2. Add a small PowerShell smoke script under `apps/android/scripts/`.
3. Document prerequisites and output paths in `apps/android/README.md`.
4. Run Android unit tests, debug assemble, the smoke command on the emulator,
   and `git diff --check`.
5. Record generated evidence paths in `EVIDENCE_AND_GATES.md` without
   committing generated screenshots.

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
