# Audio Compatibility Downmix Normalization - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

The lane is open and linked from playback architecture indexes. `ACDN-010` is
complete. `ACDN-020` is complete in `nako-playback`.

Playback now owns audio output requirement values for channel support, downmix
intent, normalization intent, and audio-specific compatibility reasons.
`TranscodeRequirement` carries the playback-owned audio output requirement, and
playback tests cover channel-limited transcode selection plus the case where
remux must not be selected when downmix is required.

## Next Task

Run `ACDN-030` after planner/reviewer acceptance of `ACDN-020`.

Owned scope:

- `crates/nako-transcode/src/policy.rs`
- `crates/nako-transcode/src/pipeline.rs`
- `crates/nako-transcode/src/profile.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/hls.rs`

Required validation:

```text
cargo nextest run -p nako-transcode audio --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
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

## ACDN-020 Evidence

```text
cargo nextest run -p nako-playback audio --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

All ACDN-020 gates passed on 2026-05-30. See
`docs/workstreams/audio-compatibility-downmix-normalization/EVIDENCE_AND_GATES.md`.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include changed files, tests run, and evidence anchors.
