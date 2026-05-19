# Android Public Client API Coverage TODO

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

- [x] APIC-010 [owner=codex] [deps=none] [scope=docs/workstreams/android-public-client-api-coverage]
  Goal: Freeze the Android Public Client API coverage problem, classify current route coverage, and record the next implementation order.
  Validation: `git diff --check`
  Evidence: `docs/workstreams/android-public-client-api-coverage/API_COVERAGE_MATRIX.md`
  Handoff: Completed in this session. Continue with APIC-020 before adding broader route clients.

## M1 - Selected Artwork Image Slice

- [x] APIC-020 [owner=codex] [deps=APIC-010] [scope=apps/android/app/src/main/java/dev/taru/android/{artwork,browse,ui}]
  Goal: Consume public selected artwork image URLs from item/list/detail DTOs and render authenticated poster/backdrop artwork in Android without leaking bearer tokens.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  Review: verify token redaction, active-server scoping, placeholder fallback, and image loading behavior before accepting.
  Evidence: `apps/android/app/src/test/java/dev/taru/android/artwork/PublicArtworkTest.kt`, `apps/android/app/src/test/java/dev/taru/android/browse/TaruBrowseClientTest.kt`, `apps/android/app/src/main/java/dev/taru/android/ui/artwork/TaruArtworkImage.kt`
  Handoff: Completed in this session. Coil handles authenticated image loading; Android consumes only Public Client API image refs and does not introduce admin artwork routes.

- [x] APIC-030 [owner=codex] [deps=APIC-020] [scope=apps/android/app/src/main/java/dev/taru/android/{browse,ui}]
  Goal: Productize artwork fallback rules across Home, Libraries, Detail, and Player surfaces using the existing Material 3 Expressive direction.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  Review: visual review against `CLIENT_INTERFACE_DESIGN.md`; no fake artwork or unbounded decorative gradients.
  Evidence: `apps/android/app/src/main/java/dev/taru/android/ui/artwork/TaruArtworkSlots.kt`, `apps/android/build/smoke-regression/20260519-131218/report.md`
  Handoff: Completed in this session. Home, Libraries, Detail, and Player now share quiet deterministic artwork fallback behavior; Player remains video-first and disables Media3 embedded artwork.

## M2 - Route Gap Decisions

- [x] APIC-040 [owner=codex] [deps=APIC-020] [scope=docs/workstreams/android-public-client-api-coverage, apps/android/app/src/main/java/dev/taru/android/browse]
  Goal: Decide whether Library Detail and library source inventory should become first-class Android routes in the next product slice.
  Validation: route decision notes plus focused Android tests if client methods are added.
  Review: avoid adding routes that do not produce a user-visible screen.
  Evidence: `API_COVERAGE_MATRIX.md`, `apps/android/app/src/test/java/dev/taru/android/browse/TaruBrowseClientTest.kt`, `apps/android/app/src/main/java/dev/taru/android/ui/browse/LibraryDetailScreen.kt`
  Handoff: Completed in this session. Library Detail is a first-class structural route with safe source inventory; it does not pretend to be a full media poster grid and does not display roots or source locators.

- [ ] APIC-050 [owner=unassigned] [deps=APIC-040] [scope=docs, apps/android/app/src/main/java/dev/taru/android/playback]
  Goal: Decide whether direct `GET /sources/{source_id}/probe` is needed for Source Picker, or whether playback decision probe data is enough.
  Validation: design note or focused Android tests if implemented.
  Review: do not duplicate playback decision semantics in a second source detail flow.
  Evidence: `API_COVERAGE_MATRIX.md`, playback/source picker tests if implemented.
  Handoff: Split deeper track/subtitle/chapter selection into its own lane.

## M3 - Playback State Contract Split

- [ ] APIC-060 [owner=planner] [deps=APIC-010] [scope=docs/workstreams/android-public-client-api-coverage, docs/workstreams]
  Goal: Split a server/client User Playback State workstream if cross-device resume, watched state, or Continue Watching is the next product priority.
  Validation: new workstream or explicit defer note.
  Review: Android must not claim server-authoritative resume without public routes.
  Evidence: `HANDOFF.md`, optional new workstream path.
  Handoff: Keep Android local resume device-local until the contract exists.

## M4 - Closeout

- [ ] APIC-070 [owner=planner] [deps=APIC-020, APIC-040, APIC-050, APIC-060] [scope=docs/workstreams/android-public-client-api-coverage]
  Goal: Close the lane or split remaining route gaps into narrower follow-ons.
  Validation: final Android test gate for implemented code, `git diff --check`, and fresh matrix review against `docs/api/HTTP_API.md`.
  Review: review-workstream has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`.
  Handoff: Summarize remaining product backlog and server contract gaps.
