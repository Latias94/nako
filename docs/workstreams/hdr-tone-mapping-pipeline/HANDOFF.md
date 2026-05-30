# HDR Tone Mapping Pipeline - Handoff

Status: Draft
Last updated: 2026-05-30

## Current State

The lane is open as a draft research workstream. No implementation code has
been changed by the planner.

## Next Task

Run `HTP-010` with `run-workstream-task`.

Owned scope:

- `docs/workstreams/hdr-tone-mapping-pipeline/`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Required validation:

```text
python -m json.tool docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json
git diff --check -- docs/workstreams/hdr-tone-mapping-pipeline docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md
```

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
