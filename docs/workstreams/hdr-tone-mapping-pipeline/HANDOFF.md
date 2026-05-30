# HDR Tone Mapping Pipeline - Handoff

Status: Draft
Last updated: 2026-05-30

## Current State

The lane remains a draft research workstream. `HTP-010` completed the
docs/research scope freeze with concerns because `ACDN-020` is still active on
the shared playback vocabulary files. No implementation code has been changed.

## Next Task

Do not start HDR implementation while `ACDN-020` is active. The next executable
HDR task is blocked `HTP-020`, and it should only start after the planner
confirms audio compatibility has completed, merged, or been serialized away
from the shared playback scope.

Planned `HTP-020` scope:

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
- the first executable slice would overlap active `ACDN-020` files;
- the lane needs a new ADR before code can start;
- hardware-specific behavior cannot be represented by existing ADRs.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include docs changed, research findings, proposed first executable task, and
whether the workstream should move from draft to active.
