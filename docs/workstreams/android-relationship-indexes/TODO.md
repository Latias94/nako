# Android Relationship Indexes - TODO

Status: Active
Last updated: 2026-05-20

## Task Ledger

- [ ] ARI-010 - Freeze index product decision and first slice.
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

- [ ] ARI-020 - Add accepted index client contracts.
  - Owner: unassigned
  - Dependencies: ARI-010.
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/browse/`
    - focused browse client tests.
  - Validation:
    - Unit tests cover request building, decoding, auth, version checking, and
      safe diagnostics for each accepted index list route.
  - Evidence: focused browse client tests.

- [ ] ARI-030 - Productize first relationship index route.
  - Owner: unassigned
  - Dependencies: ARI-020.
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/browse/`
    - focused session/navigation tests.
  - Validation:
    - `BrowseSession` opens, saves, restores, loads, retries, and backs out of
      the first accepted index route.
    - index rows open existing related Media Items routes or Person Detail.
  - Evidence: focused UI browse tests.

- [ ] ARI-040 - Build first relationship index screen.
  - Owner: unassigned
  - Dependencies: ARI-030.
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/screens/`
    - `apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShell.kt`
  - Validation:
    - Material Expressive screen uses existing tokens/components.
    - row actions preserve stable server IDs.
    - full Android debug unit gate passes.
  - Evidence: screen implementation and focused presentation tests where
    practical.

- [ ] ARI-050 - Prove first relationship index smoke or split closeout.
  - Owner: planner
  - Dependencies: ARI-040.
  - Scope:
    - `apps/android/scripts/Smoke-Emulator.ps1`
    - `docs/workstreams/android-relationship-indexes/`
  - Validation:
    - either focused smoke proves the accepted first index path or the lane
      records why smoke is not yet valuable.
    - remaining index families are completed, split, or deferred.
  - Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
