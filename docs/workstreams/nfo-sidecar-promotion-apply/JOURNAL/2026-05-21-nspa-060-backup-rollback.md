# 2026-05-21 — NSPA-060 Backup Restore And Rollback

## Scope

- Task: NSPA-060 partial failure rollback and repair gates.
- Slice: backup-backed export rollback after sidecar mutation and final audit
  failure.
- Files:
  - `crates/nako-vfs/src/lib.rs`
  - `crates/nako-vfs/src/local.rs`
  - `crates/nako-server/src/app/nfo.rs`
  - `crates/nako-server/src/app/storage.rs`
  - `crates/nako-server/src/app/tests/mod.rs`
  - `crates/nako-server/src/app/tests/nfo.rs`
  - workstream TODO, evidence, handoff, and `WORKSTREAM.json`

## Behavior Proven

- VFS now has an explicit `StorageRestoreRequest` / `StorageRestoreReport`
  boundary for restoring a target from a backup without exposing raw OS paths to
  callers.
- Local VFS restores by copying the backup into a same-directory temporary file
  and replacing the target through the existing atomic replace primitive, with
  cleanup on failure.
- NFO export apply attempts rollback when a sidecar write succeeded, a backup
  exists, and the final audit commit fails.
- If restore succeeds, the sidecar apply records `RollbackComplete`, the old
  sidecar content is visible again, replay is terminal/idempotent, and audit
  diagnostics remain redacted.
- If restore fails, the sidecar apply records `RepairPending`, preserves the
  current sidecar write for operator repair, replay is terminal/idempotent, and
  diagnostics remain redacted.

## Validation

```powershell
cargo nextest run -p nako-vfs local_backend_restore --no-fail-fast
cargo nextest run -p nako-server nfo_sidecar_apply_export_audit_failure_restores_backup_and_records_rollback_complete --no-fail-fast
cargo nextest run -p nako-server nfo_sidecar_apply_export_audit_failure_records_repair_pending_when_backup_restore_fails --no-fail-fast
cargo fmt --all -- --check
cargo nextest run -p nako-server nfo_sidecar_apply --no-fail-fast
cargo nextest run -p nako-server nfo --no-fail-fast
cargo nextest run -p nako-vfs --no-fail-fast
cargo nextest run -p nako-nfo --no-fail-fast
cargo nextest run -p nako-db nfo_sidecar_apply --no-fail-fast
cargo check -p nako-server
```

All commands passed. `cargo check -p nako-server` still reports existing
unrelated unused/dead-code warnings.

## Result

NSPA-060 is complete. Next task is NSPA-070 closeout/exposure split.
