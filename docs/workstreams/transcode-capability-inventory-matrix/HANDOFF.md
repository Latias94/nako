# Transcode Capability Inventory Matrix - Handoff

Status: Closed
Last updated: 2026-05-31

## Current State

This workstream is closed. `TCIM-030` was reviewed, verified, merged into
`main`, and accepted as the broader capability inventory evidence slice.
Optional decoder, encoder, filter, tone-map, subtitle burn-in, and bitstream
filter facts are now observable without changing HLS pipeline selection.

## Follow-ons

Open separate workstreams for:

- hardware tone-map execution and vendor filter chains;
- HEVC/AV1 output policy and FFmpeg command planning;
- subtitle burn-in execution behavior;
- Admin/Public capability reporting or release hardware matrices;
- HLS lifecycle/resource admission work.

Required context for follow-ons:

```text
docs/workstreams/transcode-capability-inventory-matrix/CONTEXT.jsonl
docs/adr/0045-ffmpeg-hardware-pipeline-planner.md
docs/adr/0046-ffmpeg-probe-inventory.md
docs/adr/0048-playback-transcode-startup-degradation.md
docs/workstreams/transcode-interface-and-runtime-plan-deepening/CLOSEOUT.md
docs/workstreams/hdr-tone-mapping-pipeline/CLOSEOUT.md
```

Completed validation:

```text
python -m json.tool docs/workstreams/transcode-capability-inventory-matrix/WORKSTREAM.json
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
- the task tries to turn optional capability evidence into an admission or
  runtime-selection rule.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include changed files, capability facts added, validation evidence, and
follow-ons split.
