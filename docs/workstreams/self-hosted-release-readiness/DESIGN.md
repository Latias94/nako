# Self-Hosted Release Readiness Design

Status: Completed
Last updated: 2026-05-21

## Problem

Taru has strong subsystem architecture but does not yet have a single release
readiness lane that proves the self-hosted operator story end to end.

Today, several critical gates are implicit or manual:

- PostgreSQL contract tests require ad-hoc local environment setup.
- The repo root has no `.github/`, `scripts/`, `ci/`, `.cargo/`, or
  `nextest.toml` release gate entrypoint.
- SQLite and PostgreSQL behavior are proven by focused workstreams, but not yet
  assembled into one self-hosted release baseline.
- API/SDK/redaction checks exist in packages and docs, but are not yet composed
  as an operator-facing release gate.
- Deployment, upgrade, backup/restore, and end-to-end self-host smoke evidence
  need one durable owner.

## Target State

- A documented and scriptable local release gate exists for self-hosted Taru.
- SQLite and PostgreSQL backend gates are explicit and repeatable.
- PostgreSQL contract execution has a repo-owned local harness and a CI-ready
  path.
- Public/Admin API, SDK generation, and redaction inventories are checked as
  release gates.
- Self-hosted deployment examples cover SQLite and PostgreSQL.
- Backup, restore, migration, and artifact-root consistency guidance exists.
- A self-host smoke path proves library scan, metadata/NFO, Addon artwork
  proposal, Managed Artwork ingest/public serving, playback, and diagnostics
  without leaking secrets or local paths.

## In Scope

- Release gate scripts and documentation.
- nextest profile and/or command composition for local and CI use.
- PostgreSQL test orchestration for contract tests.
- Docker or compose examples for self-hosted SQLite/PostgreSQL operation.
- Backup/restore and upgrade documentation for DB plus local artifact/cache
  state.
- API/SDK/redaction synchronization gates.
- End-to-end self-host smoke fixture or command path.

## Out Of Scope

- New AI product features.
- New network traversal/tunneling runtime.
- Native Plugin ABI design.
- New metadata provider breadth.
- Adaptive bitrate ladder or distributed transcode queue.
- Changing the existing Addon security model.

## Architecture Direction

- Treat release readiness as composition, not a new product feature.
- Prefer explicit scripts and docs over hidden local assumptions.
- Keep PostgreSQL tests opt-in when no database is available, but provide a
  first-party harness that can start a temporary local PostgreSQL when tooling
  exists.
- Release gates must be redaction-aware: passing tests are insufficient if
  operator diagnostics or generated contracts expose secrets, locators, local
  paths, raw source URLs, or storage handles.
- Keep artifact bytes outside PostgreSQL. Backup/restore docs must mention DB
  records and local artifact roots as separate but related state.

## Local Release Gate Shape

SHR-020 adds the first repo-owned local gate entrypoints:

- `scripts/release-gate.ps1` is the Windows/PowerShell-first entrypoint.
- `scripts/release-gate.sh` is the Linux/macOS/CI shell entrypoint.
- `.config/nextest.toml` sets the shared default nextest output/fail-fast
  posture.

Gate modes:

- `docs`: formatting, `git diff --check`, and redaction inventory only.
- `fast`: the default focused release confidence path. It includes formatting,
  `git diff --check`, redaction inventory, focused `taru-db` checks, SQLite
  Managed Artwork contracts, and focused `taru-api`/`taru-server` Managed
  Artwork redaction/API tests.
- `db`: database-only focused checks plus SQLite Managed Artwork contracts.
- `api`: API/server focused checks and Managed Artwork API/server tests.
- `postgres`: optional PostgreSQL Managed Artwork contract execution when
  `TARU_TEST_POSTGRES_URL` or an explicit URL is provided; otherwise it
  reports a safe skip. SHR-030 owns the stronger local harness and CI shape.
- `workspace`: full workspace check and full workspace nextest run.
- `all`: combines all available modes and keeps PostgreSQL optional unless a
  URL is present.

The scripts intentionally write redaction inventory output under
`target/release-gate/` so expected inventory matches do not flood normal
terminal output or become source-controlled artifacts.

## PostgreSQL Contract Harness

SHR-030 adds first-party PostgreSQL harness scripts:

- `scripts/postgres-contract-harness.ps1`
- `scripts/postgres-contract-harness.sh`

The harness supports two modes of operation:

1. If `TARU_TEST_POSTGRES_URL` or an explicit database URL is provided, run the
   ignored PostgreSQL contract tests against that database.
2. If no URL is provided and `initdb`, `pg_ctl`, and `createdb` are available,
   start an isolated temporary cluster under `target/postgres-contract/`, run
   the selected contracts, stop the server, and remove the temporary data
   unless `--keep-data` / `-KeepData` is requested.

