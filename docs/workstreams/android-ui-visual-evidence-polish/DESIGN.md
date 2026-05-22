# Android UI Visual Evidence Polish

Status: Closed
Last updated: 2026-05-19

## Why This Lane Exists

The Android V2 Material 3 Expressive baseline is functional and smoke-covered,
but visual evidence from the latest smoke run shows at least one product-grade
polish issue: the custom player bottom information panel overlaps the Media3
controller and progress bar.

The goal is not a broad redesign. This lane uses screenshot evidence to fix a
small, high-value visual problem without reopening V2 versus V3 geometry.

## Relevant Authority

- `docs/workstreams/android-material-expressive-ui/`
- `docs/workstreams/android-route-back-stack-refactor/`
- `docs/workstreams/android-navigation-state-restoration/`
- Latest smoke evidence:
  `apps/android/build/smoke-regression/20260519-112540/`

## Visual Evidence

Reviewed on 2026-05-19:

- `profile-with-media/home.png`: baseline Home is dense and usable, with media
  hero, shortcuts, and bottom navigation.
- `profile-with-media/detail.png`: detail is functional and expressive enough
  for V2, though further media immersion can be a follow-on.
- `profile-with-media/facet-genre.png`: facet result layout is readable and
  route-backed.
- `profile-with-media/player.png`: custom bottom player panel intersects the
  Media3 progress bar, playback controls, and settings affordance.

## Target State

When this lane closes:

- Player custom chrome does not overlap Media3 controls in smoke screenshots.
- The fix preserves Media3 controller ownership instead of disabling standard
  playback controls.
- Player information remains visible enough for smoke criteria and user context.
- The change is bounded to player UI layout/presentation policy plus tests and
  documentation.

## In Scope

- `apps/android/app/src/main/java/dev/nako/android/ui/screens/player/`
- `apps/android/app/src/test/java/dev/nako/android/ui/screens/player/`
- `apps/android/scripts/Smoke-Emulator.ps1` only if criteria need updating.
- Workstream docs under this directory.

## Out Of Scope

- V3 irregular/freeform geometry.
- Full Home/Detail redesign.
- Replacing Media3 controls with a custom controller.
- New playback runtime behavior.
- Public Client API changes.

## Architecture Direction

Treat the Media3 controller as the owner of playback controls. Nako custom
chrome should provide context and diagnostics around that controller, not cover
or compete with it. Keep the layout policy explicit and testable so future
player polish does not regress into overlapping controls.

## Closeout Condition

This lane can close when:

- Player chrome layout policy is implemented and tested;
- focused Android player tests pass;
- `profile-with-media` smoke passes and records a fresh player screenshot;
- `git diff --check` passes;
- remaining larger visual ambitions are deferred explicitly.

## Implemented Outcome

Closed on 2026-05-19. The Player custom context panel now reserves a fixed
clearance above the Media3 controller area, preserving standard Media3 controls
while keeping Nako playback context visible. Fresh smoke evidence at
`apps/android/build/smoke/20260519-120345-profile-with-media-emulator-5554/player.png`
shows the panel no longer intersects the progress bar, time labels, or settings
control.

Broader Home and Detail immersion work remains a separate follow-on.
