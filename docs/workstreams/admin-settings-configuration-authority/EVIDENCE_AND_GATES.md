# Admin Settings Configuration Authority - Evidence And Gates

Status: Closed
Last updated: 2026-05-26

## Smallest Current Repro

```powershell
rg -n "NakoServerConfig|load_config|NetworkBoundaryState|Semaphore::new|settings|config" crates/nako-server/src crates/nako-db/src crates/nako-core/src
git diff --check
```

This proves whether a global settings authority already exists and which
runtime resources consume startup config.

## Gate Set

### Design Gate

```powershell
rg -n "NakoServerConfig|load_config|NetworkBoundaryState|Semaphore::new|settings|config" crates/nako-server/src crates/nako-db/src crates/nako-core/src
git diff --check
```

### Backend Gate

```powershell
cargo nextest run -p nako-server <admin-settings-filter> --no-fail-fast
cargo nextest run -p nako-db <settings-contract-filter> --no-fail-fast
cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture
cargo fmt --all --check
```

Use the database gate only if ASCA-030 adds persistence.

### Docs And Generated Contract Gate

```powershell
cd apps/admin-web
npm run generate:admin-api
git diff --check
```

## Evidence Log

| Date | Task | Command or artifact | Result |
| --- | --- | --- | --- |
| 2026-05-25 | ASCA-010 | `docs/workstreams/admin-settings-configuration-authority/*` | Opened backend configuration-authority lane from settings mutation readiness evidence. |
| 2026-05-26 | ASCA-020 | `docs/workstreams/admin-settings-configuration-authority/DESIGN.md` | Pass. Selected metadata raw cache retention as the first safe global settings field group and documented persisted Admin override, startup merge, restart-required, and redaction semantics. |
| 2026-05-26 | ASCA-030 | `cargo nextest run -p nako-server admin_metadata_raw_cache_settings --no-fail-fast` | Pass. Startup behavior applies persisted Admin metadata raw cache settings across restart. |
| 2026-05-26 | ASCA-030 | `cargo nextest run -p nako-server admin_v1_metadata_raw_cache_settings --no-fail-fast` | Pass. Admin HTTP route round-trips GET/PUT, reports `requires_restart` before restart, reports `active` after restart, and rejects zero retention. |
| 2026-05-26 | ASCA-030 | `cargo nextest run -p nako-db sqlite_admin_settings_contract_metadata_raw_cache_settings_round_trip --no-fail-fast` | Pass. SQLite settings repository round-trips and replaces the single persisted Admin override record. |
| 2026-05-26 | ASCA-030 | `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture` | Pass. Admin route/type contract includes `settingsMetadataRawCache` and the metadata raw cache settings DTOs. |
| 2026-05-26 | ASCA-030 | `cd apps/admin-web && npm run generate:admin-api` | Pass. Regenerated app-local Admin Web contract from `nako-api`. |
| 2026-05-26 | ASCA-040 | `cargo test -p nako-api admin_web_generated_contract_matches_generator_output -- --nocapture` | Pass. App-local generated Admin Web contract matches the Rust generator output. |
| 2026-05-26 | ASCA-040 | `cargo test -p nako-api admin_contract_routes_stay_out_of_public_client_inventory -- --nocapture` | Pass. New Admin settings route stays out of Public Client route inventory. |
| 2026-05-26 | ASCA-040 | `cargo fmt --all --check` | Pass. Rust workspace formatting is clean. |
| 2026-05-26 | ASCA-040 | `git diff --check` | Pass with line-ending warnings only. No whitespace errors. |
| 2026-05-26 | ASCA-040 | `python -m json.tool docs\workstreams\admin-settings-configuration-authority\WORKSTREAM.json` | Pass. Workstream metadata JSON is valid. |
| 2026-05-26 | ASCA-040 | `if (Test-Path Env:NAKO_TEST_POSTGRES_URL) { ... }` | Skipped. `NAKO_TEST_POSTGRES_URL` is missing, so the ignored PostgreSQL settings contract was not run locally. |

## Evidence Anchors

- `docs/workstreams/admin-settings-configuration-authority/DESIGN.md`
- `docs/workstreams/admin-settings-configuration-authority/TODO.md`
- `crates/nako-server/src/config.rs`
- `crates/nako-server/src/app/composition.rs`
- `crates/nako-server/src/http.rs`
- `crates/nako-server/src/http/network.rs`
- `crates/nako-db/`

## Review Notes

ASCA-040 review:

- Workstream compliance: no blocking findings. ASCA-020/030 satisfy the
  selected field group, source-of-truth, restart-required, redaction, route,
  generated contract, and evidence requirements.
- Code quality: no blocking findings. Implementation follows crate boundaries:
  domain records and repository trait in `nako-core`, migrations/adapters in
  `nako-db`, DTO/contract in `nako-api`, startup merge and HTTP boundary in
  `nako-server`.
- Missing gates: PostgreSQL contract is present as an ignored paired contract
  but was not run because `NAKO_TEST_POSTGRES_URL` is not configured in this
  local environment.
- Residual risk: only metadata raw cache settings are editable through this
  authority. Admin Web must not expose save controls for broader global
  settings until their backend authority is implemented.
