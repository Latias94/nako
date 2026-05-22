# NFO Sidecar Backup Policy Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M49 is complete after M48 commit `3749ec6`. Local storage can create
same-directory backups before replacing existing sidecars, and NFO forced
export now requests and reports backups only for existing NFO overwrites.

Backup failures are classified as `NfoFailureKind::StorageBackup` and prevent
the final sidecar replacement. Fresh sidecar exports do not create backups.

The worktree already contains unrelated uncommitted `admin-web-console`
planning files and a matching unstaged `docs/workstreams/README.md` entry. Do
not revert or accidentally include those changes when committing M49 slices.

## Completed Tasks

- `NSB-010`: scope and evidence freeze.
- `NSB-020`: VFS local backup write boundary.
- `NSB-030`: NFO export backup diagnostics.
- `NSB-040`: validation and closeout.

## Validation

```powershell
cargo fmt --all -- --check
cargo check -p nako-vfs --tests
cargo nextest run -p nako-vfs --no-fail-fast
cargo check -p nako-nfo --tests
cargo nextest run -p nako-nfo --no-fail-fast
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

Closeout result: all gates passed on 2026-05-17. Focused test counts were 25
for `nako-vfs` and 18 for `nako-nfo`; workspace nextest passed 310 tests.

## Follow-ons Outside M49

- Backup retention pruning and configurable retention count.
- Public API/admin diagnostics for backup reports.
- Centralized backup directory policy.
- Soft-link and hard-link library write policy.
