# Admin Web V2 Catalog Repair Actions - Evidence And Gates

Status: Complete
Last updated: 2026-05-25

## Current Evidence

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-25 | CRA-010 | `Test-Path docs/workstreams/admin-web-v2-catalog-repair-actions` before opening. | Pass. No existing Catalog Repair Actions lane was present. |
| 2026-05-25 | CRA-010 | Workstream opened from AWA closeout and MBG follow-on order. | Pass. Scope, non-goals, milestones, task ledger, gates, route/API readiness stub, and handoff created. |
| 2026-05-25 | CRA-010 | Initial route inventory from `docs/api/HTTP_API.md`, `crates/nako-api/src/admin_contract.rs`, `crates/nako-api/src/admin/catalog_governance.rs`, `crates/nako-server/src/http/admin.rs`, and Admin Web catalog route files. | Pass. Read-only Catalog Governance list exists in Admin API and Admin Web. Detail/review-plan/mutation routes for repair actions are not yet accepted and remain CRA-020/CRA-030 prerequisites. |
| 2026-05-25 | CRA-020 | `ROUTE_API_READINESS.md` | Pass. Accepted one Media Item plus one Provider Mapping accept/reject as the first repair action; documented missing detail/review-plan/mutation routes, generated contract gaps, safe response fields, forbidden evidence, and split repair classes. |
| 2026-05-25 | CRA-020 | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |
| 2026-05-25 | CRA-030 | `cargo nextest run -p nako-server admin_v1_catalog_governance_provider_mapping_review_plan_is_redacted --no-fail-fast` | Pass. Proves Admin detail and Provider Mapping review-plan routes return safe item/mapping/subject/readiness/boundary summaries and omit `evidence_value`, `local:///`, raw evidence token text, private fingerprint text, and local temp paths. |
| 2026-05-25 | CRA-030 | `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture` | Pass. Generated Admin contract source includes new Catalog Governance detail and Provider Mapping review-plan route constants and DTO names. |
| 2026-05-25 | CRA-030 | `cd apps/admin-web && npm run generate:admin-api` | Pass. Regenerated `apps/admin-web/src/adminApi/generated/contract.ts` from `crates/nako-api/src/admin_contract.rs`. |
| 2026-05-25 | CRA-030 | `cd apps/admin-web && npm run check` | Pass. TypeScript accepts generated Catalog Governance detail/review-plan types and `AdminApiClient` wrappers. |
| 2026-05-25 | CRA-030 | `cd apps/admin-web && npm run test -- adminApi/client.test.ts` | Pass. 17 client tests passed, including encoded Catalog Governance item detail and Provider Mapping review-plan route behavior. |
| 2026-05-25 | CRA-030 | `cargo fmt --all --check` | Pass. Rust formatting is clean after applying `cargo fmt --all`. |
| 2026-05-25 | CRA-030 | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |
| 2026-05-25 | CRA-040 | `cargo nextest run -p nako-server admin_v1_catalog_governance_provider_mapping_review_mutates_idempotently --no-fail-fast` | Pass. Proves confirmed Provider Mapping review mutation updates candidate to accepted, reports changed/idempotent replay semantics, persists the mapping status, and omits unsafe evidence/path/token/fingerprint strings. |
| 2026-05-25 | CRA-040 | `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture` | Pass. Generated Admin contract source includes confirmed Catalog Governance Provider Mapping review route constants and response DTO names. |
| 2026-05-25 | CRA-040 | `cd apps/admin-web && npm run generate:admin-api` | Pass. Regenerated Admin Web contract with the confirmed Catalog Governance Provider Mapping review route and response DTO. |
| 2026-05-25 | CRA-040 | `cd apps/admin-web && npm run check` | Pass. TypeScript accepts Catalog Governance review mutation client/data-source wrappers and safe summary types. |
| 2026-05-25 | CRA-040 | `cd apps/admin-web && npm run test -- adminApi/client.test.ts adminApi/dataSource.test.ts` | Pass. 45 tests passed, including encoded mutation route behavior, redaction-safe detail/review-plan/review result projections, deterministic read fallback, and no fake mutation fallback. |
| 2026-05-25 | CRA-040 | `cargo fmt --all --check` | Pass. Rust formatting is clean after applying `cargo fmt --all`. |
| 2026-05-25 | CRA-040 | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |
| 2026-05-25 | CRA-050 | `cd apps/admin-web && npm run check` | Pass. TypeScript accepts the route-owned Catalog Governance repair context route, Provider Mapping selector, review-plan flow, and mutation result rendering. |
| 2026-05-25 | CRA-050 | `cd apps/admin-web && npm run test -- App.test.tsx` | Pass. 73 App route tests passed, including Catalog Governance repair route render, URL decision state, explicit confirmation before mutation, mutation unavailable error, and unsafe text exclusion. |
| 2026-05-25 | CRA-060 | `cd apps/admin-web && npm run check` | Pass. Frontend route, client, data-source, and generated contract types are clean. |
| 2026-05-25 | CRA-060 | `cd apps/admin-web && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts` | Pass. 118 focused frontend tests passed. |
| 2026-05-25 | CRA-060 | `cargo nextest run -p nako-server admin_v1_catalog_governance_provider_mapping_review_mutates_idempotently --no-fail-fast` | Pass. Confirmed mutation route still passes idempotency and redaction checks. |
| 2026-05-25 | CRA-060 | `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture` | Pass. Admin contract route constants remain in sync. |
| 2026-05-25 | CRA-060 | `cargo fmt --all --check` | Pass. Rust formatting is clean. |
| 2026-05-25 | CRA-060 | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |
| 2026-05-25 | CRA-070 | `cd apps/admin-web && npm run test` | Pass. Full Admin Web test suite passed: 4 files, 120 tests. |
| 2026-05-25 | CRA-070 | `cd apps/admin-web && npm run build` | Pass. Production build completed. Vite reported existing-style chunk-size/plugin-timing warnings only. |
| 2026-05-25 | CRA-070 | `cargo nextest run -p nako-server catalog_governance --no-fail-fast` | Pass. Three Catalog Governance server tests passed: list redaction, review-plan redaction, and confirmed mutation idempotency. |
| 2026-05-25 | CRA-060 | Browser smoke via `playwright-cli` against `http://127.0.0.1:5177/` | Pass. Desktop queue at `1440x1000` had no document horizontal overflow (`scrollWidth=1440`), no console errors, and no unsafe rendered text. |
| 2026-05-25 | CRA-060 | Browser smoke via `playwright-cli` for `/catalog/governance/item-low-confidence?mapping_id=mapping-tmdb-603&decision=accept` | Pass. Repair context rendered live mocked detail/review-plan, explicit prepare/confirm flow produced `accepted` and `new result`, no console errors, and no unsafe rendered text. |
| 2026-05-25 | CRA-060 | Browser smoke via `playwright-cli` at mobile `390x844` | Pass. Repair context had no document horizontal overflow (`scrollWidth=390`), no console errors, and no unsafe rendered text. |
| 2026-05-25 | CRA-060 | Browser smoke failure path via `playwright-cli` non-JSON mutation mock | Pass. Visible `Admin API request returned a non-JSON response` error appeared without a fake success result and with no console errors. |

