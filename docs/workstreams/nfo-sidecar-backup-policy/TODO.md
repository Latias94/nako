# NFO Sidecar Backup Policy Task Ledger

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

- [x] NSB-010 [owner=codex] [deps=none] [scope=docs/workstreams/nfo-sidecar-backup-policy,docs/GOALS.md]
  Goal: Open M49 with backup policy boundaries, non-goals, and validation
  gates.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/nfo-sidecar-backup-policy/DESIGN.md`.
  Handoff: Continue with VFS local backup support.

## M1 - VFS Local Backup Write Boundary

- [x] NSB-020 [owner=codex] [deps=NSB-010] [scope=crates/taru-vfs/src/lib.rs,crates/taru-vfs/src/local.rs,crates/taru-vfs/src/cache.rs]
  Goal: Extend explicit storage writes with optional existing-file backup and
  implement local same-directory backup before atomic replace.
  Validation: `cargo check -p taru-vfs --tests`;
  `cargo nextest run -p taru-vfs local::tests::local_backend_backup --no-fail-fast`.
  Evidence: local backend tests prove backup is created before overwrite,
  fresh writes do not create backup, and unsupported backup requests fail.
  Handoff: Wire NFO forced export to request backup.

## M2 - NFO Export Backup Diagnostics

- [x] NSB-030 [owner=codex] [deps=NSB-020] [scope=crates/taru-nfo/src/export.rs,crates/taru-nfo/src/summary.rs,crates/taru-nfo/src/lib.rs]
  Goal: Have NFO forced export request backup only for existing sidecar
  overwrites and report backup creation/failure internally.
  Validation: `cargo check -p taru-nfo --tests`;
  `cargo nextest run -p taru-nfo nfo_service --no-fail-fast`.
  Evidence: service tests prove forced overwrite records a backup, fresh export
  does not, and backup failure prevents final replacement.
  Handoff: Run focused and workspace closeout gates.

## M3 - Validation And Closeout

- [x] NSB-040 [owner=codex] [deps=NSB-030] [scope=workspace,docs]
  Goal: Close M49 with focused and workspace validation, evidence updates, and
  follow-on notes.
  Validation: `cargo fmt --all -- --check`; `cargo check -p taru-vfs --tests`;
  `cargo nextest run -p taru-vfs --no-fail-fast`; `cargo check -p taru-nfo --tests`;
  `cargo nextest run -p taru-nfo --no-fail-fast`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Evidence: `EVIDENCE_AND_GATES.md` and `docs/GOALS.md`.
  Handoff: Recommend follow-ons for retention pruning, public diagnostics, or
  library file link policy only after M49 evidence is complete.
