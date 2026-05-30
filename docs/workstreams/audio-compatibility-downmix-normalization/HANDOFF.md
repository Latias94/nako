# Audio Compatibility Downmix Normalization - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

The lane is open and linked from playback architecture indexes. `ACDN-010` is
complete. No implementation code has been changed by the planner.

## Next Task

Run `ACDN-020` with `run-workstream-task`.

Owned scope:

- `crates/nako-playback/src/capability.rs`
- `crates/nako-playback/src/values.rs`
- `crates/nako-playback/src/lib.rs`

Required validation:

```text
cargo nextest run -p nako-playback audio --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Stop Conditions

Return to planner coordination if:

- the task needs `nako-transcode`, server playback, public API DTO, or generated
  contract changes;
- HDR tone mapping or subtitle burn-in becomes part of the design;
- the audio vocabulary needs a new ADR rather than fitting ADR 0038/0044;
- existing user changes appear in files you need to edit.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include changed files, tests run, and evidence anchors.
