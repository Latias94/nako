# Android Route Back-Stack Refactor - Milestones

Status: Closed
Last updated: 2026-05-19

## M0 - Boundary Freeze

Exit criteria:

- Workstream docs exist and agree on Android browse shell navigation scope.
- Jetpack Navigation, deep links, and process-death restoration are deferred.

Status: Complete.

## M1 - Route Stack Model And Shell Wiring

Exit criteria:

- Route stack model has focused JVM tests.
- `NakoBrowseShell` uses navigation state open/back/destination selection
  instead of assigning return targets in each screen callback.
- Existing detail, facet, player, settings, and root flows compile and behave
  through the same model.

Status: Complete.

## M2 - Smoke Return-Path Evidence

Exit criteria:

- `profile-with-media` smoke proves Detail -> Facet -> Back -> Detail.
- Existing local resume and playback smoke surfaces still pass.
- Three-state smoke regression passes.

Status: Complete.

## M3 - Closeout

Exit criteria:

- Evidence docs reference final reports.
- TODO, DESIGN, HANDOFF, and WORKSTREAM status are closed.
- Follow-ons such as deep links and saved-state restoration remain split.

Status: Complete.
