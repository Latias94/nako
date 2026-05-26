# Admin Web V2 Catalog Repair Actions - Closeout

Status: Complete
Closed: 2026-05-25

## Delivered

- Catalog Governance route/API readiness audit.
- Redaction-safe Admin API item detail route:
  - `GET /admin/v1/catalog/governance/items/{item_id}`.
- Redaction-safe Provider Mapping review-plan route:
  - `POST /admin/v1/catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review-plan`.
- Confirmed Provider Mapping review mutation:
  - `POST /admin/v1/catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review`.
- Generated Admin Web contract coverage for the new routes and DTOs.
- Admin Web client/data-source wrappers for detail, review-plan, and mutation.
- Route-owned Admin Web repair context:
  - `/catalog/governance/{item_id}`.
- Queue-to-review navigation from `/catalog/governance`.
- Provider Mapping selector, decision URL state, review-plan preview, explicit
  prepare/confirm flow, mutation result, and visible failure state.
- Redaction tests and safe UI rendering tests for raw evidence, Source
  Locators, local paths, provider raw bodies/URLs, NFO XML/paths, tokens,
  credentials, and arbitrary raw payload fields.

## Verification

- `cargo nextest run -p nako-server admin_v1_catalog_governance_provider_mapping_review_mutates_idempotently --no-fail-fast`
- `cargo nextest run -p nako-server catalog_governance --no-fail-fast`
- `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture`
- `cd apps/admin-web && npm run generate:admin-api`
- `cd apps/admin-web && npm run check`
- `cd apps/admin-web && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`
- `cd apps/admin-web && npm run test`
- `cd apps/admin-web && npm run build`
- `cargo fmt --all --check`
- `git diff --check`
- Browser smoke via `playwright-cli` at desktop and mobile sizes.

## Review

No blocking workstream-compliance or code-quality findings remain for this
lane. The implementation stays within ADR 0027's Admin API boundary and does
not add Admin routes to Public Client API, public OpenAPI, SDK, or
`nako-client-protocol` inventories.

## Follow-On Split

The following repair classes remain split to future lanes:

- rematch/provider search;
- duplicate-source merge/split repair;
- hierarchy repair;
- NFO writes and NFO sidecar action states;
- arbitrary metadata editing forms;
- users/permissions and Library Access;
- broader Catalog Governance repair batching or automation.

## Residual Risk

- The repair route currently ships one Provider Mapping status action only.
  Broader repair classes need their own review-plan and mutation boundaries.
- Browser smoke used mocked Admin API responses for clean local verification
  because no live backend Admin API was running with seeded Catalog Governance
  fixtures.
- `npm run build` reports Vite chunk-size/plugin-timing warnings. The build
  succeeds; code-splitting is a future Admin Web performance task, not a blocker
  for this lane.