## Gate Set

### Route/API Readiness Gate

```bash
git diff --check
```

Use after CRA-020 planning-only route/API readiness updates.

### Generated Contract Gate

```bash
cd apps/admin-web
npm run generate:admin-api
npm run check
npm run test -- adminApi/client.test.ts
```

Use after adding or changing Admin Web contract coverage and client bridge
methods.

### Rust/Admin API Gate

Use focused `cargo nextest` commands for repository, API DTO, and server route
tests affected by CRA changes. Likely packages:

```bash
cargo nextest run -p nako-api catalog_governance
cargo nextest run -p nako-server catalog_governance
cargo nextest run -p nako-db catalog_governance
```

Adjust filters to the implemented route/action names and record the exact
commands.

### Targeted Frontend Gate

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts
```

Use after route, confirmation, client, or data-source changes.

### Full Admin Web Gate

```bash
cd apps/admin-web
npm run check
npm run test
npm run build
```

Run before closeout or after broad route/control changes.

### Browser Smoke Gate

Verify desktop `1440x1000` and mobile `390x844` for:

- `/catalog/governance`;
- one Catalog Governance repair context route or modal;
- one review-plan path;
- one confirmed mutation path;
- one visible mutation failure path.

Checks:

- nonblank route content;
- no document-level horizontal overflow;
- no console errors in mocked/fallback path;
- no unsafe Local Inference evidence value, Source Locator, local path,
  provider raw body, provider URL/query string, NFO path/XML/content, token,
  credential, or arbitrary raw metadata text.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Use
`verify-rust-workstream` before completion claims.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-catalog-repair-actions/DESIGN.md`
- `docs/workstreams/admin-web-v2-catalog-repair-actions/TODO.md`
- `docs/workstreams/admin-web-v2-catalog-repair-actions/ROUTE_API_READINESS.md`
- `docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance/FOLLOW_ON_SPLIT.md`
- `docs/workstreams/admin-web-v2-item-artwork-selection/CLOSEOUT.md`
- `docs/workstreams/admin-catalog-governance-read-model/DESIGN.md`
- `docs/api/HTTP_API.md`
- `crates/nako-api/src/admin/catalog_governance.rs`
- `crates/nako-api/src/admin_contract.rs`
- `crates/nako-server/src/http/admin.rs`
- `apps/admin-web/src/features/catalog/CatalogGovernancePage.tsx`
- `apps/admin-web/src/adminApi/generated/contract.ts`

## Notes

Fresh verification is required before marking CRA tasks, this Codex goal, or
the workstream complete.
