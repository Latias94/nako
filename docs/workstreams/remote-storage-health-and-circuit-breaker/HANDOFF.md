# Remote Storage Health And Circuit Breaker - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

The lane is open and linked from the storage/VFS architecture indexes.
`RSHC-010` is complete. No implementation code has been changed by the planner.

## Next Task

Run `RSHC-020` with `run-workstream-task`.

Owned scope:

- `crates/nako-core/src/repository/vfs.rs`
- storage health domain module additions under `crates/nako-core/src/`
- SQLite/PostgreSQL VFS repository adapter modules under `crates/nako-db/src/`
- repository contract tests that prove parity

Required validation:

```text
cargo nextest run -p nako-db storage_backend_health --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Stop Conditions

Return to planner coordination if:

- the task needs schema migration policy outside storage health;
- playback staging, scan scheduling, or Admin route changes become necessary;
- the health model requires a durable ADR instead of fitting ADR 0016/0017;
- existing user changes appear in files you need to edit.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include changed files, tests run, and evidence anchors.
