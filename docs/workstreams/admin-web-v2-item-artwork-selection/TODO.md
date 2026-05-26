# Admin Web V2 Item Artwork Selection - TODO

Status: Closed
Last updated: 2026-05-25

Task IDs use the `AWA` prefix.

## M0 - Scope And Evidence Freeze

- [x] AWA-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-web-v2-item-artwork-selection,docs/workstreams/README.md]
  Goal: Open the lane, freeze scope, non-goals, validation gates, route/API
  readiness order, and first executable task.
  Validation: Workstream docs exist and agree with GAR closeout and the MBG
  follow-on split.
  Review: This lane must not absorb catalog repair, Generated Artifact review,
  NFO writes, settings mutation, users/permissions/Library Access, full-site
  i18n, or broader Managed Artwork lifecycle/remediation controls.
  Evidence: `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `ROUTE_API_READINESS.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`,
  `HANDOFF.md`.
  Result: DONE 2026-05-25. Lane opened from GAR closeout and the MBG follow-on
  order. Current generated Admin Web contract gap is explicit.
  Handoff: Continue with AWA-020 route/API readiness.

## M1 - Route/API Contract Readiness

- [x] AWA-020 [owner=codex] [deps=AWA-010] [scope=docs/workstreams/admin-web-v2-item-artwork-selection,docs/api/HTTP_API.md,crates/nako-api/src/admin_contract.rs,crates/nako-api/src/admin/managed_artwork.rs,crates/nako-server/src/http/admin.rs,apps/admin-web/src/adminApi/generated/contract.ts]
  Goal: Audit item artwork gallery/select/unpublish route shapes, generated
  Admin Web contract gaps, request bodies, result redaction, fallback needs,
  and route UX shape before UI implementation.
  Validation: `git diff --check`.
  Review: If generated contract coverage is missing, split AWA-030 before UI;
  if DTOs expose unsafe artwork storage/source/cache/hash data, split
  backend/API hardening before rendering.
  Evidence: readiness notes and updated handoff/evidence.
  Result: DONE 2026-05-25. Backend and HTTP docs already expose the first
  item artwork gallery/select/unpublish slice with redaction tests. Generated
  Admin Web contract coverage is missing and is the AWA-030 prerequisite. No
  backend DTO hardening blocker was found for the first UI slice.
  Handoff: Continue with AWA-030 generated contract coverage.

## M2 - Generated Contract And API Bridge

- [x] AWA-030 [owner=codex] [deps=AWA-020] [scope=crates/nako-api/src/admin_contract.rs,apps/admin-web/src/adminApi/generated/contract.ts,apps/admin-web/src/adminApi/client.ts,apps/admin-web/src/adminApi/client.test.ts,apps/admin-web/src/adminApi/types.ts,apps/admin-web/src/adminApi/mockData.ts]
  Goal: Add generated Admin Web contract coverage and explicit client methods
  for item artwork gallery, select, and unpublish.
  Validation: `cd apps/admin-web && npm run generate:admin-api && npm run check && npm run test -- adminApi/client.test.ts`; focused Rust contract test if present.
  Review: Do not hand-edit generated output without updating the Rust contract
  source; no unsafe artwork locator/hash/source fields in exported DTO shapes.
  Evidence: generated contract diff, client tests, and redaction inventory.
  Result: DONE 2026-05-25. Added generated Admin Web route constants and DTOs
  for item artwork gallery/select/unpublish from `nako-api` contract source,
  regenerated `apps/admin-web/src/adminApi/generated/contract.ts`, and added
  `AdminApiClient` methods plus tests for encoded item IDs, select request
  body, and DELETE unpublish route.
  Handoff: Continue with AWA-040 gallery UI.

## M3 - Artwork Gallery UI

- [x] AWA-040 [owner=frontend] [deps=AWA-030] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/features,apps/admin-web/src/adminApi,apps/admin-web/src/App.test.tsx,apps/admin-web/src/adminApi/dataSource.test.ts]
  Goal: Add item-scoped artwork gallery UI reachable from `/items/:itemId`,
  with safe candidate/artifact/selected summaries, deterministic fallback, and
  redaction tests.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/dataSource.test.ts`.
  Review: Do not post select/unpublish mutations from the gallery without
  explicit confirmation.
  Evidence: route, data-source, fallback, and redaction tests.
  Result: DONE 2026-05-25. Added `/items/:itemId/artwork` as a route-owned
  read-only Managed Artwork gallery, linked it from Media Item detail, added
  `AdminDataSource.loadItemArtworkGallery` with safe projection and
  deterministic fallback, and covered route rendering, URL pagination, fallback,
  no select/unpublish buttons, and unsafe text exclusions.
  Handoff: Continue with AWA-050 confirmed actions.

