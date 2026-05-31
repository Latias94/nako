# Transcode Capability Inventory Matrix - Handoff

Status: Active
Last updated: 2026-05-31

## Current State

`TCIM-010` is complete. The lane is open as a transcode-only capability
inventory workstream that may run in parallel with HDR `HTP-030` as long as it
does not change pipeline selection or FFmpeg command planning.

## Next Task

Assign `TCIM-020`.

Required context:

```text
docs/workstreams/transcode-capability-inventory-matrix/CONTEXT.jsonl
docs/adr/0045-ffmpeg-hardware-pipeline-planner.md
docs/adr/0046-ffmpeg-probe-inventory.md
docs/adr/0048-playback-transcode-startup-degradation.md
docs/workstreams/transcode-interface-and-runtime-plan-deepening/CLOSEOUT.md
docs/workstreams/hdr-tone-mapping-pipeline/HANDOFF.md
```

Required validation:

```text
cargo nextest run -p nako-transcode hardware --no-fail-fast
cargo nextest run -p nako-transcode probe --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Stop Conditions

Return to planner coordination if:

- the task needs `pipeline.rs`, `ffmpeg.rs`, server routes, Public Client DTOs,
  or release packaging;
- the task tries to choose a hardware pipeline rather than report capability
  evidence;
- the task changes HDR `HTP-030` scope or timing.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include changed files, capability facts added, validation evidence, and
follow-ons split.
