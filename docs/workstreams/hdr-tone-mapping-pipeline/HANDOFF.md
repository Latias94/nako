# HDR Tone Mapping Pipeline - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

The lane is active for `HTP-020`. `HTP-010` completed the docs/research scope freeze, and the planner merged the accepted `ACDN-020` audio output baseline into this HDR branch so the shared playback vocabulary files are current. No HDR implementation code has been changed yet.

## Next Task

Run `HTP-020` with `run-workstream-task`. Keep the task playback-only and build on the merged `ACDN-020` audio output requirement vocabulary.

Owned `HTP-020` scope:

- `crates/nako-playback/src/capability.rs`
- `crates/nako-playback/src/values.rs`
- `crates/nako-playback/src/lib.rs`

Owned docs scope for any continuation:

- `docs/workstreams/hdr-tone-mapping-pipeline/`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Required docs validation:

```text
python -m json.tool docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json
git diff --check -- docs/workstreams/hdr-tone-mapping-pipeline docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md
```

Planned `HTP-020` implementation validation after planner activation:

```text
cargo nextest run -p nako-playback hdr --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Research conclusion:

- current probe facts are sufficient for the first playback planning slice;
- current `supports_hdr` client capability is sufficient for the first slice;
- first implementation is playback-owned **Color Pipeline Requirement** and
  typed reasons, not FFmpeg command planning;
- first media-output behavior is a later software-first HLS HDR-to-SDR path.

## Stop Conditions

Return to planner coordination if:

- implementation code seems necessary;
- the task would expand beyond playback-owned color requirement vocabulary;
- the lane needs a new ADR before code can start;
- hardware-specific behavior cannot be represented by existing ADRs.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include docs changed, research findings, proposed first executable task, and
whether `HTP-020` is ready for planner review.
