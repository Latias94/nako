# Android Public Client API Coverage Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This workstream was opened after merging `main` into the
`android-client-foundation` worktree. The merge completed without conflicts.

APIC-010, APIC-020, and APIC-030 are complete. The route coverage baseline
lives in `API_COVERAGE_MATRIX.md`, and Android now consumes public selected
artwork image routes for core browse/detail surfaces with a shared productized
fallback presentation.

## Key Findings

- Android is already wired to real Taru server interfaces for setup, browse,
  detail, search, genre/tag/person facet items, playback decision, stream
  targets, and playback session inspection/cancellation.
- Android is not just a static demo. Production app wiring uses
  `JdkTaruHttpTransport`, `TaruConnectionClient`, `TaruBrowseClient`, and
  `TaruPlaybackClient`.
- Android consumes public selected artwork image refs through `GET
  /items/{item_id}/images`, item detail image refs, and authenticated
  `/images/{image_id}` requests rendered by Coil.
- Home, Libraries, Detail, and Player now share deterministic missing-artwork
  treatment. Player stays video-first and disables Media3 embedded artwork
  rather than carrying authenticated artwork requests into playback launch
  state.
- `HEAD /images/{image_id}` is intentionally deferred until explicit preflight
  UX or diagnostics need it.
- Server-authoritative User Playback State is still not public. Android must
  keep current resume behavior device-local.

## Next Task

Run APIC-040 next:

- APIC-040 if the next priority is route coverage: decide whether Library
  Detail and library source inventory should become first-class Android routes.

Recommended validation:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
git diff --check
```

Latest visual smoke evidence:

```powershell
pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States profile-with-media
```

Result: PASS on 2026-05-19. Report:
`apps/android/build/smoke-regression/20260519-131218/report.md`.

## Constraints To Preserve

- Do not consume Admin API routes from Android.
- Do not expose bearer tokens in UI, logs, screenshots, safe request previews,
  or test failure messages.
- Do not parse or store raw Source Locators, local paths, managed artwork
  storage URIs, or provider secret material.
- Keep local resume clearly separate from server-authoritative User Playback
  State.
- Do not revive the completed Android foundation workstream for this work; this
  lane is the follow-up route coverage lane.

## Open Decisions

- Coil 3.3.0 is now the selected image-loading dependency for authenticated
  public artwork routes.
- Whether Library Detail should become a real product route before broader
  people/tag/genre index screens.
- Whether Source Picker needs a direct source probe refresh route, or playback
  decision probe data is enough.
