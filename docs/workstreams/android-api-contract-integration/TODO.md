# Android API Contract Integration - TODO

Status: Active
Last updated: 2026-05-20

## Task Ledger

- [x] APICI-010 - Freeze Android Public Client API integration matrix.
  - Owner: Codex
  - Scope:
    - `docs/workstreams/android-api-contract-integration/`
  - Validation:
    - `WORKSTREAM.json` parses.
    - Matrix answers which server public routes Android has already connected.
    - First implementation slice is explicit.
  - Evidence: Docs created on 2026-05-20.

- [x] APICI-020 - Add Person Detail client contract.
  - Owner: Codex
  - Dependencies: APICI-010.
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/browse/BrowseModels.kt`
    - `apps/android/app/src/main/java/dev/taru/android/browse/TaruBrowseClient.kt`
    - focused browse client tests.
  - Validation:
    - Unit tests prove `GET /people/{person_id}` request building, decoding,
      version checking, auth, and safe error diagnostics.
  - Evidence: Focused `TaruBrowseClientTest` passed on 2026-05-20.

- [x] APICI-030 - Productize Person Detail route state.
  - Owner: Codex
  - Dependencies: APICI-020.
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/browse/`
    - focused session/navigation tests.
  - Validation:
    - `BrowseSession` opens Person Detail from a stable person ID.
    - stale route responses are ignored.
    - related Media Items use the existing person-items route.
  - Evidence: Focused UI browse tests and full debug unit gate passed on
    2026-05-20.

- [x] APICI-040 - Build Person Detail screen.
  - Owner: Codex
  - Dependencies: APICI-030.
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/screens/`
    - `apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShell.kt`
    - `apps/android/app/src/main/java/dev/taru/android/ui/screens/detail/MediaItemDetailRoute.kt`
  - Validation:
    - Cast & Crew rows open Person Detail when a stable `person_id` exists.
    - Compose/unit coverage where practical.
    - Existing Material Expressive design tokens are reused.
  - Evidence: Focused detail presentation test and full debug unit gate passed
    on 2026-05-20.

- [x] APICI-050 - Prove server-backed Person Detail smoke.
  - Owner: Codex
  - Dependencies: APICI-040.
  - Scope:
    - `apps/android/scripts/Smoke-Emulator.ps1`
    - fixture/smoke docs if needed.
  - Validation:
    - focused `profile-with-media` smoke opens Cast & Crew person detail and
      returns to related Media Items.
  - Evidence: Focused `profile-with-media` smoke passed on 2026-05-20 with
    `person-detail` surface evidence.

- [ ] APICI-060 - Decide People/Tags/Genres indexes.
  - Owner: Codex
  - Dependencies: APICI-050.
  - Scope:
    - workstream docs and, if accepted, follow-on task split.
  - Validation:
    - either productize index pages with explicit tasks or mark them deferred
      with rationale.
