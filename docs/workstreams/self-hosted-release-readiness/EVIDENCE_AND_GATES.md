# Self-Hosted Release Readiness — Evidence And Gates

Status: Completed
Last updated: 2026-06-07

## Baseline Gates

Initial expected closeout gate family:

```bash
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
cargo nextest run -p nako-db sqlite_managed_artwork_contract --no-fail-fast
NAKO_TEST_POSTGRES_URL=<url> cargo nextest run -p nako-db postgres_managed_artwork_contract --run-ignored ignored-only --no-fail-fast
cargo nextest run -p nako-api managed_artwork --no-fail-fast
cargo nextest run -p nako-server managed_artwork --no-fail-fast
git diff --check
```

This list will be refined by SHR-020/030/040 into a scriptable release gate.

SHR-020 entrypoints:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode docs
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode fast
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode postgres
```

```bash
scripts/release-gate.sh --mode docs
scripts/release-gate.sh --mode fast
scripts/release-gate.sh --mode postgres
```

Both scripts support `docs`, `fast`, `db`, `api`, `postgres`, `workspace`,
and `all` modes. PostgreSQL execution is routed through the SHR-030 harness.

SHR-030 PostgreSQL harness:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite managed-artwork
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite storage-source-parity
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite job-runtime
```

```bash
bash scripts/postgres-contract-harness.sh --suite managed-artwork
bash scripts/postgres-contract-harness.sh --suite all-contracts
bash scripts/postgres-contract-harness.sh --suite storage-source-parity
bash scripts/postgres-contract-harness.sh --suite job-runtime
```

Without `NAKO_TEST_POSTGRES_URL`, the harness starts a temporary local cluster
under `target/postgres-contract/` when `initdb`, `pg_ctl`, and `createdb` are
available. If tooling is unavailable, it prints a clear skip and exits
successfully unless `--require-tooling` / `-RequireTooling` is set.

`storage-source-parity` is the combined M2 storage-VFS reliability slice. It
reuses the existing `storage-runtime` and `source-identity` contract filters in
sequence under one harness entry, while the narrower suites remain available
when you want a smaller focus area.

`job-runtime` is the focused durable job runtime slice. It runs the existing
PostgreSQL job lease and retry contracts for claim filters, run-token fencing,
cancellation acknowledgement, expired lease recovery, redacted queue pressure,
priority ordering, and retry preservation without widening to `all-contracts`.

## Redaction Inventory Gate

```bash
rg -n "storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|artifact_root|local_path|database_url|token|secret" crates docs
```

The inventory must distinguish safe docs/tests/internal records from public or
admin response leaks.

`scripts/release-gate.*` writes the inventory to
`target/release-gate/redaction-inventory.txt`.

## API/SDK/Redaction Gate

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode api
```

```bash
bash scripts/release-gate.sh --mode api
```

This gate composes:

- `cargo check -p nako-api --tests`
- `cargo check -p nako-client --tests`
- `cargo check -p nako-client-protocol --tests`
- `cargo nextest run -p nako-api openapi --no-fail-fast`
- `cargo nextest run -p nako-api sdk --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-client --no-fail-fast`
- `cargo nextest run -p nako-client-protocol --no-fail-fast`
- `cargo tree -p nako-client`
- `cargo tree -p nako-client-protocol`
- `npm run generate --prefix sdk/typescript`
- `npm run check --prefix sdk/typescript`
- `npm run generate:admin-api --prefix apps/admin-web`
- `npm run check --prefix apps/admin-web`
- `cargo nextest run -p nako-server self_host_smoke --no-fail-fast`
- `git diff --check`

## Self-Host Smoke Gate

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/self-host-smoke.ps1 -Backend sqlite
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/self-host-smoke.ps1 -Backend postgres
```

```bash
bash scripts/self-host-smoke.sh --backend sqlite
bash scripts/self-host-smoke.sh --backend postgres
```

The SQLite path runs the public/Admin HTTP self-host smoke in `nako-server`.
The PostgreSQL path delegates to `scripts/postgres-contract-harness.* --suite
managed-artwork`, which proves the release-critical PostgreSQL artifact and
ingest contracts without inspecting private SQL from the HTTP smoke.

## Evidence Log

