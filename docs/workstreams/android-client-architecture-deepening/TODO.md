# Android Client Architecture Deepening — TODO

Status: Draft
Last updated: 2026-05-22

## Task Ledger

### M0 — Scope And Evidence Freeze

- [x] ACAD-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-client-architecture-deepening]
  Goal: Open the fearless Android architecture-deepening lane, freeze target
  state, non-goals, authority, and validation gates.
  Validation: `DESIGN.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`,
  `WORKSTREAM.json`, and `HANDOFF.md` exist and agree.
  Review: Confirm this lane is a new follow-on and does not reopen closed AFCR,
  Rust core route, browse host, or player host lanes.
  Evidence: `docs/workstreams/android-client-architecture-deepening/DESIGN.md`
  Handoff: DONE. Start with ACAD-020.

### M1 — Client Runtime And Request Execution Seam

- [x] ACAD-020 [owner=codex] [deps=ACAD-010] [scope=apps/android/app/src/main/java/dev/nako/android/connection,apps/android/app/src/main/java/dev/nako/android/browse,apps/android/app/src/main/java/dev/nako/android/playback,apps/android/app/src/main/java/dev/nako/android/userplayback,apps/android/app/src/test/java/dev/nako/android]
  Goal: Introduce a deeper Android Public Client runtime seam above
  `PublicClientApiExecutor` that removes duplicated token/core/executor/decode
  orchestration while preserving route-family product semantics.
  Validation: targeted connection, browse, playback, and User Playback State
  client tests; full `:app:testDebugUnitTest` if targeted gates pass.
  Review: The new runtime must be a deep module, not a pass-through facade.
  Route-family clients must still own product categories and user-facing model
  mapping. Token values must remain absent from saved state, diagnostics, and
  `toString` output.
  Evidence: runtime source/test paths and `EVIDENCE_AND_GATES.md` evidence log.
  Handoff: DONE. `PublicClientRuntime` now owns authenticated request
  validation, request execution, JSON decode, safe request propagation, and
  core-response execution. Browse, Playback, User Playback State, and
  Connection clients now use it while keeping route-family failure categories
  and product model mapping local. Obsolete base/path/auth helpers were removed
  from `PublicClientApiExecutor`.

### M2 — Browse Effects And Route Loading Ownership

- [x] ACAD-030 [owner=codex] [deps=ACAD-020] [scope=apps/android/app/src/main/java/dev/nako/android/ui/browse,apps/android/app/src/test/java/dev/nako/android/ui/browse]
  Goal: Make browse route side effects explicit and testable. Replace implicit
  state-collection-driven route loading/save orchestration with a clear effect,
  load-intent, or coordinator interface while keeping `BrowseSession` as the
  deterministic state machine.
  Validation: focused browse session/host tests; route stale-response tests;
  navigation save/restore tests; full browse UI/session test package.
  Review: Route loading, saveable-state publishing, stale-response protection,
  and player-route transient behavior must have obvious ownership. Avoid
  introducing a framework or Android lifecycle dependency unless it pays for
  itself.
  Evidence: browse state/effect/coordinator tests and evidence log.
  Handoff: DONE. `BrowseSessionEffectCoordinator` now owns saveable-state
  publication and route-displayed load intents. `BrowseShellHost` only wires the
  coordinator to `BrowseSession`, and `BrowseRouteLoadingSession` treats player
  routes as transient non-load intents so previous detail state remains intact.

### M3 — Android Player Runtime Seam

- [x] ACAD-040 [owner=codex] [deps=ACAD-020] [scope=apps/android/app/src/main/java/dev/nako/android/ui/screens/player,apps/android/app/src/main/java/dev/nako/android/playback,apps/android/app/src/test/java/dev/nako/android/ui/screens/player,apps/android/app/src/test/java/dev/nako/android/playback]
  Goal: Promote route-scoped player orchestration into an Android-owned
  PlayerRuntime/PlaybackSessionRuntime seam that owns Media3 lifecycle
  orchestration, player event mapping, exit-effect dispatch, resume seek policy,
  and future MediaSession/PiP/Cast extension points without moving player
  ownership into Rust.
  Validation: focused player route/runtime tests; playback client tests;
  player exit effect tests; full Android JVM tests if focused gates pass.
  Review: Configuration/disposal/background semantics must be deliberate and
  documented. Exit effects must remain idempotent and token-safe. The runtime
  must not store bearer tokens in saveable route state.
  Evidence: player runtime source/test paths and evidence log.
  Handoff: DONE. `PlaybackSessionRuntime` and
  `PlaybackSessionRuntimeFactory` now form the Android-owned runtime seam.
  `PlayerRouteHost` implements the runtime interface, while
  `PlaybackPlayerRoute` only creates/collects/binds the runtime for rendering.
  MediaSession/PiP/Cast/track-selection remain follow-ons behind the runtime
  seam rather than implemented in this lane.

### M4 — UI Design-System And Screen Modularization

