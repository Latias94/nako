# Android Material Expressive UI — Milestones

Status: Active
Last updated: 2026-05-18

## M0 — Scope And Evidence Freeze

Exit criteria:

- V2 is recorded as the initial implementation target.
- V3 irregular geometry is explicitly deferred.
- Material 3 Expressive interpretation is recorded.
- Dynamic color, artwork accents, and animation boundaries are recorded.
- First executable task is selected.

Primary evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`

## M1 — UI Foundation Rewrite

Exit criteria:

- Theme and token layer support dark-first static roles and optional dynamic
  color.
- Motion tokens and artwork-accent hooks exist.
- Adaptive app chrome supports phone bottom navigation and wider navigation
  rail behavior.
- Existing top-level routes still compile and run through the new shell.
- `AME-020` is complete and validated.

Primary gates:

- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
- `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`
- `git diff --check`

## M2 — Home And Browse V2

Exit criteria:

- Home reads as playback-first and media-led.
- Libraries remain reliable structural anchors.
- Browse Facet Result handles only Public Client API backed facets as real
  results.
- Empty and loading states use shared components.
- `AME-030` is the next executable slice.

## M3 — Detail And Source Picker V2

Exit criteria:

- Detail page presents a clear playback decision surface.
- Source / Version Picker explains source choice and playback-mode
  consequences without leaking internals.
- Metadata relationship chips route to supported browse results or explicit
  API-gap states.

## M4 — Player And Settings V2

Exit criteria:

- Player remains immersive and quiet with understandable loading/error/exit
  behavior.
- Settings and Server Profile are restrained, safe, and consistent with the
  design system.
- Token values and unsafe diagnostics remain hidden.

## M5 — Verification And Closeout

Exit criteria:

- All Android gates pass fresh.
- Relevant Rust/public API gates pass when touched or before closeout.
- Workstream docs reflect shipped behavior.
- Follow-on work is split for API gaps, downloads, authoritative User Playback
  State, external player, or V3 exploration.
