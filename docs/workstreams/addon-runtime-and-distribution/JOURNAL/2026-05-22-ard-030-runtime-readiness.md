# ARD-030 Runtime Readiness

Date: 2026-05-22

## Work Completed

- Added Admin runtime readiness DTOs for Addon Sidecar diagnostics.
- Added `AddonAppService::check_addon_runtime_readiness`.
- Added `POST /admin/v1/addons/{addon_id}/runtime-readiness`.
- Synced generated Admin TypeScript contract and `AdminApiClient` support.
- Added HTTP tests for:
  - ready sidecar readiness without token/payload echo;
  - degraded sidecar status preservation;
  - missing grants and missing Secret Reference configuration without sidecar
    calls;
  - network policy blockers without URL or credential echo;
  - protocol mismatch, manifest mismatch, and unsafe sidecar response
    classification without raw payload echo.

## Validation

- `cargo check -p taru-api -p taru-server --tests`
- `cargo nextest run -p taru-server admin_addon_runtime_readiness --no-fail-fast`
- `cargo nextest run -p taru-api admin_contract --no-fail-fast`
- `cargo nextest run -p taru-server addons --no-fail-fast`
- `npm run check`
- `npm test -- src/adminApi/client.test.ts`
- `cargo fmt --all -- --check`
- `python -m json.tool docs/workstreams/addon-runtime-and-distribution/WORKSTREAM.json`
- `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`
- `git diff --check`
- `git diff --name-only -- crates/taru-client-protocol`

## Next

ARD-040: route declared Addon Tasks and Event Subscriptions into explicit
Taru-owned plans without hidden schedulers.