- [x] ACAD-050 [owner=codex] [deps=ACAD-030] [scope=apps/android/app/src/main/java/dev/nako/android/ui/components,apps/android/app/src/main/java/dev/nako/android/ui/browse,apps/android/app/src/main/java/dev/nako/android/ui/screens,apps/android/app/src/test/java/dev/nako/android/ui]
  Goal: Separate generic design-system components from Nako media-specific
  presentation components and screen route composition. Delete redundant
  pass-through wrappers and move independently testable display-model/copy
  builders out of oversized composable files.
  Validation: focused UI presentation tests; source picker/detail/settings/player
  tests; smoke criteria updates if visible copy or content descriptions change.
  Review: Preserve Material Expressive baseline, token-safe diagnostics,
  accessibility labels, and existing route behavior. Do not churn purely for
  file-size aesthetics; every extraction must improve locality or deletion.
  Evidence: component source/test paths and evidence log.
  Handoff: DONE. Cross-screen callers now import generic design-system
  primitives directly from `ui.components`; obsolete `BrowseComponents`
  pass-through wrappers for screen columns, section headers, surface cards,
  status chips/pills, icon badges, artwork backdrops, and pressable scale were
  removed. Source Picker and Detail display/copy builders moved into
  test-covered presentation files. Remaining large route files are documented in
  `HANDOFF.md` with reasons or follow-on routing.

### M5 — Home Section Read Model And Progressive Artwork

- [x] ACAD-060 [owner=codex] [deps=ACAD-020,ACAD-050] [scope=apps/android/app/src/main/java/dev/nako/android/ui/browse,apps/android/app/src/test/java/dev/nako/android/ui/browse,apps/android/app/src/main/java/dev/nako/android/artwork]
  Goal: Replace monolithic Home loading with a section-oriented read model that
  can render partial Media Library, Media Item, Continue Watching, and Managed
  Artwork states independently, including degraded section errors where useful.
  Validation: focused `ClientBrowseDataSource`/Home tests for partial success,
  artwork failure, continue-watching failure, and first-paint state; browse UI
  tests; smoke regression if visible loading/copy changes.
  Review: Do not invent local filtering, fake totals, or unsupported server
  semantics. Artwork enrichment must stay token-safe and should not block the
  whole Home surface unless required for correctness.
  Evidence: Home read-model tests and evidence log.
  Handoff: DONE. `BrowseUiState.Content` now carries a `HomeReadModel` with
  independent Media Libraries, Visible Titles, Continue Watching, and Managed
  Artwork section states. `ClientBrowseDataSource.loadHome()` returns partial
  content when one primary section succeeds, records Continue Watching and
  artwork failures as degraded sections, and only fails the whole Home surface
  when both primary sections fail or the access token is missing. Home and
  Libraries screens now render section-level unavailable states instead of
  silently converting failures into empty lists.

### M6 — Persistence, Lifecycle, And Build Hygiene Decision Sweep

- [x] ACAD-070 [owner=codex] [deps=ACAD-040,ACAD-060] [scope=apps/android,docs/workstreams/android-client-architecture-deepening]
  Goal: Perform a deletion/decision sweep for local persistence, lifecycle-aware
  collection, Gradle/UniFFI validation, and stale transition code. Implement
  small safe cleanups; split larger DataStore/Room, lifecycle-runtime-compose,
  Gradle module split, or CI/ABI policy into follow-ons if they exceed this
  lane.
  Validation: `git diff --check`; relevant focused Android JVM tests; JSON
  validation for `WORKSTREAM.json`; run `apps/android/scripts/Validate-AndroidLocal.ps1`
  when code changes are complete and the local environment supports it.
  Review: Do not hide product scopes like downloads/offline or Android TV in
  this sweep. Each deferred item must have a named follow-on or explicit reason.
  Evidence: decision notes in `EVIDENCE_AND_GATES.md` and updated handoff.
  Handoff: DONE. Compose `StateFlow` collection now uses
  `collectAsStateWithLifecycle()` via `androidx.lifecycle:lifecycle-runtime-compose`.
  Local validation passed with Android JVM tests and debug assemble through
  `Validate-AndroidLocal.ps1 -SkipSmoke`. DataStore/Room migration, Gradle
  module split, CI/ABI policy hardening, and smoke/device-farm expansion remain
  explicit follow-ons rather than hidden in this sweep.

### M7 — Closeout

- [x] ACAD-090 [owner=planner] [deps=ACAD-020,ACAD-030,ACAD-040,ACAD-050,ACAD-060,ACAD-070] [scope=docs/workstreams/android-client-architecture-deepening]
  Goal: Close the lane with fresh verification evidence, review findings,
  residual risks, and follow-on routing.
  Validation: JSON validation for `WORKSTREAM.json`; `git diff --check`;
  focused gates from completed tasks; full Android JVM tests; Android local
  validation/smoke when practical.
  Review: Run workstream/code review before marking complete. No blocking
  workstream compliance or token-safety findings may remain.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, optional `CLOSEOUT.md`.
  Handoff: DONE. Final review found no blocking workstream compliance or code
  quality findings. Full Android JVM tests, x86_64 debug assemble, hygiene
  checks, and JSON validation passed on 2026-05-22. Remaining product scopes
  are routed as follow-ons in `CLOSEOUT.md`.
