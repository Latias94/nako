# NFO Backup Retention And Diagnostics Task Ledger

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

- [x] NBR-010 [owner=codex] [deps=none] [scope=docs/workstreams/nfo-backup-retention-diagnostics,docs/GOALS.md]
  Goal: Open M50 with retention, diagnostics, non-goals, and validation gates.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/nfo-backup-retention-diagnostics/DESIGN.md`.
  Handoff: Continue with VFS retention policy.

## M1 - VFS Backup Retention Policy

- [x] NBR-020 [owner=codex] [deps=NBR-010] [scope=crates/nako-vfs/src/lib.rs,crates/nako-vfs/src/local.rs]
  Goal: Extend storage backup requests/reports with keep-latest retention and
  implement conservative local pruning for Nako-created backups of the same
  sidecar.
  Validation: `cargo check -p nako-vfs --tests`;
  `cargo nextest run -p nako-vfs local::tests::local_backend_backup --no-fail-fast`.
  Evidence: tests prove old Nako backups are pruned, newest backups are kept,
  non-matching files are preserved, and prune failures are reported.
  Handoff: Completed. VFS backup policy now carries keep-latest retention,
  local storage prunes matching Nako backups conservatively, and tests cover
  successful pruning, unrelated-file preservation, zero-retention pruning, and
  prune failure diagnostics.

## M2 - NFO Export Retention Diagnostics

- [x] NBR-030 [owner=codex] [deps=NBR-020] [scope=crates/nako-nfo/src/export.rs,crates/nako-nfo/src/summary.rs,crates/nako-nfo/src/lib.rs]
  Goal: Have NFO forced export request backup retention and report created,
  pruned, and failed backup/pruning diagnostics in internal summaries.
  Validation: `cargo check -p nako-nfo --tests`;
  `cargo nextest run -p nako-nfo nfo_service --no-fail-fast`.
  Evidence: service tests prove retention diagnostics on forced overwrite and
  no public protocol dependency.
  Handoff: Completed. Forced NFO export requests keep-latest backup retention
  and records backup, pruning, and prune-failure diagnostics in internal
  summaries.

## M3 - Admin/Public Boundary Audit

- [x] NBR-040 [owner=codex] [deps=NBR-030] [scope=crates/nako-api,crates/nako-server,crates/nako-client-protocol]
  Goal: Verify admin-facing diagnostics are inspectable through existing job
  summaries or add admin-only DTO mapping without changing public client
  protocol.
  Validation: `cargo check -p nako-api --tests`; relevant `nako-api`/`nako-server`
  nextest tests; public route inventory tests if touched.
  Evidence: tests or source audit prove `nako-client-protocol` remains
  unchanged and public route inventory does not expose NFO internals.
  Handoff: Completed. Existing admin job summaries preserve NFO retention
  diagnostics, public OpenAPI inventory stays clean, and `nako-client-protocol`
  remains unchanged.

## M4 - Validation And Closeout

- [x] NBR-050 [owner=codex] [deps=NBR-040] [scope=workspace,docs]
  Goal: Close M50 with focused and workspace validation, evidence updates, and
  follow-on notes.
  Validation: `cargo fmt --all -- --check`; `cargo check -p nako-vfs --tests`;
  `cargo nextest run -p nako-vfs --no-fail-fast`; `cargo check -p nako-nfo --tests`;
  `cargo nextest run -p nako-nfo --no-fail-fast`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Evidence: `EVIDENCE_AND_GATES.md` and `docs/GOALS.md`.
  Handoff: Completed. Follow-ons remain configurable retention, persistent
  backup history, and admin UX only after this evidence-backed storage/NFO
  boundary.
