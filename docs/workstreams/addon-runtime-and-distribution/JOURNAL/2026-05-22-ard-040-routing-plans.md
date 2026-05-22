# ARD-040 Declared Routing Plans

Date: 2026-05-22

## Work Completed

- Added durable Addon routing-plan domain records and SQLite/PostgreSQL
  persistence for manifest-declared Addon Tasks and Event Subscriptions.
- Added `JobKind::AddonTask` as the safe Taru-owned execution target label for
  future Addon Task jobs without introducing a scheduler.
- Added `AddonAppService::sync_addon_routing_plans` and
  `POST /admin/v1/addons/{addon_id}/routing-plans`.
- Synced generated Admin TypeScript contract and `AdminApiClient` support.
- Added HTTP and DB contract tests for:
  - executable task/event routing plans;
  - disabled-addon, missing-grant, and unsupported-event deferral;
  - idempotent manifest-declaration replacement and stale-plan removal;
  - no hidden job creation or outbox delivery side effects;
  - redacted plan diagnostics without manifest filter/token echo.

## Validation

- `cargo check -p taru-api -p taru-server --tests`
- `cargo nextest run -p taru-db addon --no-fail-fast`
- `cargo nextest run -p taru-server admin_addon_routing_plans --no-fail-fast`
- `cargo nextest run -p taru-server addons --no-fail-fast`
- `cargo nextest run -p taru-api admin_contract --no-fail-fast`
- `npm run check`
- `npm test -- src/adminApi/client.test.ts`
- `cargo fmt --all -- --check`
- `git diff --name-only -- crates/taru-client-protocol`

## Next

ARD-050: route Addon-produced Generated Artifacts and acquisition candidates
into existing AILO proposal/review and DWI intake boundaries.
