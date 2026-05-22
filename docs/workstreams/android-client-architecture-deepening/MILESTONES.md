# Android Client Architecture Deepening — Milestones

Status: Draft
Last updated: 2026-05-22

## M0 — Scope And Evidence Freeze

Exit criteria:

- New workstream docs exist and agree on scope, non-goals, authority, task
  order, and gates.
- The lane is explicitly a follow-on to closed Android foundation/refactor
  lanes, not a reopening of them.
- First executable task is chosen.

Primary evidence:

- `docs/workstreams/android-client-architecture-deepening/DESIGN.md`
- `docs/workstreams/android-client-architecture-deepening/TODO.md`
- `docs/workstreams/android-client-architecture-deepening/WORKSTREAM.json`

## M1 — Client Runtime Deep Module

Exit criteria:

- A deeper Android Public Client runtime seam removes duplicated generic
  execution orchestration.
- Route-family clients keep product-specific semantics and model mapping.
- Token redaction/safe request behavior remains proven by tests.

Primary gates:

- Focused connection/browse/playback/User Playback State client tests.
- Full `:app:testDebugUnitTest` when targeted tests pass.

## M2 — Explicit Browse Effects

Status: DONE on 2026-05-22.

Exit criteria:

- Browse route loading and saveable-state side effects are explicit and
  testable.
- `BrowseSession` remains deterministic state machinery.
- Stale response invalidation, transient player routes, and route restoration
  remain covered.

Primary gates:

- Focused browse session/host tests.
- Browse UI/session package tests.

## M3 — Android Player Runtime

Status: DONE on 2026-05-22.

Exit criteria:

- Player orchestration has an Android-owned runtime seam separate from broad
  Composable route logic.
- Media3 lifecycle, event mapping, retry/back/dispose, resume seek, and exit
  effects have clear ownership.
- Rust still does not own player/platform behavior.

Primary gates:

- Focused player route/runtime tests.
- Playback exit effect tests.
- Playback client tests.

## M4 — UI Design-System Locality

Status: DONE on 2026-05-22.

Exit criteria:

- Generic design-system components and Taru media-specific components have
  distinct ownership.
- Redundant wrappers and obsolete transition code are deleted where safe.
- Large screen files lose independently testable display-model/copy logic.
- Accessibility and token-safe diagnostics remain covered.

Primary gates:

- Focused UI presentation tests.
- Updated smoke criteria when visible copy changes.

## M5 — Home Section Read Model

Status: DONE on 2026-05-22.

Exit criteria:

- Home can represent partial/degraded section states instead of one coarse
  all-or-nothing load.
- Managed Artwork enrichment can progress or fail independently where safe.
- No unsupported server semantics, fake totals, or local filtering are invented.

Primary gates:

- `ClientBrowseDataSource`/Home read-model tests.
- Browse UI/session tests.
- Smoke regression if visible flow changes.

## M6 — Decision Sweep

Status: DONE on 2026-05-22.

Exit criteria:

- Persistence, lifecycle-aware collection, Gradle/UniFFI validation, and stale
  transition-code questions have explicit implement/defer/split decisions.
- Small safe cleanup is implemented; larger scopes are routed to follow-ons.
- Evidence gates are updated.

Primary gates:

- `git diff --check`
- relevant focused Android tests
- JSON validation for `WORKSTREAM.json`

## M7 — Closeout

Status: DONE on 2026-05-22.

Exit criteria:

- All accepted tasks are complete or explicitly split.
- Fresh focused and broad validation evidence is recorded.
- Review finds no blocking architecture, token-safety, or workstream-compliance
  issues.
- `WORKSTREAM.json` and `HANDOFF.md` reflect final state.

Primary gates:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon --no-parallel`
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug -PtaruRustAndroidAbis=x86_64 --no-daemon --no-parallel`
- `apps/android/scripts/Validate-AndroidLocal.ps1` when local smoke environment is available
- `git diff --check`
