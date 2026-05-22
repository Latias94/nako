# NFO Backup Retention And Diagnostics Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Baseline Evidence

- M49 creates same-directory local backups before overwriting existing NFO
  sidecars.
- M49 backup files use the `*.nako-backup-*` naming convention.
- M49 does not prune backups or expose pruning diagnostics.
- Public client protocol crates should remain unchanged in this slice.

## Focused Gates

```powershell
cargo fmt --all -- --check
cargo check -p nako-vfs --tests
cargo nextest run -p nako-vfs --no-fail-fast
cargo check -p nako-nfo --tests
cargo nextest run -p nako-nfo --no-fail-fast
```

## Closeout Gates

```powershell
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Log

- 2026-05-17: Workstream opened for M50.
- 2026-05-17: `StorageBackupPolicy` and `StorageBackupRetention` added to the
  VFS write boundary; `StorageBackupReport` now reports pruned backups and
  prune failures.
- 2026-05-17: `LocalFsBackend` prunes only same-sidecar Nako backup files that
  match the `*.nako-backup-*` prefix and leaves unrelated backups/manual files
  untouched.
- 2026-05-17: NFO forced export requests keep-latest backup retention and
  reports created backups, pruned backup counts, and prune failures in
  `NfoExportSummary`.
- 2026-05-17: Admin diagnostics reuse existing `JobResponse.summary`; no public
  client protocol crate changed.

## Completed Gates

```powershell
cargo fmt --all -- --check
cargo check -p nako-vfs --tests
cargo nextest run -p nako-vfs --no-fail-fast
# 28 tests passed

cargo check -p nako-nfo --tests
cargo nextest run -p nako-nfo --no-fail-fast
# 19 tests passed

cargo check -p nako-api --tests
cargo nextest run -p nako-api --no-fail-fast
# 13 tests passed

cargo check -p nako-server --tests
cargo nextest run -p nako-server nfo --no-fail-fast
# 5 selected tests passed

git diff --name-only -- crates/nako-client-protocol
# no output

cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
# 315 tests passed

git diff --check
# passed with Git CRLF normalization warnings only
```

## Expected Closeout Evidence

- VFS tests prove keep-latest pruning of Nako-created backups.
- VFS tests prove unrelated files and non-matching backups are preserved.
- VFS tests prove prune failures are reported without aborting the completed
  sidecar write.
- NFO tests prove forced export records backup and pruning diagnostics.
- Admin/public boundary audit proves no public client protocol change.
- Focused and workspace gates pass.
