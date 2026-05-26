# Admin Web V2 Catalog Repair Actions - TODO

Status: Complete
Last updated: 2026-05-25

Task IDs use the `CRA` prefix.

## M0 - Scope And Evidence Freeze

- [x] CRA-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-web-v2-catalog-repair-actions,docs/workstreams/README.md]
  Goal: Open the lane, freeze scope, non-goals, validation gates, repair
  readiness order, and first executable task.
  Validation: Workstream docs exist and agree with MBG, GAR, AWA closeouts,
  ADR 0027, the M60 Catalog Governance read model, and current Admin Web route
  shape.
  Review: This lane must not absorb settings mutation, users/permissions/
  Library Access, full-site i18n, Generated Artifact review, Managed Artwork
  selection, item NFO writes, playback controls, or arbitrary metadata edit
  forms.
  Evidence: `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `ROUTE_API_READINESS.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`,
  `HANDOFF.md`.
  Result: DONE 2026-05-25. Lane opened from AWA closeout and MBG follow-on
  order. Current generated Admin Web contract exposes only the read-only
  Catalog Governance list route for this surface.
  Handoff: Continue with CRA-020 route/API repair-readiness audit.

## M1 - Route/API Repair Readiness

- [x] CRA-020 [owner=codex] [deps=CRA-010] [scope=docs/workstreams/admin-web-v2-catalog-repair-actions,docs/api/HTTP_API.md,crates/nako-api/src/admin_contract.rs,crates/nako-api/src/admin,crates/nako-server/src/http,crates/nako-core/src,crates/nako-db/src,apps/admin-web/src/adminApi/generated/contract.ts]
  Goal: Audit Catalog Governance detail/readiness, Provider Mapping
  accept/reject, Local Inference evidence, duplicate-source repair, unknown-item
  classification, hierarchy repair, and generated Admin Web contract gaps before
  implementation.
  Validation: `git diff --check`.
  Review: If no redaction-safe detail/review-plan route exists, split CRA-030
  before UI. If Provider Mapping accept/reject semantics are not sufficient for
  the first action, document and select a safer first action. No UI mutation
  controls may be added from this task.
  Evidence: readiness notes and updated handoff/evidence.
  Result: DONE 2026-05-25. `ROUTE_API_READINESS.md` accepts one Media
  Item plus one Provider Mapping accept/reject as the first catalog repair
  action, documents route/API gaps, safe response fields, forbidden data, and
  split repair classes. Admin Web UI remains blocked until CRA-030/CRA-040 add
  generated Admin contract coverage and backend routes.
  Handoff: Continue with CRA-030 backend/API detail and review-plan route work.

## M2 - Repair Detail And Review Plan

- [x] CRA-030 [owner=backend] [deps=CRA-020] [scope=crates/nako-core,crates/nako-db,crates/nako-api,crates/nako-server,docs/api/HTTP_API.md,apps/admin-web/src/adminApi/generated/contract.ts]
  Goal: Add the redaction-safe Admin API detail/readiness and dry-run or
  review-plan route required for the first catalog repair action.
  Validation: focused `cargo nextest` gates for repository/API/server
  redaction behavior; `cd apps/admin-web && npm run generate:admin-api && npm
  run check && npm run test -- adminApi/client.test.ts` when generated contract
  changes.
  Review: Review-plan responses must not expose raw Local Inference evidence,
  Source Locators, local paths, provider raw bodies, NFO paths/XML, tokens, or
  arbitrary metadata payloads.
  Evidence: backend/API tests, generated contract diff, and redaction inventory.
  Result: DONE 2026-05-25. Added redaction-safe Catalog Governance item detail
  and Provider Mapping review-plan Admin API routes, DTOs, generated Admin Web
  contract coverage, `AdminApiClient` wrappers, HTTP API docs, and tests proving
  unsafe evidence/path/token/fingerprint strings are not returned.
  Handoff: Continue with CRA-040 confirmed Provider Mapping mutation.

## M3 - Confirmed Repair Mutation

- [x] CRA-040 [owner=backend] [deps=CRA-030] [scope=crates/nako-core,crates/nako-db,crates/nako-api,crates/nako-server,docs/api/HTTP_API.md,apps/admin-web/src/adminApi/generated/contract.ts,apps/admin-web/src/adminApi/client.ts,apps/admin-web/src/adminApi/client.test.ts,apps/admin-web/src/adminApi/dataSource.ts,apps/admin-web/src/adminApi/dataSource.test.ts]
  Goal: Add the first real catalog repair mutation with idempotent result
  semantics and redaction-safe Admin Web client/data-source wrappers.
  Validation: focused Rust repository/server tests; `cd apps/admin-web && npm
  run generate:admin-api && npm run check && npm run test --
  adminApi/client.test.ts adminApi/dataSource.test.ts`.
  Review: Mutation fallback must not report fake success; failures must be
  visible; result rendering data must stay scoped to item/action/mapping IDs and
  safe audit summaries.
  Evidence: mutation tests, idempotency tests, generated contract sync, and
  no-fake-fallback tests.
  Result: DONE 2026-05-25. Added confirmed Provider Mapping review mutation
  route, idempotent status-change response semantics, generated Admin Web
  contract coverage, `AdminApiClient` wrapper, redaction-safe `AdminDataSource`
  detail/review-plan/review result projections, HTTP API docs, and no fake
  mutation fallback tests.
  Handoff: Continue with CRA-050 Admin Web route/confirmation UI.

## M4 - Admin Web Repair UI

- [x] CRA-050 [owner=frontend] [deps=CRA-040] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/features/catalog,apps/admin-web/src/adminApi,apps/admin-web/src/App.test.tsx,apps/admin-web/src/adminApi/client.test.ts,apps/admin-web/src/adminApi/dataSource.test.ts,apps/admin-web/src/styles.css]
  Goal: Add route-owned Catalog Governance repair UI reachable from
  `/catalog/governance`, render safe review-plan context, require explicit
  prepare/confirm steps, show mutation results, and make failures visible.
  Validation: `cd apps/admin-web && npm run check && npm run test --
  App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`.
  Review: Do not render raw evidence, provider bodies, Source Locators, local
  paths, NFO content, tokens, or unsafe mutation payloads. Do not submit any
  mutation before confirmation.
  Evidence: route, data-source, client, confirmation, mutation failure,
  fallback, and redaction tests.
  Result: DONE 2026-05-25. Added `/catalog/governance/{item_id}` repair
  context route, queue-to-review navigation, Provider Mapping selector,
  decision URL state, review-plan preview, explicit prepare/confirm mutation
  flow, result/failure states, and unsafe text exclusion tests.
  Handoff: Continue with CRA-060 verification and browser smoke.

## M5 - Verification And Browser Smoke

- [x] CRA-060 [owner=codex] [deps=CRA-050] [scope=apps/admin-web,crates/nako-api,crates/nako-server,docs/workstreams/admin-web-v2-catalog-repair-actions]
  Goal: Run focused/full gates and browser smoke for queue, repair context,
  review-plan, confirmation, mutation result, failure, and unsafe text
  exclusions.
  Validation: `cd apps/admin-web && npm run check && npm run test && npm run
  build`; relevant `cargo nextest` gates; `cargo fmt --all --check`; `git diff
  --check`; browser smoke.
  Review: Browser smoke must check desktop/mobile, no horizontal overflow, no
  console errors, and no unsafe rendered Local Inference/provider/path/NFO/token
  text.
  Evidence: `EVIDENCE_AND_GATES.md`, browser smoke notes.
  Result: DONE 2026-05-25. Focused Rust/API/Admin Web gates passed, browser
  smoke covered queue, repair context, review-plan, confirmation success,
  visible failure, desktop/mobile overflow checks, console error checks, and
  unsafe text exclusion.
  Handoff: Continue with CRA-070 closeout.

## M6 - Closeout

- [x] CRA-070 [owner=planner] [deps=CRA-060] [scope=docs/workstreams/admin-web-v2-catalog-repair-actions]
  Goal: Close the lane or split remaining catalog repair breadth and update
  status fields.
  Validation: final gates, browser smoke evidence, review, verify, closeout
  docs, and `git diff --check`.
  Review: `review-workstream` and `verify-rust-workstream` before completion
  claims.
  Evidence: `CLOSEOUT.md`, `WORKSTREAM.json`, `HANDOFF.md`,
  `EVIDENCE_AND_GATES.md`.
  Result: DONE 2026-05-25. Lane closed with follow-on repair classes split to
  later work: rematch/provider search, duplicate-source merge/split, hierarchy
  repair, NFO writes, arbitrary metadata editing, and broader users/permissions
  or Library Access work.
  Handoff: Recommend the next Admin Web V2 lane.
