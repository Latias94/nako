# Self-Hosted Release Readiness — TODO

Status: Completed
Last updated: 2026-05-21

Task IDs use the `SHR` prefix.

## M0 — Baseline And Release-Gate Scope

- [x] SHR-010 [owner=planner] [deps=none] [scope=docs/workstreams/self-hosted-release-readiness,docs/workstreams/README.md]
  Goal: Open the durable self-hosted release readiness lane, freeze the current
  repo/CI/gate baseline, and define the release readiness task sequence.
  Validation: `git status --short --branch`; root release-entrypoint inventory;
  `git diff --check`.
  Review: Do not mark the larger goal complete from planning docs alone.
  Evidence: M0 inventory in `DESIGN.md` and `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with SHR-020 local release gate entrypoint.

## M1 — Local Release Gate Entrypoint

- [x] SHR-020 [owner=codex] [deps=SHR-010] [scope=scripts,nextest.toml,docs/workstreams/self-hosted-release-readiness]
  Goal: Add a repo-owned local release gate entrypoint that composes formatting,
  focused package checks, SQLite tests, API/SDK/redaction checks, and optional
  PostgreSQL gates without relying on chat-only command recipes.
  Validation: run the new local gate in at least fast/doc-safe mode; `cargo fmt
  --all -- --check`; `git diff --check`.
  Review: The entrypoint must be safe on Windows PowerShell and CI Linux; it
  must not delete user data or assume Docker is available.
  Evidence: `scripts/release-gate.ps1`, `scripts/release-gate.sh`,
  `.config/nextest.toml`; `scripts/release-gate.ps1 -Mode fast` passed;
  PostgreSQL mode gracefully skipped without `TARU_TEST_POSTGRES_URL`.
  Handoff: Continue with PostgreSQL contract harness.

## M2 — PostgreSQL Contract Harness And CI Shape

- [x] SHR-030 [owner=codex] [deps=SHR-020] [scope=scripts,.github,docs/workstreams/self-hosted-release-readiness,crates/taru-db]
  Goal: Provide a repeatable local PostgreSQL contract harness and CI-ready
  job shape for ignored PostgreSQL backend contracts.
  Validation: run `postgres_*_contract` nextest cases through the harness when
  local PostgreSQL tooling is available; otherwise prove graceful skip with a
  clear message; `git diff --check`.
  Review: Temporary PostgreSQL data directories must be created under `target/`
  and cleaned up safely.
  Evidence: `scripts/postgres-contract-harness.ps1`,
  `scripts/postgres-contract-harness.sh`, `.github/workflows/release-gate.yml`;
  local PowerShell harness ran PostgreSQL Managed Artwork contracts 6/6 and
  cleaned `target/postgres-contract`; missing-tooling simulation skipped with a
  clear message and exit 0.
  Handoff: Continue with API/SDK/redaction release gates.

## M3 — API, SDK, And Redaction Gates

- [x] SHR-040 [owner=codex] [deps=SHR-020] [scope=crates/taru-api,crates/taru-client,taru-client-protocol,sdk,docs/api,docs/workstreams/self-hosted-release-readiness]
  Goal: Compose Admin/Public API, generated SDK, OpenAPI, and redaction checks
  into a release gate.
  Validation: focused `taru-api` tests, SDK/contract generation checks that
  exist or are added, redaction inventory scan, and `git diff --check`.
  Review: Public Client, Admin API, and internal server DTO boundaries must
  remain separate.
  Evidence: `scripts/release-gate.ps1 -Mode api` passed with redaction
  inventory, OpenAPI/Public SDK/Admin contract tests, Rust client/protocol
  tests, TypeScript SDK generation/check, Admin Web contract generation/check,
  and final `git diff --check`.
  Handoff: Continue with deployment examples.

## M4 — Self-Hosted Deployment Examples

- [x] SHR-050 [owner=codex] [deps=SHR-020] [scope=docs,deploy,docker-compose.yml,docs/workstreams/self-hosted-release-readiness]
  Goal: Add self-hosted SQLite and PostgreSQL deployment examples plus
  configuration guidance for database URLs, artifact roots, staging roots,
  auth, Addons, Webhooks, Playback Runtime, and diagnostics.
  Validation: syntax/static checks for added config files; docs grep for
  required operator topics; `git diff --check`.
  Review: Examples must not contain real secrets or unsafe default public
  exposure.
  Evidence: `deploy/sqlite/taru.toml`, `deploy/postgres/taru.toml`,
  `deploy/compose/postgres.yml`, `docs/deployment/SELF_HOSTED.md`; TOML parse
  passed for both config examples; `docker compose config` passed with an
  example placeholder password; operator-topic grep passed.
  Handoff: Continue with backup/restore/upgrade docs.

## M5 — Backup, Restore, And Upgrade Runbook

- [x] SHR-060 [owner=codex] [deps=SHR-050] [scope=docs,self-hosted runbooks,docs/workstreams/self-hosted-release-readiness]
  Goal: Document backup, restore, migration, upgrade, and artifact-root
  consistency procedures for SQLite and PostgreSQL deployments.
  Validation: docs inventory proves DB, artifact root, NFO sidecars, staging
  cache, secrets, and migration rollback/forward expectations are covered;
  `git diff --check`.
  Review: Clearly distinguish durable state from cache/rebuildable state.
  Evidence: `docs/deployment/BACKUP_RESTORE_UPGRADE.md`; required-topic grep
  covered SQLite, PostgreSQL, artifact root, NFO sidecars, staging, cache,
  secrets, migration, rollback, forward, durable state, and rebuildable state.
  Handoff: Continue with end-to-end self-host smoke.

## M6 — End-To-End Self-Host Smoke

- [x] SHR-070 [owner=codex] [deps=SHR-030,SHR-040,SHR-050,SHR-060] [scope=tests,scripts,docs/workstreams/self-hosted-release-readiness]
  Goal: Add or compose a self-host smoke path that proves a real operator flow:
  library setup, scan, metadata/NFO path, Addon artwork proposal, Managed
  Artwork ingest/selection/public image serving, playback decision or stream,
  and redacted Admin diagnostics.
  Validation: smoke command passes for SQLite and has a PostgreSQL path or a
  documented PostgreSQL contract equivalent; `git diff --check`.
  Review: The smoke must exercise public/Admin boundaries, not private SQL
  inspection only.
  Evidence: `crates/taru-server/src/http/tests/self_host_smoke.rs`,
  `scripts/self-host-smoke.ps1`, `scripts/self-host-smoke.sh`; SQLite smoke
  passed 1/1; PostgreSQL smoke path delegated to the Managed Artwork contract
  harness and passed 6/6; `cargo fmt --all -- --check` and `git diff --check`
  passed.
  Handoff: Continue with closeout.

## M7 — Closeout

- [x] SHR-080 [owner=planner] [deps=SHR-070] [scope=docs/workstreams/self-hosted-release-readiness,docs/workstreams/README.md]
  Goal: Review, verify, and close the release readiness lane.
  Validation: release gate evidence in `EVIDENCE_AND_GATES.md`, final
  `cargo fmt --all -- --check`, `git diff --check`, and all required local/CI
  gate outputs.
  Review: Completion requires requirement-by-requirement evidence for the
  original goal.
  Evidence: closeout journal and completed workstream docs; docs/fast/postgres
  and workspace release gates passed; final `cargo fmt --all -- --check` and
  `git diff --check` passed.
  Handoff: Open separate lanes for AI, network traversal, Native Plugin ABI, or
  provider breadth only after this baseline is trustworthy.
