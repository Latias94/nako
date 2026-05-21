# Android Fearless Client Refactor — Milestones

Status: Complete
Last updated: 2026-05-21

## M0 — Scope And Evidence Freeze

Exit criteria:

- Workstream docs exist and agree.
- Refactor register records all architecture review findings.
- Priority order is explicit.
- First executable task is chosen.

Primary evidence:

- `docs/workstreams/android-fearless-client-refactor/DESIGN.md`
- `docs/workstreams/android-fearless-client-refactor/REFACTOR_REGISTER.md`
- `docs/workstreams/android-fearless-client-refactor/TODO.md`

## M1 — P0/P1 Architecture Boundaries

Status:

- AFCR-010 Public Client API execution policy centralization is complete and
  validated on 2026-05-20.
- AFCR-020 token-safe playback launch route state is complete and validated on
  2026-05-20.
- AFCR-030 browse state deepening is complete and validated on 2026-05-21.
- M1 architecture boundaries are complete.

Exit criteria:

- Public Client API execution policy is centralized.
- Route-specific clients no longer duplicate generic protocol behavior.
- Playback launch route state is token-safe by construction.
- `BrowseSession` is no longer the only module that must understand every
  browse/detail/source/playback route.

Primary gates:

- Focused Android JVM tests for connection, browse, playback, user playback,
  player, and browse state modules.
- Token-safety regression tests for route state, diagnostics, and player launch.

## M2 — P2 Production Hardening

Status:

- AFCR-040 Android transport and network-security hardening is complete and
  validated on 2026-05-21.
- AFCR-050 paging for large-library browse surfaces is complete and validated
  on 2026-05-21.
- M2 production hardening is complete for this lane.

Exit criteria:

- Android transport and network-security policy are explicit.
- Cleartext behavior is no longer an accidental global production default.
- Paging state exists for large-library browse surfaces.
- Public Client API pagination semantics remain server-backed.

Primary gates:

- Focused transport and browse paging JVM tests.
- Manifest/network policy review.
- Optional smoke evidence when visible flows change.

## M3 — P3 Product UI Hardening

Status:

- AFCR-060 product UI copy, accessibility semantics, and first localization
  seams are complete and validated on 2026-05-21.
- M3 product UI hardening is complete for this lane.

Exit criteria:

- Stable user-facing strings have localization seams.
- Developer-facing copy is replaced with product-facing media-client language.
- Key custom controls have accessibility semantics.
- Safe diagnostics remain available and sanitized.

Primary gates:

- Android JVM tests.
- Smoke screenshots if broad UI copy changes occur.

## M4 — Architecture Reassessment

Status:

- AFCR-070 architecture reassessment is complete on 2026-05-21.
- The current Kotlin package seams remain the closeout shape for this lane.
- Generated Kotlin SDK, shared Rust/UniFFI client core, Gradle module split,
  artwork descriptors, broader Home/Library Detail paging, downloads/offline,
  external player handoff, and Android TV are split or deferred follow-ons
  rather than hidden work inside this refactor.

Exit criteria:

- Remaining duplication is reassessed after adapter and state refactors.
- Generated SDK, UniFFI, or Gradle module split decisions are recorded.
- Follow-on workstreams are opened only when they have independent closeout
  criteria.

Primary evidence:

- `HANDOFF.md`
- `WORKSTREAM.json`

## M5 — Closeout

Status:

- AFCR-080 final verification is complete on 2026-05-21.
- Fresh full Android JVM tests, debug assemble, local validation smoke, JSON
  validation, and `git diff --check` passed.
- The lane is closed with remaining work split or deferred.

Exit criteria:

- Final gate set is fresh and recorded.
- No known P0 issue from `REFACTOR_REGISTER.md` remains unresolved or unsplit.
- Remaining P1/P2/P3 work is complete, intentionally deferred, or split.
- Workstream status is updated.

Primary gates:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
- `apps/android/scripts/Validate-AndroidLocal.ps1`
- `git diff --check`
