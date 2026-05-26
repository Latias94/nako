# Admin Web V2 Library Management And Localization - Evidence And Gates

Status: Closed
Last updated: 2026-05-25

## Current Evidence

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-25 | AWL-010 | Workstream docs opened from Admin Web V2 audit and existing closeouts. | Pass. Scope, milestones, task ledger, and handoff created. |
| 2026-05-25 | AWL-020 | `cd apps/admin-web && npm run check` | Pass. TypeScript accepts `/libraries/:libraryId`, library detail data-source composition, and Admin API client metadata-profile route usage. |
| 2026-05-25 | AWL-020 | `cd apps/admin-web && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts` | Pass. Route, fallback, unsafe-text, Admin API route, and data-source hybrid fallback coverage passed. |
| 2026-05-25 | AWL-040 | `cd apps/admin-web && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts` | Pass. Shell and library-management localization rendering is covered in route tests while API query values remain unchanged. |
| 2026-05-25 | AWL-020/AWL-040 | `cd apps/admin-web && npm run test` | Pass. Full Admin Web Vitest suite passed, 4 files / 75 tests. |
| 2026-05-25 | AWL-020/AWL-040 | `cd apps/admin-web && npm run build` | Pass. Production build completed; Vite emitted the existing large-chunk warning for the app bundle. |
| 2026-05-25 | AWL-020/AWL-040 | `git diff --check` | Pass. No whitespace errors. |
| 2026-05-25 | AWL-020/AWL-040 | Playwright CLI smoke against `http://127.0.0.1:5180/libraries` and `/libraries/library-anime` at `1440x1000` and `390x844`. | Pass. Both routes rendered nonblank, localized English/Chinese library management copy was visible, document-level horizontal overflow was false, console had zero errors, and parent/back links did not mark themselves as the current page on detail. |
| 2026-05-25 | AWL-030 | `cd apps/admin-web && npm run generate:admin-api` | Pass. Generated Admin API contract includes `libraryScan`, `libraryNfoImport`, `libraryNfoExport`, and `AdminJobCommandResponse`. |
| 2026-05-25 | AWL-030 | `cd apps/admin-web && npm run check` | Pass. TypeScript accepts Metadata Profile full-replacement editing, library command actions, public-read Source inventory bridge summaries, and generated Admin API route types. |
| 2026-05-25 | AWL-030 | `cd apps/admin-web && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts` | Pass. 3 files / 76 tests. Covers detail management rendering, full-profile replacement, command confirmation, Admin API command routes, bridge fallback, and unsafe-text exclusions. |
| 2026-05-25 | AWL-030 | `cd apps/admin-web && npm run build` | Pass. Production build completed; Vite emitted the existing large-chunk warning for the app bundle. |
| 2026-05-25 | AWL-030 | `cargo fmt --all` | Pass. Rust formatting applied after Admin API route changes. |
| 2026-05-25 | AWL-030 | `cargo nextest run -p nako-server admin_library_command_routes_queue_background_jobs` | Pass. Admin `/scan`, `/nfo/import`, and `/nfo/export` command wrappers enqueue accepted background jobs with expected kinds and redaction-safe job fields. |
| 2026-05-25 | AWL-030 | `cargo nextest run -p nako-server admin_library_metadata_profile_route_reads_and_persists_updates` | Pass. Metadata Profile Admin route still reads and persists updates. |
| 2026-05-25 | AWL-030 | `cargo nextest run -p nako-api admin_contract` | Pass. 5 admin contract tests passed, including generated contract parity and public/client boundary checks. |
| 2026-05-25 | AWL-030 | `git diff --check` | Pass. No whitespace errors. |
| 2026-05-25 | AWL-030 | Playwright CLI smoke against `http://127.0.0.1:5181/libraries/library-anime` at `1440x1000` and `390x844`. | Pass. Route rendered Metadata Profile, full replacement notice, Source inventory, Operations, edit form, and scan confirmation state; document-level horizontal overflow was false, console had zero errors, and unsafe raw fields were absent. |
| 2026-05-25 | AWL-030 | `review-workstream` self-review against `DESIGN.md`, ADR 0027, task ledger, and current diff. | Pass after fix. Review found and fixed cross-library job summary leakage in hybrid/mock Source inventory; no blocking findings remain. |
| 2026-05-25 | AWL-050 | `PARITY_GAP_SPLIT.md` | Pass. Re-scored remaining Admin Web V2 management gaps and split bounded follow-on lane candidates for media browsing/item detail, settings/network mutation authority, users/library access, governance repair actions, Addon operations mutations, playback support detail, and i18n expansion. |
| 2026-05-25 | AWL-050 | `git diff --check` | Pass. No whitespace errors after parity split documentation updates. |
| 2026-05-25 | AWL-060 | `cd apps/admin-web && npm run check && npm run test && npm run build` | Pass. TypeScript check passed, full Admin Web Vitest suite passed 4 files / 78 tests, and production build completed with the existing large-chunk warning. |
| 2026-05-25 | AWL-060 | `cargo fmt --check` | Pass. Rust formatting is clean after Admin route changes. |
| 2026-05-25 | AWL-060 | `cargo nextest run -p nako-server admin_library_command_routes_queue_background_jobs` | Pass. Admin scan/NFO command wrappers still enqueue expected accepted jobs. |
| 2026-05-25 | AWL-060 | `cargo nextest run -p nako-api admin_contract` | Pass. 5 admin contract tests passed, including generated contract parity and public/Admin boundary checks. |
| 2026-05-25 | AWL-060 | Playwright CLI smoke against `http://127.0.0.1:5181/libraries` and `/libraries/library-anime` at desktop/mobile widths. | Pass. Routes rendered nonblank, library detail showed Metadata Profile and Operations, document-level horizontal overflow was false, console had zero errors, and unsafe raw fields were absent. |
| 2026-05-25 | AWL-060 | `git diff --check` | Pass. No whitespace errors at closeout. |
| 2026-05-25 | AWL-060 | `review-workstream` and `close-workstream` audit. | Pass. No blocking findings; remaining parity work is split to follow-on lanes in `PARITY_GAP_SPLIT.md` and `CLOSEOUT.md`. |

