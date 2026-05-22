# 2026-05-21 — NSPA-060 Retention Diagnostic Failure

## Scope

- Task: NSPA-060 partial failure rollback and repair gates.
- Slice: export-sidecar backup retention diagnostic failure.
- Files:
  - `crates/nako-server/src/app/tests/nfo.rs`
  - `docs/workstreams/nfo-sidecar-promotion-apply/TODO.md`
  - `docs/workstreams/nfo-sidecar-promotion-apply/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/nfo-sidecar-promotion-apply/HANDOFF.md`

## Behavior Proven

- Forced export sidecar apply still commits when the underlying VFS backup
  retention pruning reports a diagnostic failure.
- The updated sidecar keeps round-trip preserved XML content from the previous
  sidecar.
- The failed prune candidate remains in place, proving the diagnostic came from
  the retention boundary rather than being silently ignored.
- Durable audit outcome records a redacted `prune_failure_count` while avoiding
  raw local paths and raw XML in operator-facing diagnostics.

## Validation

```powershell
cargo nextest run -p nako-server nfo_sidecar_apply_export_retention_diagnostic_failure_commits_with_redacted_warning --no-fail-fast
cargo fmt --all -- --check
cargo nextest run -p nako-server nfo_sidecar_apply --no-fail-fast
cargo nextest run -p nako-server nfo --no-fail-fast
```

All commands passed.

## Remaining Work

- NSPA-060 still needs backup restore/rollback failure coverage before the task
  can be marked DONE.
