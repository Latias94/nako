# Android Public Client API Coverage Handoff

Status: Closed
Last updated: 2026-05-19

## Current State

This workstream was opened after merging `main` into the
`android-client-foundation` worktree. The merge completed without conflicts.

APIC-010, APIC-020, APIC-030, APIC-040, APIC-050, APIC-060, and APIC-070 are complete.
The route coverage baseline lives in `API_COVERAGE_MATRIX.md`, Android consumes
public selected artwork image routes for core browse/detail surfaces, Library
Detail is now a first-class structural route, and Source Picker consumes direct
source probe facts separately from playback decision.

## Key Findings

- Android is already wired to real Nako server interfaces for setup, browse,
  detail, search, genre/tag/person facet items, playback decision, stream
  targets, and playback session inspection/cancellation.
- Android is not just a static demo. Production app wiring uses
  `JdkNakoHttpTransport`, `NakoConnectionClient`, `NakoBrowseClient`, and
  `NakoPlaybackClient`.
- Android consumes public selected artwork image refs through `GET
  /items/{item_id}/images`, item detail image refs, and authenticated
  `/images/{image_id}` requests rendered by Coil.
- Home, Libraries, Detail, and Player now share deterministic missing-artwork
  treatment. Player stays video-first and disables Media3 embedded artwork
  rather than carrying authenticated artwork requests into playback launch
  state.
- Android now consumes `GET /libraries/{library_id}` and
  `GET /libraries/{library_id}/sources` through `NakoBrowseClient` and a
  Library Detail screen. The screen shows safe source inventory, not raw roots,
  source locators, or a fake media poster grid.
- Android now consumes `GET /sources/{source_id}/probe` through
  `NakoPlaybackClient.getSourceProbe`. Source Picker uses it for compact source
  facts and keeps `GET /sources/{source_id}/playback/decision` dedicated to
  playback launch decisions.
- `HEAD /images/{image_id}` is intentionally deferred until explicit preflight
  UX or diagnostics need it.
- Server-authoritative User Playback State is still not public. Android must
  keep current resume behavior device-local until
  `docs/workstreams/user-playback-state-contract/` ships.

## Next Task

Run UPS-010 next:

- `docs/workstreams/user-playback-state-contract/` owns
  server-authoritative **User Playback State**, cross-device resume, watched
  state, and Continue Watching.

Latest closeout validation:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
git diff --check
```

Result: PASS on 2026-05-19.

Latest visual smoke evidence:

```powershell
pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States profile-with-media
```

Result: PASS on 2026-05-19. Report:
`apps/android/build/smoke-regression/20260519-141311/report.md`.

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

## Follow-Ons

- Coil 3.3.0 is now the selected image-loading dependency for authenticated
  public artwork routes.
- Direct source probe is now covered for compact Source Picker source facts.
- Server-authoritative **User Playback State** is split to
  `docs/workstreams/user-playback-state-contract/`.
- People/tag/genre index and Person Detail pages remain product backlog.
- `HEAD /images/{image_id}` remains deferred until preflight or diagnostics UX
  needs it.
