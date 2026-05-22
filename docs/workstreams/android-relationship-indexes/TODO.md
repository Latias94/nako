# Android Relationship Indexes - TODO

Status: Closed
Last updated: 2026-05-20

## Task Ledger

- [x] ARI-010 - Freeze index product decision and first slice.
  - Owner: planner
  - Dependencies: APICI-060.
  - Scope:
    - `docs/workstreams/android-relationship-indexes/`
    - `docs/workstreams/android-api-contract-integration/API_INTEGRATION_MATRIX.md`
  - Validation:
    - `WORKSTREAM.json` parses.
    - People, Tags, and Genres each have an explicit accept/defer decision.
    - First implementation slice is chosen.
  - Evidence: `DESIGN.md`, `TODO.md`, and updated API integration matrix.
    Completed on 2026-05-20: Genres accepted as first slice, Tags accepted as
    second slice, and top-level People index deferred.

- [x] ARI-020 - Add Genre index client contract.
  - Owner: Codex
  - Dependencies: ARI-010.
  - Scope:
    - `apps/android/app/src/main/java/dev/nako/android/browse/`
    - focused browse client tests.
  - Validation:
    - Unit tests cover `GET /genres?limit=&offset=` request building,
      decoding, auth, version checking, and safe diagnostics.
  - Evidence: focused browse client tests.
    Completed on 2026-05-20:
    `NakoBrowseClient.listGenres` and `GenreListResponse` are covered by
    `dev.nako.android.browse.NakoBrowseClientTest`.

- [x] ARI-030 - Productize Genre index route state.
  - Owner: Codex
  - Dependencies: ARI-020.
  - Scope:
    - `apps/android/app/src/main/java/dev/nako/android/ui/browse/`
    - focused session/navigation tests.
  - Validation:
    - `BrowseSession` opens, saves, restores, loads, retries, and backs out of
      the Genre Index route.
    - Genre rows open existing Genre related Media Items routes.
  - Evidence: focused UI browse tests.
    Completed on 2026-05-20:
    `RelationshipIndexFamily.Genres`, `NakoRoute.RelationshipIndex`, and
    `RelationshipIndexUiState` are covered by focused UI browse tests.

- [x] ARI-040 - Build Genre index screen.
  - Owner: Codex
  - Dependencies: ARI-030.
  - Scope:
    - `apps/android/app/src/main/java/dev/nako/android/ui/screens/`
    - `apps/android/app/src/main/java/dev/nako/android/ui/browse/NakoBrowseShell.kt`
  - Validation:
    - Material Expressive screen uses existing tokens/components.
    - row actions preserve stable server IDs.
    - full Android debug unit gate passes.
  - Evidence: screen implementation and focused presentation tests where
    practical.
    Completed on 2026-05-20:
    `RelationshipIndexRouteContent` replaced the temporary placeholder,
    Home exposes a Genres anchor, and the full Android debug unit gate passed.

- [x] ARI-050 - Prove Genre index smoke or split closeout.
  - Owner: Codex
  - Dependencies: ARI-040.
  - Scope:
    - `apps/android/scripts/Smoke-Emulator.ps1`
    - `docs/workstreams/android-relationship-indexes/`
  - Validation:
    - either focused smoke proves the Genre Index path or the lane
      records why smoke is not yet valuable.
    - Tags are scheduled as the next reuse slice or deferred with rationale.
    - People index remains deferred unless a richer People IA is accepted.
  - Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
    Completed on 2026-05-20:
    `profile-with-media` smoke proves Home -> Genres -> Mystery -> Related
    Media Items with server-backed fixture data. Tags Index is split to
    `docs/workstreams/android-tags-index/`; top-level People Index remains
    deferred.
