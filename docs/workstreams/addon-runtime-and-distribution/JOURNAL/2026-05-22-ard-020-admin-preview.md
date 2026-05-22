# ARD-020 Admin Preview

Date: 2026-05-22

## Work Completed

- Added Admin DTOs for install-guide preview requests and responses.
- Added `AddonAppService::preview_addon_install_guide` so Admin HTTP callers use
  protocol validation before generating redacted guide summaries.
- Added `POST /admin/v1/addons/install-guide-preview`.
- Added HTTP tests for:
  - successful preview with safe runtime/manifest/Secret Reference facts;
  - no raw secret value, token string, local path, or credential material in
    response JSON;
  - rejected local runtime references without echoing the rejected path;
  - rejected raw Secret Reference values without echoing the rejected value;
  - rejected manifest paths without echoing the rejected path.

## Validation

- `cargo nextest run -p taru-addon-protocol --no-fail-fast`
- `cargo nextest run -p taru-server admin_addon_install_guide_preview --no-fail-fast`
- `cargo nextest run -p taru-server register_addon --no-fail-fast`
- `cargo nextest run -p taru-server addons --no-fail-fast`
- `cargo nextest run -p taru-api --no-fail-fast`
- `cargo fmt --all -- --check`
- `python -m json.tool docs/workstreams/addon-runtime-and-distribution/WORKSTREAM.json`
- `git diff --check`
- `git diff --name-only -- crates/taru-client-protocol`

## Next

ARD-030: runtime readiness diagnostics for Addon Sidecars.
