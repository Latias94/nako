# Android Detail Facet Smoke Evidence - Evidence And Gates

Status: Closed
Last updated: 2026-05-19

## Required Gates

- Focused smoke:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media`
- Regression smoke:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media -SkipBuild`
- Diff hygiene:
  `git diff --check`

## Evidence Ledger

### ADF-010 - Boundary Freeze

- Evidence: `docs/workstreams/android-detail-facet-smoke-evidence/DESIGN.md`
- Result: Complete.
- Notes: Lane is scoped to API-backed Genre, Tag, and Person detail facets.

### ADF-020 - Detail Facet Navigation Smoke Slice

- Evidence:
  - Focused smoke report:
    `apps/android/build/smoke/20260519-104315-profile-with-media-emulator-5554/report.md`
  - Regression smoke report:
    `apps/android/build/smoke-regression/20260519-104729/report.md`
  - Criteria files:
    `apps/android/build/smoke/20260519-104315-profile-with-media-emulator-5554/detail-metadata.criteria.txt`
    `apps/android/build/smoke/20260519-104315-profile-with-media-emulator-5554/facet-genre.criteria.txt`
    `apps/android/build/smoke/20260519-104315-profile-with-media-emulator-5554/facet-tag.criteria.txt`
    `apps/android/build/smoke/20260519-104315-profile-with-media-emulator-5554/detail-cast-crew.criteria.txt`
    `apps/android/build/smoke/20260519-104315-profile-with-media-emulator-5554/facet-person.criteria.txt`
- Result: Complete.
- Notes: Smoke proved detail metadata chips for `Mystery` and `Lighthouse`,
  the `Actor / as Keeper` Person row, and API-backed facet result routes that
  returned `Night Harbor`. Regression smoke passed for `empty-setup`,
  `profile-missing-token`, and `profile-with-media`.

### ADF-030 - Closeout

- Evidence: this document, `TODO.md`, `DESIGN.md`, `HANDOFF.md`, and
  `WORKSTREAM.json`.
- Result: Complete.
- Notes: Collection, Studio, Series/Hierarchy, Year, Item Kind, and richer
  role-specific navigation remain follow-ons.
