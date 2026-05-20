# Android Tags Index - TODO

Status: Active
Last updated: 2026-05-20

## Task Ledger

- [x] ATI-010 - Add Tags index client contract.
  - Owner: Codex
  - Dependencies: closed `docs/workstreams/android-relationship-indexes/`.
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/browse/`
    - focused browse client tests.
    - API coverage matrices.
  - Validation:
    - Unit tests cover `GET /tags?limit=&offset=` request building, decoding,
      auth, version checking, and safe diagnostics.
  - Evidence: focused `TaruBrowseClientTest` coverage.
    Completed on 2026-05-20:
    `TagListResponse` and `TaruBrowseClient.listTags` are covered by focused
    tests for request construction, decoding, bearer auth redaction, safe
    diagnostics, and unsupported API version rejection.

- [ ] ATI-020 - Reuse relationship index route state for Tags.
  - Owner: unassigned
  - Dependencies: ATI-010.
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/browse/`
    - focused browse session/navigation/data-source tests.
  - Validation:
    - `BrowseSession` opens, saves, restores, loads, retries, and backs out of
      the Tags Index route.
    - Tag rows open existing Tag related Media Items routes.
  - Evidence: focused UI browse tests.

- [ ] ATI-030 - Productize Tags Index screen entry.
  - Owner: unassigned
  - Dependencies: ATI-020.
  - Scope:
    - relationship index screen family.
    - `HomeScreen` and `TaruBrowseShell`.
    - focused presentation tests where practical.
  - Validation:
    - UI reuses the proven relationship index screen shape.
    - Home exposes Tags as a nested browse route.
    - full Android debug unit gate passes.
  - Evidence: screen implementation and unit gate.

- [ ] ATI-040 - Verify smoke value and close.
  - Owner: unassigned
  - Dependencies: ATI-030.
  - Scope:
    - `apps/android/scripts/Smoke-Emulator.ps1`
    - `docs/workstreams/android-tags-index/`
  - Validation:
    - either focused smoke proves Home -> Tags -> Lighthouse -> Related Media
      Items or the lane records why the existing tag facet smoke is enough.
    - workstream closeout docs are updated.
  - Evidence: smoke report or explicit non-smoke rationale.