## Gate Set

### Targeted Frontend Gate

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/dataSource.test.ts
```

Use after route/data-source changes that do not touch Admin API client request
shapes.

### Admin API Client Gate

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts
```

Use when adding metadata profile, scan/NFO, or public-read bridge methods.

### Full Admin Web Gate

```bash
cd apps/admin-web
npm run check
npm run test
npm run build
```

Run before closeout or after broad localization changes.

### Contract Sync Gate

```bash
cd apps/admin-web
npm run generate:admin-api
npm run check
```

Use only if the generated Admin API contract is touched or suspected stale.

### Backend Admin Command Gate

```bash
cargo nextest run -p nako-server admin_library_command_routes_queue_background_jobs
cargo nextest run -p nako-api admin_contract
```

Use when Admin Web gains Admin API route constants, generated contract shape, or
library command wrappers.

### Browser Smoke Gate

Verify desktop `1440x1000` and mobile `390x844` for:

- `/libraries`
- `/libraries/:libraryId`
- any locale-aware route added in this lane

Checks:

- nonblank route content;
- no document-level horizontal overflow;
- no console errors in the mocked/fallback path;
- no unsafe rendered terms such as raw tokens, Secret Reference values, Source
  Locators, local paths, raw roots, raw provider payloads, or raw response
  bodies.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or in a linked review
note.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-library-management-and-localization/DESIGN.md`
- `docs/workstreams/admin-web-v2-library-management-and-localization/TODO.md`
- `docs/workstreams/admin-web-v2-library-management-and-localization/MILESTONES.md`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/features/libraries/`
- `apps/admin-web/src/adminApi/`
- `apps/admin-web/src/i18n/`

## Notes

Fresh verification is required before marking AWL tasks, this Codex goal, or
the workstream complete.

AWL-020 deliberately left live source inventory and library scan/NFO mutations
as follow-on work. AWL-030 wires those through safe boundaries: source inventory
uses an explicitly named public-read bridge summary, while scan/NFO mutations
use Admin API command wrappers and require explicit user confirmation.
