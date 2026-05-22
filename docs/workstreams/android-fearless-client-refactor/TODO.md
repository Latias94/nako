# Android Fearless Client Refactor — TODO

Status: Complete
Last updated: 2026-05-21

## Task Ledger

### M0 — Scope And Evidence Freeze

- [x] AFCR-000 [owner=codex] [deps=none] [priority=P0] [scope=docs/workstreams/android-fearless-client-refactor]
  Goal: Open the durable workstream and freeze the review findings, decisions,
  priority order, and validation gates.
  Validation: `DESIGN.md`, `REFACTOR_REGISTER.md`, `TODO.md`,
  `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and
  `HANDOFF.md` exist and agree.
  Evidence: this workstream directory.
  Handoff: Start implementation with AFCR-010 unless the user explicitly asks
  to tackle token-safe playback launch first.

### M1 — P0/P1 Architecture Boundaries

- [x] AFCR-010 [owner=codex] [deps=AFCR-000] [priority=P1] [scope=apps/android/app/src/main/java/dev/nako/android/connection,apps/android/app/src/main/java/dev/nako/android/browse,apps/android/app/src/main/java/dev/nako/android/playback,apps/android/app/src/main/java/dev/nako/android/userplayback,apps/android/app/src/test/java/dev/nako/android]
  Goal: Introduce a deep Public Client API execution adapter and migrate
  connection, browse, playback, and User Playback State clients away from
  duplicated protocol policy.
  Scope:
  - Create a route-independent adapter for authenticated and unauthenticated
    Public Client API calls.
  - Centralize API-version checks, public error-envelope parsing, JSON decode
    failures, request previews, bearer redaction, transport failure mapping,
    and path/query helper policy.
  - Keep route clients small: route path/query/body construction and typed DTO
    mapping only.
  - Delete superseded duplicated helpers from route clients.
  Validation:
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.connection.* --no-daemon`
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.browse.* --no-daemon`
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.playback.* --no-daemon`
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.userplayback.* --no-daemon`
  Review: verify no route client owns generic protocol policy after migration.
  Evidence: `EVIDENCE_AND_GATES.md` records focused and full Android JVM
  test output from 2026-05-20.

- [x] AFCR-020 [owner=codex] [deps=AFCR-010] [priority=P0] [scope=apps/android/app/src/main/java/dev/nako/android/playback,apps/android/app/src/main/java/dev/nako/android/player,apps/android/app/src/main/java/dev/nako/android/ui/screens/player,apps/android/app/src/main/java/dev/nako/android/ui/browse,apps/android/app/src/test/java/dev/nako/android]
  Goal: Make playback launch route state token-safe by construction.
  Scope:
  - Replace route-level raw `NakoHttpRequest` launch state with a token-safe
    playback launch descriptor.
  - Inject Authorization headers only inside the player runtime adapter or a
    non-saveable final request builder.
  - Ensure Player route save/restore, diagnostics copy, error presentation,
    and route `toString` cannot expose bearer tokens.
  - Keep Media3 playback behavior and playback session cancellation semantics.
  Validation:
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.player.* --no-daemon`
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.player.* --no-daemon`
  - token-safety grep or focused tests proving no route/saveable state contains `Bearer secret-token`.
  Review: verify no raw bearer token can enter saveable route state or visible
  UI diagnostics.
  Evidence: `EVIDENCE_AND_GATES.md` records focused and full Android JVM
  test output from 2026-05-20.

- [x] AFCR-030 [owner=codex] [deps=AFCR-010] [priority=P1] [scope=apps/android/app/src/main/java/dev/nako/android/ui/browse,apps/android/app/src/main/java/dev/nako/android/ui/screens,apps/android/app/src/test/java/dev/nako/android/ui/browse]
  Goal: Split `BrowseSession` into deep state modules without changing product
  behavior.
  Scope:
  - Keep top-level `BrowseSession` as a composition root.
  - Extract deep modules for catalog/search, relationship browsing, Media Item
    Detail/source selection, and playback start policy.
  - Preserve stale-response protection and route-aware loading semantics.
  - Delete obsolete broad-state helpers after the new modules own behavior.
  Validation:
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.ui.browse.* --no-daemon`
  - focused tests for stale route responses, retry behavior, source selection,
    and playback route opening.
  Review: deletion test on each extracted module; reject pass-through modules.
  Evidence: `EVIDENCE_AND_GATES.md` records browse-focused and full Android
  JVM test output from 2026-05-21.

### M2 — P2 Production Hardening