| Date | Scope | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | SHR-010 baseline | `git status --short --branch` | Pass. Worktree was clean on `main` before opening the workstream. |
| 2026-05-21 | SHR-010 baseline | Root release entrypoint inventory for `.github`, `scripts`, `ci`, `.cargo`, `.config`, and `nextest.toml` | Pass. None exists at repo root; release gate orchestration is missing and belongs in this lane. |
| 2026-05-21 | SHR-010 baseline | `Cargo.toml` workspace inspection | Pass. Workspace uses resolver `3`, edition `2024`, Rust `1.85`, and `members = ["crates/*"]`. |
| 2026-05-21 | SHR-020 local release gate | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode docs` | Pass. Ran format/diff gates and wrote 3148 redaction inventory matches to `target/release-gate/redaction-inventory.txt`. |
| 2026-05-21 | SHR-020 local release gate | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode fast` | Pass. Format/diff gates passed; redaction inventory wrote 3148 matches; `nako-db` check passed; SQLite Managed Artwork contracts passed 6/6; `nako-api` and `nako-server` checks passed; `nako-api managed_artwork` passed 12/12; `nako-server managed_artwork` passed 13/13. |
| 2026-05-21 | SHR-020 local release gate | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode postgres -SkipRedactionInventory` | Pass. Format/diff gates passed and PostgreSQL contracts were safely skipped because no `NAKO_TEST_POSTGRES_URL` or `-PostgresUrl` was provided. |
| 2026-05-21 | SHR-020 local release gate | `cargo fmt --all -- --check`; `git diff --check` | Pass. |
| 2026-05-21 | SHR-020 local release gate | `cargo nextest show-config version` | Pass. `.config/nextest.toml` is loadable by installed `cargo-nextest 0.9.116`; no repository version requirement is currently set. |
| 2026-05-21 | SHR-030 PostgreSQL harness | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite managed-artwork` | Pass. Local PostgreSQL 17 tooling started an isolated cluster under `target/postgres-contract`, ran PostgreSQL Managed Artwork contracts 6/6, stopped PostgreSQL, and removed the harness data directory. |
| 2026-05-21 | SHR-030 PostgreSQL harness | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode postgres -SkipRedactionInventory` | Pass. Release gate delegated to the PostgreSQL harness and PostgreSQL Managed Artwork contracts passed 6/6. |
| 2026-05-21 | SHR-030 PostgreSQL harness | Simulated missing local tooling by running the harness with `PATH=C:\Windows\System32;C:\Windows` and no `NAKO_TEST_POSTGRES_URL` | Pass. Harness printed a clear missing-tooling skip for `initdb`, `pg_ctl`, and `createdb` and exited 0. |
| 2026-05-21 | SHR-030 PostgreSQL harness | `Test-Path target/postgres-contract` after harness runs | Pass. Returned `False`; temporary harness data was cleaned. |
| 2026-05-21 | SHR-030 PostgreSQL harness | `cargo fmt --all -- --check`; `git diff --check` | Pass. |
| 2026-05-21 | SHR-040 API/SDK/redaction gate | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode api` | Pass. Redaction inventory wrote 3148 matches. OpenAPI tests passed 6/6; SDK tests passed 6/6; Admin contract tests passed 5/5; `nako-client` tests passed 9/9; `nako-client-protocol` tests passed 8/8; TypeScript SDK generate/check passed; Admin Web contract generate/check passed; Managed Artwork API/server redaction tests passed 12/12 and 13/13. |
| 2026-05-21 | SHR-040 API/SDK/redaction gate | `git diff -- sdk/typescript/src/index.ts apps/admin-web/src/adminApi/generated/contract.ts` | Pass. Generator runs left tracked generated files unchanged. |
| 2026-05-21 | SHR-040 API/SDK/redaction gate | `cargo fmt --all -- --check`; `git diff --check` | Pass. |
| 2026-05-21 | SHR-050 deployment examples | Python `tomllib` parse for `deploy/sqlite/nako.toml` and `deploy/postgres/nako.toml` | Pass. Both TOML files parsed; SQLite backend had one library and PostgreSQL backend had one library. |
| 2026-05-21 | SHR-050 deployment examples | `docker compose -f deploy/compose/postgres.yml config` with no `NAKO_POSTGRES_PASSWORD` | Pass. Failed safely with required-variable error instead of silently using an unsafe default password. |
| 2026-05-21 | SHR-050 deployment examples | `NAKO_POSTGRES_PASSWORD=example-not-a-secret docker compose -f deploy/compose/postgres.yml config` | Pass. Compose rendered a PostgreSQL 17 service bound to `127.0.0.1:5432` with a named data volume. |
| 2026-05-21 | SHR-050 deployment examples | `rg -n "database_url\|artifact_root\|staging\|auth\|Addon\|Webhook\|Playback Runtime\|diagnostics\|SQLite\|PostgreSQL" docs/deployment/SELF_HOSTED.md deploy/sqlite/nako.toml deploy/postgres/nako.toml deploy/compose/postgres.yml` | Pass. Required operator topics are present. |
| 2026-05-21 | SHR-050 deployment examples | `cargo fmt --all -- --check`; `git diff --check` | Pass. |
| 2026-05-21 | SHR-060 backup/restore/upgrade | `rg -n "SQLite\|PostgreSQL\|artifact root\|NFO sidecars\|staging\|cache\|secrets\|migration\|rollback\|forward\|durable state\|rebuildable" docs/deployment/BACKUP_RESTORE_UPGRADE.md` | Pass. Required state-boundary, backend, and migration topics are present. |
| 2026-05-21 | SHR-060 backup/restore/upgrade | `cargo fmt --all -- --check`; `git diff --check` | Pass. |
| 2026-05-21 | SHR-070 self-host smoke | `cargo nextest run -p nako-server self_host_smoke --no-fail-fast` | Pass. SQLite HTTP operator smoke passed 1/1, covering health, scan enqueue, NFO export enqueue, metadata refresh enqueue, Addon artwork proposal, Managed Artwork ingest/publish, public image serving, playback decision/range streaming, and diagnostic redaction. |
| 2026-05-21 | SHR-070 self-host smoke | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/self-host-smoke.ps1 -Backend sqlite` | Pass. Scripted SQLite self-host smoke passed 1/1. |
| 2026-05-21 | SHR-070 self-host smoke | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/self-host-smoke.ps1 -Backend postgres` | Pass. Delegated to the PostgreSQL Managed Artwork harness; local PostgreSQL 17 cluster ran 6/6 contracts and cleaned `target/postgres-contract`. |
| 2026-05-21 | SHR-070 self-host smoke | `cargo fmt --all -- --check`; `git diff --check` | Pass. |
| 2026-05-21 | SHR-080 closeout | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode docs` | Pass. Format/diff gates passed and redaction inventory wrote 3208 matches to `target/release-gate/redaction-inventory.txt`. |
| 2026-05-21 | SHR-080 closeout | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode fast` | Pass. Format/diff gates passed; redaction inventory wrote 3208 matches; SQLite Managed Artwork contracts passed 6/6; API/OpenAPI/Admin contract/client/protocol/TypeScript/Admin Web gates passed; Managed Artwork API tests passed 12/12; Managed Artwork server tests passed 13/13; SQLite self-host smoke passed 1/1. |
| 2026-05-21 | SHR-080 closeout | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode postgres -SkipRedactionInventory` | Pass. PostgreSQL Managed Artwork contracts passed 6/6 through the local PostgreSQL 17 harness. |
| 2026-05-21 | SHR-080 closeout | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode workspace -SkipRedactionInventory` | Pass. `cargo check --workspace --tests` passed; `cargo nextest run --workspace --no-fail-fast` passed 506/506 with 25 skipped. |
| 2026-06-07 | SHR-030 PostgreSQL harness | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite storage-source-parity -Port 55434` | Pass. Local PostgreSQL 17 tooling started an isolated cluster under `target/postgres-contract`, ran the storage-runtime suite 4/4 and the source-identity suite 6/6 in sequence, stopped PostgreSQL, and removed the harness data directory. |
| 2026-06-07 | SHR-030 PostgreSQL harness | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite job-runtime -Port 55436` | Pass. Local PostgreSQL 17 tooling started an isolated cluster under `target/postgres-contract`, ran the durable job runtime suite 6/6, stopped PostgreSQL within the 90s cleanup window, and removed the harness data directory. |

## Follow-On Gaps

- Linux/Git Bash execution evidence for shell scripts is still pending in CI or
  a Linux shell; this Windows environment only had the WSL `bash.exe` shim.
- Future lanes should cover AI product features, network traversal/tunneling,
  Native Plugin ABI, provider breadth, and release packaging once this
  self-hosted baseline is committed.
