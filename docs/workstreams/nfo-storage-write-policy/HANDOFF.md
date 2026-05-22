# NFO Storage Write Policy Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M48 is complete. VFS now exposes explicit write requests/reports, local storage
supports atomic replace with same-directory temp files where supported, and NFO
export requests that safer write policy for sidecar persistence.

NFO import/export summaries now carry internal `NfoFailureKind` diagnostics for
parse, preservation, storage read/write, unsupported atomic write, invalid
sidecar path, missing media item, and unknown failures.

The worktree already contains unrelated uncommitted `admin-web-console`
planning files and a matching unstaged `docs/workstreams/README.md` entry.
Do not revert or accidentally include those changes when committing M48 slices.

## Completed Tasks

- `NSW-010`: scope and evidence freeze.
- `NSW-020`: VFS atomic local write boundary.
- `NSW-030`: NFO export write policy and diagnostics.
- `NSW-040`: validation and closeout.

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

Closeout result: all gates passed on 2026-05-17. Focused test counts were 22
for `nako-vfs` and 16 for `nako-nfo`; workspace nextest passed 305 tests.

## Follow-ons Outside M48

- Backup/retention policy for sidecar writes.
- Public API exposure for NFO export diagnostics.
- Nested XML preservation and broad NFO compatibility profiles.
- Soft-link and hard-link library write policy.
