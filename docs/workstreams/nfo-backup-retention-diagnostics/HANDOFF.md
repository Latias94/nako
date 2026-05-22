# NFO Backup Retention And Diagnostics Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M50 completed after M49 commit `e6f304f`.

The worktree currently contains unrelated `admin-web-console` planning files and
a matching unstaged `docs/workstreams/README.md` entry. Do not revert or
accidentally include those changes when committing M50 slices.

## Completed Work

- `nako-vfs` now exposes `StorageBackupPolicy` with keep-latest retention and
  reports created backups, pruned backups, and prune failures.
- `LocalFsBackend` prunes only same-sidecar Nako backup files matching the
  backup prefix, preserving unrelated sidecar backups and manual files.
- `nako-nfo` forced export requests keep-latest retention when it requests a
  backup and surfaces backup/pruning diagnostics in `NfoExportSummary`.
- Admin diagnostics reuse existing job summary passthrough; no public client
  protocol crate changed.

## Validation

Completed gates:

```powershell
cargo check -p nako-vfs --tests
cargo nextest run -p nako-vfs --no-fail-fast
cargo check -p nako-nfo --tests
cargo nextest run -p nako-nfo --no-fail-fast
cargo check -p nako-api --tests
cargo nextest run -p nako-api --no-fail-fast
cargo check -p nako-server --tests
cargo nextest run -p nako-server nfo --no-fail-fast
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

Final workspace nextest result: 315 tests passed.

## Follow-ons Outside M50

- Configurable retention count in library/profile options.
- Persistent backup history if job summaries are insufficient.
- UI/admin page for NFO backup history.
- Age-based retention.
