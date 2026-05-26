# Admin Settings Configuration Authority - TODO

Status: Closed
Last updated: 2026-05-26

## M0 - Scope And Evidence Freeze

- [x] ASCA-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-settings-configuration-authority]
  Goal: Open backend configuration-authority lane from ASM-020 evidence.
  Validation: `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, and `WORKSTREAM.json` agree.
  Evidence: `docs/workstreams/admin-settings-configuration-authority/DESIGN.md`
  Handoff: DONE. First executable task is ASCA-020.

## M1 - First Field Group And Authority Model

- [x] ASCA-020 [owner=codex] [deps=ASCA-010] [scope=crates/nako-server/src/config.rs,crates/nako-server/src/app,crates/nako-server/src/http,crates/nako-db,crates/nako-core,docs/workstreams/admin-settings-configuration-authority]
  Goal: Choose the first global settings field group and freeze its
  source-of-truth semantics, startup merge rules, redaction rules, and route
  shape.
  Validation: `rg -n "NakoServerConfig|load_config|NetworkBoundaryState|Semaphore::new|settings|config" crates/nako-server/src crates/nako-db/src crates/nako-core/src`; `git diff --check`
  Review: The result must explicitly state whether values are persisted,
  runtime-only, or restart-required.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: DONE. First accepted field group is metadata raw cache retention:
  `metadata.raw_cache_retention_ms` and
  `metadata.maintenance.raw_cache_cleanup_on_startup`. Admin values are
  persisted as desired-state overrides, TOML is merged first at startup, Admin
  overrides win on startup, PUT reports `requires_restart` until the process is
  restarted, and no raw config, paths, URLs, roots, hosts, env vars, tokens, or
  credentials are accepted or returned.

## M2 - Backend Implementation

- [x] ASCA-030 [owner=codex] [deps=ASCA-020] [scope=crates/nako-core,crates/nako-db,crates/nako-server,crates/nako-api,docs/api/HTTP_API.md]
  Goal: Implement the accepted persistence/runtime model and Admin API
  route(s) for the first field group.
  Validation: `cargo nextest run -p nako-server <admin-settings-filter> --no-fail-fast`; `cargo nextest run -p nako-db <settings-contract-filter> --no-fail-fast`; `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture`
  Review: No Public Client API changes, no raw config leaks, no secrets, no raw
  paths/URLs/roots.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE_WITH_CONCERNS. Backend route exists at
  `GET|PUT /admin/v1/settings/metadata/raw-cache`, generated Admin contract is
  refreshed, and focused server/DB/API gates pass. Admin Web controls are
  unblocked only for this field group and must display the restart-required
  effect; broader settings remain blocked until their own backend authority is
  implemented.

## M3 - Closeout

- [x] ASCA-040 [owner=codex] [deps=ASCA-030] [scope=docs/workstreams/admin-settings-configuration-authority]
  Goal: Verify, review, close, and hand off to
  `admin-web-v2-settings-mutation-authority`.
  Validation: focused Rust gates, generated Admin contract gate, `cargo fmt --all --check`, `git diff --check`.
  Review: Run `review-workstream` and `verify-rust-workstream`.
  Evidence: `CLOSEOUT.md`, `WORKSTREAM.json`
  Handoff: DONE_WITH_CONCERNS. Backend lane is closed and handed back to
  `admin-web-v2-settings-mutation-authority`; Admin Web may continue with UI
  controls only for metadata raw cache settings. PostgreSQL parity code and
  ignored contract test exist, but the local verification environment did not
  provide `NAKO_TEST_POSTGRES_URL`, so the PostgreSQL contract gate is recorded
  as skipped.