If neither a URL nor local PostgreSQL tooling is available, the harness reports
a clear skip and exits successfully for local developer ergonomics. CI can pass
`--require-tooling` / `-RequireTooling` to convert missing tooling into a hard
failure.

Harness suites:

- `managed-artwork`: the release-critical PostgreSQL Managed Artwork contract
  subset.
- `all-contracts`: every ignored `postgres_` contract test in `taru-db`.

The harness never removes paths outside `target/postgres-contract/`; the
PowerShell implementation verifies the resolved cleanup target before recursive
delete, and the shell implementation verifies it is inside the resolved
`target/` directory.

## API, SDK, And Redaction Gate Shape

SHR-040 extends `scripts/release-gate.* --mode api` and the `fast` gate with
API/SDK/redaction checks:

- `taru-api`, `taru-client`, and `taru-client-protocol` compile checks.
- Public OpenAPI tests.
- Public TypeScript SDK generator/package synchronization tests.
- Admin Web TypeScript contract generator/package synchronization tests.
- Public Rust client SDK tests.
- Public protocol DTO/route inventory tests.
- `cargo tree` inventory for `taru-client` and `taru-client-protocol` so the
  permissive public boundary remains visible in release evidence.
- `npm run generate --prefix sdk/typescript` plus strict TypeScript compile.
- `npm run generate:admin-api --prefix apps/admin-web` plus Admin Web
  TypeScript compile.
- redaction inventory output under `target/release-gate/`.

This gate intentionally keeps Public Client protocol, Admin API contract, and
server-internal DTOs separate. Public SDK/OpenAPI generation is allowed to
consume public protocol facts; Admin Web generation consumes the admin contract
only; neither path should expose server storage handles, local paths, source
URLs, database URLs, raw secrets, or bearer token values.

The `fast` and `api` gate modes also include the SQLite self-host smoke test so
release confidence proves the composed operator flow, not only isolated DTO and
Managed Artwork contract boundaries.

## Self-Hosted Deployment Examples

SHR-050 adds operator-facing deployment examples:

- `deploy/sqlite/taru.toml`
- `deploy/postgres/taru.toml`
- `deploy/compose/postgres.yml`
- `docs/deployment/SELF_HOSTED.md`

The examples bind Taru to `127.0.0.1` by default, keep inbound auth enabled,
use environment variable names for secrets, and separate durable state from
cache/rebuildable state. They intentionally do not provide a public Taru
container image yet; the current release baseline remains source-built with a
PostgreSQL service example for operators who want that backend locally.

## Backup, Restore, And Upgrade Runbook

SHR-060 adds `docs/deployment/BACKUP_RESTORE_UPGRADE.md`. The runbook treats
database state, Managed Artwork artifacts, media/NFO sidecars, config, and
secrets as durable state. Remux/HLS outputs, remote staging inputs, raw-cache
rows, and developer `target/` evidence are cache or rebuildable state.

The runbook's migration stance is forward-only unless release notes say
otherwise: rollback means restoring the pre-upgrade database, artifact root,
config, and matching binary rather than running an older binary against a newer
migrated schema.

## End-To-End Self-Host Smoke

SHR-070 adds a composed smoke artifact:

- `crates/taru-server/src/http/tests/self_host_smoke.rs`
- `scripts/self-host-smoke.ps1`
- `scripts/self-host-smoke.sh`

The SQLite smoke exercises one realistic operator flow through HTTP boundaries:

1. health and configured library visibility,
2. library scan job enqueue,
3. NFO export job enqueue,
4. metadata refresh job enqueue,
5. Addon artwork proposal and admin acceptance,
6. Managed Artwork ingest processing and artifact publication,
7. public item image listing and public image serving,
8. playback decision and byte-range stream serving,
9. Admin overview/system diagnostics with secret, locator, storage handle, raw
   source URL, raw token, content hash, database URL, and temp-path redaction
   assertions.

The PostgreSQL smoke path deliberately reuses the PostgreSQL Managed Artwork
contract harness rather than duplicating private SQL inspection in the server
test. That gives the release baseline both a public/Admin HTTP SQLite flow and
a storage-adapter PostgreSQL parity gate for the most stateful self-hosted
artifact path.

## M0 Baseline Inventory

Initial inspection on 2026-05-21 found:

- `git status --short --branch` is clean on `main`.
- No repo-root `.github/`, `scripts/`, `ci/`, `.cargo/`, `.config`, or
  `nextest.toml` exists.
- `Cargo.toml` uses a Rust workspace with `resolver = "3"` and `members =
  ["crates/*"]`.
- Previous lanes closed PostgreSQL production readiness and Managed Artwork
  PostgreSQL parity, but the release gate that composes them remains missing.