- [x] AFCR-040 [owner=codex] [deps=AFCR-010] [priority=P2] [scope=apps/android/app/src/main/AndroidManifest.xml,apps/android/app/src/main/java/dev/nako/android/connection,apps/android/app/src/test/java/dev/nako/android/connection]
  Goal: Harden Android transport and network security policy for production
  self-hosted use.
  Scope:
  - Keep `NakoHttpTransport` as the seam.
  - Either harden the existing transport with final cleanup/cancellation policy
    or replace production transport with a cleaner adapter such as OkHttp.
  - Split debug/local cleartext behavior from release policy.
  - Make insecure HTTP connection state user-visible and token-safe.
  Validation:
  - focused transport/client tests;
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.connection.* --no-daemon`;
  - manifest/network policy review.
  Review: confirm self-hosted local development still works while release
  policy is no longer globally permissive by accident.
  Evidence: `EVIDENCE_AND_GATES.md` records connection-focused and full
  Android JVM test output from 2026-05-21.

- [x] AFCR-050 [owner=codex] [deps=AFCR-030] [priority=P2] [scope=apps/android/app/src/main/java/dev/nako/android/ui/browse,apps/android/app/src/main/java/dev/nako/android/browse,apps/android/app/src/test/java/dev/nako/android]
  Goal: Add reusable paging state for large Media Libraries.
  Scope:
  - Introduce page state and load-more actions for browse surfaces.
  - Apply to Search and one relationship/facet route first as vertical proof.
  - Extend Home/Library Detail only when the shared policy is proven.
  - Preserve API-supported `limit`, `offset`, and `returned` semantics.
  Validation:
  - focused paging state tests;
  - browse client query construction tests;
  - optional smoke state if UI behavior materially changes.
  Review: reject local filtering or invented totals not backed by Public Client
  API.
  Evidence: `EVIDENCE_AND_GATES.md` records focused and full Android JVM test
  output from 2026-05-21.

### M3 — P3 Product UI Hardening

- [x] AFCR-060 [owner=codex] [deps=AFCR-020,AFCR-030] [priority=P3] [scope=apps/android/app/src/main/java/dev/nako/android/ui,apps/android/app/src/main/res/values,apps/android/app/src/test/java/dev/nako/android/ui]
  Goal: Productize copy, accessibility semantics, and localization seams.
  Scope:
  - Move stable user-facing strings to Android resources where practical.
  - Rewrite developer-facing copy in Settings, Source Picker, API-gap states,
    Detail, Player, and errors into user-facing media-client language.
  - Add semantics for custom clickable rows, chips, artwork, source selection,
    player session status, and diagnostics copy actions.
  - Preserve sanitized advanced diagnostics.
  Validation:
  - focused presentation tests where existing;
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`;
  - smoke screenshots if visible copy changes are broad.
  Review: run a UI copy pass against the Product UI principles in
  `android-material-expressive-ui`.
  Evidence: `EVIDENCE_AND_GATES.md` records focused and full Android JVM
  test output from 2026-05-21.

### M4 — Architecture Reassessment And Follow-On Split

- [x] AFCR-070 [owner=codex] [deps=AFCR-010,AFCR-020,AFCR-030] [priority=P2] [scope=docs/workstreams/android-fearless-client-refactor,apps/android]
  Goal: Decide whether the next clean boundary is a generated Kotlin SDK,
  shared Rust/UniFFI client core, Gradle module split, or continued package
  seams.
  Scope:
  - Reassess remaining duplication after the Public Client API adapter lands.
  - Decide whether DTO/route drift still justifies generated SDK or UniFFI.
  - Decide whether package seams are deep enough for a Gradle module split.
  - Record follow-on workstreams for downloads/offline, external player
    handoff, Android TV, or SDK/FFI if they become the next clean lane.
  Validation:
  - architectural review note in `HANDOFF.md`;
  - update `WORKSTREAM.json` continue policy.
  Review: do not start module splitting or UniFFI packaging without a separate
  target-state document.
  Evidence: `HANDOFF.md` records the 2026-05-21 architecture reassessment:
  keep package seams for closeout, split generated Kotlin SDK and shared
  Rust/UniFFI client core into separate target-state workstreams, defer Gradle
  module splitting until there is a second adapter or build/dependency pressure,
  and track artwork descriptors plus broader paging as follow-ons.

### M5 — Final Verification And Closeout

- [x] AFCR-080 [owner=codex] [deps=AFCR-040,AFCR-050,AFCR-060,AFCR-070] [priority=P0] [scope=docs/workstreams/android-fearless-client-refactor]
  Goal: Close or split the lane with fresh evidence.
  Validation:
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  - `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
  - `apps/android/scripts/Validate-AndroidLocal.ps1`
  - `git diff --check`
  Review: no blocking findings in code quality and workstream compliance.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Remaining work must be explicitly complete, deferred, or split.
  Result: DONE 2026-05-21. Full Android JVM tests, debug assemble, local
  validation including smoke, JSON validation, and `git diff --check` passed.
  Smoke harness expected copy was updated to the AFCR-060 product language
  rather than reverting UI text.

## Parallelization Notes

Parallel workers are safe only after AFCR-010 lands because it changes the
shared client execution surface. After that:

- AFCR-020 and AFCR-030 can run in parallel if their write sets stay disjoint.
- AFCR-040 can run in parallel with AFCR-030 if transport interfaces are stable.
- AFCR-060 should wait for token-safe launch and browse state boundaries.
