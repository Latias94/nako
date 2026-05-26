# Admin Web V2 Item Artwork Selection - Evidence And Gates

Status: Closed
Last updated: 2026-05-25

## Current Evidence

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-25 | AWA-010 | Workstream opened from GAR closeout and MBG follow-on order. | Pass. Scope, non-goals, milestones, task ledger, gates, readiness stub, and handoff created. |
| 2026-05-25 | AWA-010 | `Test-Path docs/workstreams/admin-web-v2-item-artwork-selection` before opening. | Pass. No existing item artwork Admin Web V2 lane was present. |
| 2026-05-25 | AWA-010 | Initial route inventory from `crates/nako-server/src/http/admin.rs`, `docs/api/HTTP_API.md`, and `apps/admin-web/src/adminApi/generated/contract.ts`. | Pass. Backend/HTTP docs expose item artwork gallery/select/unpublish, but generated Admin Web contract lacks those route constants and DTOs. |
| 2026-05-25 | AWA-020 | `ROUTE_API_READINESS.md` | Pass. Accepted backend item artwork gallery/select/unpublish routes for the first Admin Web artwork slice, documented safe/forbidden fields, confirmed mutation fallback rules, and split generated contract coverage to AWA-030. |
| 2026-05-25 | AWA-020 | `rg -n "AdminManagedArtwork\\|itemArtwork\\|items/\\{item_id\\}/artwork" apps/admin-web/src/adminApi/generated/contract.ts crates/nako-api/src/admin_contract.rs crates/nako-server/src/http/admin.rs docs/api/HTTP_API.md` plus targeted reads of `nako-api::admin::managed_artwork` and server artwork tests. | Pass. Backend/HTTP docs and API/server redaction tests exist; generated Admin Web contract source/output currently lacks item artwork routes and DTOs. |
| 2026-05-25 | AWA-020 | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |
| 2026-05-25 | AWA-030 | TDD red: `cd apps/admin-web && npm run test -- adminApi/client.test.ts` after adding the client route test. | Failed as expected. `getItemArtworkGallery` was not yet implemented, proving the test guarded the missing Admin Web item artwork bridge. |
| 2026-05-25 | AWA-030 | `cd apps/admin-web && npm run generate:admin-api` | Pass. Regenerated `apps/admin-web/src/adminApi/generated/contract.ts` from `crates/nako-api/src/admin_contract.rs` after adding item artwork route constants and DTOs. |
| 2026-05-25 | AWA-030 | `cd apps/admin-web && npm run check` | Pass. TypeScript accepts generated item artwork DTOs and `AdminApiClient` methods. |
| 2026-05-25 | AWA-030 | `cd apps/admin-web && npm run test -- adminApi/client.test.ts` | Pass. 16 client tests passed, including gallery GET, select POST body, encoded item ID, and unpublish DELETE route coverage. |
| 2026-05-25 | AWA-030 | `cargo nextest run -p nako-api admin_contract` | Pass. 5 focused `nako-api` admin contract tests passed, including generated Admin Web contract sync and public-client route exclusion. |
| 2026-05-25 | AWA-030 | `cargo fmt --all --check` | Pass. Rust formatting is clean. |
| 2026-05-25 | AWA-030 | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |
| 2026-05-25 | AWA-040 | TDD red: `cd apps/admin-web && npm run test -- adminApi/dataSource.test.ts` after adding the gallery data-source test. | Failed as expected. `loadItemArtworkGallery` was not yet implemented, proving the missing safe projection bridge. |
| 2026-05-25 | AWA-040 | TDD red: `cd apps/admin-web && npm run test -- App.test.tsx` after adding the `/items/:itemId/artwork` route test. | Failed as expected with route-level Not Found before the gallery route was registered. |
| 2026-05-25 | AWA-040 | `cd apps/admin-web && npm run test -- adminApi/dataSource.test.ts` | Pass. 23 data-source tests passed, including item artwork gallery generated query params, safe projection, fallback, and unsafe field exclusions. |
| 2026-05-25 | AWA-040 | `cd apps/admin-web && npm run test -- App.test.tsx` | Pass. 64 route shell tests passed, including item detail gallery link, read-only artwork gallery rendering, deterministic fallback, no select/unpublish buttons, and unsafe text exclusions. |
| 2026-05-25 | AWA-040 | `cd apps/admin-web && npm run check` | Pass. TypeScript accepts the new route, page props, data-source contract, and artwork summary types. |
| 2026-05-25 | AWA-040 | `cd apps/admin-web && npm run test -- App.test.tsx adminApi/dataSource.test.ts` | Pass. 87 focused frontend tests passed. |
| 2026-05-25 | AWA-040 | Browser smoke on `http://127.0.0.1:5176/items/item-unknown-1/artwork?limit=20&offset=0` via local Vite fallback. | Pass. Desktop and 390x844 mobile rendered the read-only gallery with no console errors, no document horizontal overflow, and no unsafe artwork source/storage/cache/path/hash/token text. |
| 2026-05-25 | AWA-040 | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |
| 2026-05-25 | AWA-040 verify | `cd apps/admin-web && npm run check`; `cd apps/admin-web && npm run test -- App.test.tsx adminApi/dataSource.test.ts`; `git diff --check`. | Pass. Fresh verify before DONE claim: TypeScript passed, 87 focused frontend tests passed, and no whitespace errors were found. Broader full Admin Web/build gates are deferred to AWA-060 because AWA-040 is gallery-only and did not change Rust/backend contract source. |
| 2026-05-25 | AWA-050 | TDD red: `cd apps/admin-web && npm run test -- App.test.tsx -- -t "selects item artwork only after explicit confirmation"` before UI implementation. | Failed as expected. The item artwork gallery had no `Prepare select artifact-backdrop-1` control, proving select was still read-only before the guarded mutation slice. |
| 2026-05-25 | AWA-050 | `cd apps/admin-web && npm run test -- App.test.tsx -- -t "selects item artwork only after explicit confirmation"` | Pass. The route prepares select without calling the mutation, then confirms with item/kind/artifact scope and renders the safe selection result. |
| 2026-05-25 | AWA-050 | `cd apps/admin-web && npm run test -- adminApi/dataSource.test.ts -- -t "maps item artwork select and unpublish"` | Pass. Data source maps select/unpublish mutation results into redaction-safe summaries, posts/deletes the generated routes, strips unsafe image URLs, and rejects HTTP 503 instead of returning mock mutation success. |
| 2026-05-25 | AWA-050 | `cd apps/admin-web && npm run test -- App.test.tsx -- -t "item artwork"` | Pass. Seven item artwork route tests passed, covering guarded route rendering, select confirmation, unpublish confirmation, unavailable mutation error visibility, fallback gallery, and unsafe gallery/mutation result exclusions. |
| 2026-05-25 | AWA-050 verify | `cd apps/admin-web && npm run check`; `cd apps/admin-web && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`; `git diff --check`. | Pass. Fresh verify before DONE claim: TypeScript passed, 108 focused frontend/client/data-source tests passed, and no whitespace errors were found. Browser smoke and full Admin Web/build gates remain assigned to AWA-060. |
| 2026-05-25 | AWA-050 supplemental smoke | Playwright CLI on `http://127.0.0.1:5176/items/item-unknown-1/artwork?limit=20&offset=0` with mocked Admin item artwork gallery/select/unpublish responses. | Pass. Desktop route rendered live mocked data, select confirmation produced `Selection updated`, unpublish confirmation produced `Selection unpublished`, console had no errors in the mocked run, desktop/mobile had no document horizontal overflow, and mobile text check found no unsafe artwork source/storage/path/hash/token text. Full browser smoke remains AWA-060. |
| 2026-05-25 | AWA-060 | `cd apps/admin-web && npm run check` | Pass. Fresh TypeScript project build accepted the Admin Web V2 route/action surface. |
| 2026-05-25 | AWA-060 | `cd apps/admin-web && npm run test` | Pass. 4 Vitest files passed with 110 tests, covering App route shell, admin components, client, and data source behavior. |
| 2026-05-25 | AWA-060 | `cd apps/admin-web && npm run build` | Pass. Production Vite build completed; Vite emitted the existing large-chunk warning for the bundled app asset. |
| 2026-05-25 | AWA-060 | Playwright CLI browser smoke at `1440x1000` on `http://127.0.0.1:5176/items/item-unknown-1` and `/items/item-unknown-1/artwork?limit=20&offset=0` with mocked Admin artwork gallery/select/unpublish responses. | Pass. Item detail rendered fallback-safe item facts and an `Open Artwork Gallery` link; gallery rendered live mocked Admin API data; select confirmation produced `Selection updated`; unpublish confirmation produced `Selection unpublished`; no browser console errors were reported. |
| 2026-05-25 | AWA-060 | Playwright CLI mobile smoke at `390x844` on `/items/item-unknown-1/artwork?limit=20&offset=0`. | Pass. Gallery remained nonblank with no document horizontal overflow (`scrollWidth=390`, `clientWidth=390`), no console errors, and no rendered unsafe artwork source/storage/path/hash/token text. |
| 2026-05-25 | AWA-060 | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |
| 2026-05-25 | AWA-070 review | Workstream closeout review against `DESIGN.md`, `TODO.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`, ADR 0027, and targeted implementation diffs. | Pass. No blocking workstream-compliance or code-quality findings. Remaining lifecycle, repair, provider search, upload, NFO, settings, users/permissions, and full-site i18n breadth remains split. |
| 2026-05-25 | AWA-070 verify | `cd apps/admin-web && npm run check` | Pass. TypeScript build accepted the route/action surface during closeout. |
| 2026-05-25 | AWA-070 verify | `cd apps/admin-web && npm run test` | Pass. 4 Vitest files passed with 110 tests, including route shell, client, data-source, confirmation, mutation error, fallback, and unsafe text coverage. |
| 2026-05-25 | AWA-070 verify | `cd apps/admin-web && npm run build` | Pass. Production Vite build completed; Vite emitted the existing large-chunk warning for the bundled app asset. |
| 2026-05-25 | AWA-070 verify | `cargo nextest run -p nako-api admin_contract` | Pass. 5 focused `nako-api` admin contract tests passed, including generated Admin Web contract sync and public-client route exclusion. |
| 2026-05-25 | AWA-070 verify | `cargo fmt --all --check` | Pass. Rust formatting is clean. |
| 2026-05-25 | AWA-070 verify | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |
| 2026-05-25 | AWA-070 browser smoke decision | AWA-060 Playwright CLI desktop/mobile browser smoke evidence reused for closeout. | Pass by unchanged runtime scope. AWA-070 changed only closeout docs, so the official AWA-060 smoke remains the relevant runtime evidence for item detail, artwork gallery, select confirmation, unpublish confirmation, overflow, console errors, and unsafe text exclusions. |

