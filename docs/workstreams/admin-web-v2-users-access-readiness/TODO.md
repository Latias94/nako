# Admin Web V2 Users Access Readiness - TODO

Status: Complete
Last updated: 2026-05-26

## M0 - Scope And Boundary Freeze

- [x] AWR-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-web-v2-users-access-readiness,CONTEXT.md,docs/adr]
  Goal: Confirm the current domain and ADR boundary for User, Role, Library
  Access, and Single-Admin Mode.
  Validation: `rg -n "User|Role|Library Access|Single-Admin|RBAC|ACL|access" CONTEXT.md docs/adr docs/workstreams crates/nako-api/src crates/nako-server/src apps/admin-web/src`
  Review: Do not open account CRUD or RBAC mutation scope unless existing
  backend authority is found.
  Evidence: `DESIGN.md`
  Handoff: DONE. Existing evidence shows inbound bearer auth and stable
  `local-admin` principal only; account, Role, and Library Access policy stores
  are follow-ons.

## M1 - Admin API Access Summary

- [x] AWR-020 [owner=codex] [deps=AWR-010] [scope=crates/nako-api/src,crates/nako-server/src/http/admin.rs,crates/nako-server/src/http/tests/system.rs,docs/api/HTTP_API.md]
  Goal: Add `GET /admin/v1/access/summary` with Single-Admin Mode principal,
  readiness, and effective Library Access for configured Media Libraries.
  Validation: `cargo nextest run -p nako-server admin_v1_access_summary --no-fail-fast`; `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture`
  Review: Response must be redaction-safe and must not imply accounts or RBAC
  are active.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. Admin API now exposes a redaction-safe access summary route,
  generated Admin Web contract route/types, and focused server coverage for
  Single-Admin Mode effective library access.

## M2 - Admin Web Route

- [x] AWR-030 [owner=codex] [deps=AWR-020] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/features/access,apps/admin-web/src/adminApi,apps/admin-web/src/styles.css]
  Goal: Add route-owned Users & Access page backed by `AdminDataSource`.
  Validation: `cd apps/admin-web && npm run test -- adminApi/client.test.ts adminApi/dataSource.test.ts App.test.tsx`; `cd apps/admin-web && npm run check`
  Review: UI must show live/mock source truth, current principal, readiness,
  per-library access, and no fake mutation buttons.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. `/access` is route-owned, appears in navigation, uses
  `AdminDataSource`, renders live/mock fallback, and deliberately omits account
  or RBAC mutation controls.

## M3 - Verification And Closeout

- [x] AWR-040 [owner=codex] [deps=AWR-030] [scope=apps/admin-web,crates/nako-api,crates/nako-server,docs/workstreams/admin-web-v2-users-access-readiness]
  Goal: Run focused and broad gates, browser smoke, update evidence, and close
  or split follow-ons.
  Validation: `cargo fmt --all --check`; focused Rust/API/Admin Web tests;
  `cd apps/admin-web && npm run generate:admin-api && npm run check && npm run build`; `git diff --check`; browser desktop/mobile smoke.
  Review: Record any blocked PostgreSQL or browser gates explicitly.
  Evidence: `EVIDENCE_AND_GATES.md`, `CLOSEOUT.md`
  Handoff: DONE. Focused and broad gates passed, browser fallback/live
  intercepted smoke was recorded, and account/RBAC/Library Access policy
  mutation remains split to a future backend-authority lane.
