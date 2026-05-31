# HLS Runtime Lifecycle Boundary - Handoff

Status: Active
Last updated: 2026-05-31

## Current State

The workstream is open. `HRLB-010` is the next task and should freeze HLS
lifecycle invariants before behavior changes.

## Next Task

Assign `HRLB-010`.

Required context:

```text
docs/workstreams/hls-runtime-lifecycle-boundary/CONTEXT.jsonl
docs/adr/0052-hls-runtime-and-media-engine-boundary.md
docs/architecture/PLAYBACK.md
docs/architecture/LANES.md
docs/workstreams/transcode-capability-inventory-matrix/CLOSEOUT.md
docs/workstreams/hdr-tone-mapping-pipeline/CLOSEOUT.md
docs/workstreams/hls-progressive-runtime-boundary/HANDOFF.md
docs/workstreams/hls-seek-restart-lifecycle/HANDOFF.md
```

Required validation for `HRLB-010`:

```text
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-runtime-lifecycle-boundary docs/architecture/PLAYBACK.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

## Stop Conditions

Return to planner coordination if:

- implementation needs `nako-transcode` pipeline selection or FFmpeg command
  planning;
- the task needs Public/Admin DTO changes or storage schema changes;
- artifact I/O pressure requires storage health/circuit-breaker behavior
  changes;
- client/player UX, LL-HLS/CMAF, DASH/CMAF, DRM/key delivery, or remote worker
  execution becomes necessary.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include files changed, invariant coverage, validation evidence, and follow-ons
split.