## Gate Set

### Route/API Readiness Gate

```bash
git diff --check
```

Use after AWA-020 planning-only route/API readiness updates.

### Generated Contract Gate

```bash
cd apps/admin-web
npm run generate:admin-api
npm run check
npm run test -- adminApi/client.test.ts
```

Use after adding Admin Web contract coverage and client bridge methods.

### Targeted Frontend Gate

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts
```

Use after artwork gallery/action route, data-source, or client bridge changes.

### Full Admin Web Gate

```bash
cd apps/admin-web
npm run check
npm run test
npm run build
```

Run before closeout or after broad route/control changes.

### Rust/Admin Contract Gate

Run focused `cargo nextest` or `cargo test` commands for `nako-api` contract
generation and Managed Artwork redaction tests when AWA changes Rust contract
source or backend DTOs.

### Browser Smoke Gate

Verify desktop `1440x1000` and mobile `390x844` for:

- `/items/:itemId`
- item artwork gallery route
- one select confirmation path
- one unpublish confirmation path

Checks:

- nonblank route content;
- no document-level horizontal overflow;
- no console errors in mocked/fallback path;
- no unsafe `source_uri`, `storage_uri`, `managed-artwork://`, cache URI,
  local path, artifact root, content hash, provider URL/query string, file
  content, token, or credential text.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Use
`verify-rust-workstream` before completion claims.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-item-artwork-selection/DESIGN.md`
- `docs/workstreams/admin-web-v2-item-artwork-selection/TODO.md`
- `docs/workstreams/admin-web-v2-item-artwork-selection/ROUTE_API_READINESS.md`
- `docs/api/HTTP_API.md`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-api/src/admin/managed_artwork.rs`
- `crates/nako-api/src/admin_contract.rs`
- `apps/admin-web/src/adminApi/generated/contract.ts`
- `docs/workstreams/admin-web-v2-item-artwork-selection/CLOSEOUT.md`

## Notes

AWA-070 closeout verification is complete. This workstream is closed; new
Admin Web V2 breadth should open or reuse a bounded follow-on lane rather than
extending this one.
