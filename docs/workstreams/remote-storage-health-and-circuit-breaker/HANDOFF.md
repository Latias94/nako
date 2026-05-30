# Remote Storage Health And Circuit Breaker - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

The lane is open and linked from the storage/VFS architecture indexes.
`RSHC-010` and `RSHC-020` are complete.

`RSHC-020` added the durable **Storage Backend Health** repository contract,
domain records, SQLite and PostgreSQL adapters, facade dispatch, baseline
schema, and repository contract coverage. The contract records backend-scoped
health status, **Storage Circuit Breaker** state, consecutive failures,
redaction-safe last failure class/message, backoff timestamp, recovery state,
and reset behavior.

## Next Task

Planner can assign `RSHC-030` after review if the lane should continue.

Expected next owned scope:

- `crates/nako-server/src/app/storage.rs`
- `crates/nako-server/src/app/tests/storage*.rs`

Expected validation:

```text
cargo nextest run -p nako-server storage_health --no-fail-fast
cargo nextest run -p nako-server storage --no-fail-fast
```

## Stop Conditions

Return to planner coordination if:

- the task needs schema migration policy outside storage health;
- playback staging, scan scheduling, or Admin route changes become necessary;
- the health model requires a durable ADR instead of fitting ADR 0016/0017;
- existing user changes appear in files you need to edit.

## RSHC-020 Validation

```text
cargo nextest run -p nako-db storage_backend_health --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

All required gates passed on 2026-05-30. `git diff --check` reported only
Windows line-ending normalization warnings.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include changed files, tests run, and evidence anchors.
