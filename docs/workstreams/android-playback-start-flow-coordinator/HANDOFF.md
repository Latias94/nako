# Android Playback Start Flow Coordinator - Handoff

Status: Closed
Last updated: 2026-05-19

## Completed

- APSF-020: extracted playback start coordinator/use case and wired it into
  `TaruBrowseShell`.
- APSF-030: verified targeted regressions and closed the lane.

## Notes

- Preserve active-remux semantics from the previous lane: source checking must
  not start a session; start playback must perform preflight.
- Keep UI state and navigation in `TaruBrowseShell`.
- Do not touch generated `output/` or `tmp/` directories.

## Evidence

- `PlaybackStartCoordinatorTest`: Remux preflight/session launch,
  missing-token failure, direct playback without transport preflight, and
  resume propagation.
- `PlaybackResumeResolverTest`: server resume precedence and local fallback.
- Adjacent playback tests passed: `TaruPlaybackClientTest`,
  `PlaybackLaunchTest`, and `PlaybackExitEffectsTest`.
- Full Android debug unit test suite passed with
  `:app:testDebugUnitTest --no-daemon --no-parallel`.
- `git diff --check` passed.

## Next

No continuation task remains in this workstream.
