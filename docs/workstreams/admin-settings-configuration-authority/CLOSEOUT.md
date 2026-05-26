# Admin Settings Configuration Authority - Closeout

Status: DONE_WITH_CONCERNS
Date: 2026-05-26

## Result

This lane is closed. Nako now has a backend-owned Admin settings authority for
metadata raw cache retention:

- persisted Admin desired-state storage;
- startup merge after TOML and before service construction;
- redaction-safe `GET|PUT /admin/v1/settings/metadata/raw-cache`;
- generated Admin Web contract DTOs and route constant;
- HTTP API documentation;
- focused server, database, API contract, formatting, and whitespace gates.

## Handoff

Continue in `admin-web-v2-settings-mutation-authority` ASM-030/ASM-040.

Admin Web may expose save controls only for:

- `metadata.raw_cache_retention_ms`
- `metadata.maintenance.raw_cache_cleanup_on_startup`

The UI must surface that PUT persists an override but does not hot-apply the
current process when the route reports `effect = requires_restart`.

## Verification

- `cargo nextest run -p nako-server admin_metadata_raw_cache_settings --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_metadata_raw_cache_settings --no-fail-fast`
- `cargo nextest run -p nako-db sqlite_admin_settings_contract_metadata_raw_cache_settings_round_trip --no-fail-fast`
- `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture`
- `cargo test -p nako-api admin_web_generated_contract_matches_generator_output -- --nocapture`
- `cargo test -p nako-api admin_contract_routes_stay_out_of_public_client_inventory -- --nocapture`
- `cd apps/admin-web && npm run generate:admin-api`
- `cargo fmt --all --check`
- `git diff --check`
- `python -m json.tool docs\workstreams\admin-settings-configuration-authority\WORKSTREAM.json`

## Concern

PostgreSQL adapter and ignored paired contract test were added, but this local
environment did not define `NAKO_TEST_POSTGRES_URL`, so PostgreSQL runtime
contract execution remains a skipped local gate.