## M4 - Confirmed Select And Unpublish Actions

- [x] AWA-050 [owner=frontend] [deps=AWA-040] [scope=apps/admin-web/src/features,apps/admin-web/src/adminApi,apps/admin-web/src/App.test.tsx,apps/admin-web/src/adminApi/client.test.ts,apps/admin-web/src/adminApi/dataSource.test.ts]
  Goal: Add explicit select/replace and unpublish confirmation flows with
  redacted result rendering.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`.
  Review: Mutation fallback must not report fake success; failures must be
  visible; selection must be item/kind/artifact scoped.
  Evidence: mutation tests, confirmation tests, error tests, and redaction
  tests.
  Result: DONE 2026-05-25. Added guarded select/replace and unpublish actions
  to the item artwork gallery, wired live-only `AdminDataSource` mutation
  wrappers, rendered redaction-safe mutation results, and covered confirmation,
  visible failure, unsafe-result filtering, and no fake mutation fallback.
  Handoff: Continue with AWA-060 verification and browser smoke.

## M5 - Verification And Browser Smoke

- [x] AWA-060 [owner=codex] [deps=AWA-050] [scope=apps/admin-web,docs/workstreams/admin-web-v2-item-artwork-selection]
  Goal: Run focused/full Admin Web gates and browser smoke for item detail,
  artwork gallery, select confirmation, and unpublish confirmation paths.
  Validation: `cd apps/admin-web && npm run check && npm run test && npm run build`; `git diff --check`; browser smoke; focused Rust/Admin contract gates when contract source changes.
  Review: Browser smoke must check desktop/mobile, no horizontal overflow, no
  console errors, and no unsafe rendered artwork source/storage/path/hash/token
  text.
  Evidence: `EVIDENCE_AND_GATES.md`, browser smoke notes.
  Result: DONE 2026-05-25. Full Admin Web check/test/build passed, `git diff
  --check` passed, and Playwright CLI smoke covered item detail navigation,
  artwork gallery, select confirmation, unpublish confirmation, desktop/mobile
  overflow, console errors, and unsafe artwork text exclusions.
  Handoff: Continue with AWA-070 closeout.

## M6 - Closeout

- [x] AWA-070 [owner=planner] [deps=AWA-060] [scope=docs/workstreams/admin-web-v2-item-artwork-selection]
  Goal: Close the lane or split blockers and update status fields.
  Validation: final gates, browser smoke evidence, review, verify, closeout
  docs, and `git diff --check`.
  Review: `review-workstream` and `verify-rust-workstream` before completion
  claims.
  Evidence: `CLOSEOUT.md`, `WORKSTREAM.json`, `HANDOFF.md`,
  `EVIDENCE_AND_GATES.md`.
  Result: DONE 2026-05-25. Review found no blocking workstream compliance or
  code-quality findings. Fresh closeout gates passed: Admin Web
  `check`/`test`/`build`, focused `nako-api` admin contract tests,
  `cargo fmt --all --check`, and `git diff --check`. AWA-060 browser smoke
  remains the official desktop/mobile runtime evidence because AWA-070 changed
  only closeout docs.
  Handoff: Open `admin-web-v2-catalog-repair-actions` next unless settings
  mutation or users/permissions/Library Access is reprioritized.
