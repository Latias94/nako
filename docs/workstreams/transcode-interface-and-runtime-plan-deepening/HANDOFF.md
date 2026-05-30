# Transcode Interface And Runtime Plan Deepening - Handoff

Status: Active
Last updated: 2026-05-31

## Current State

`TIRP-010` is complete. The lane is open to deepen the `nako-transcode`
Interface before HDR `HTP-030` adds color pipeline and FFmpeg filter pressure.

`audio-compatibility-downmix-normalization` is closed, so the main remaining
playback/transcode risk is server-side assembly of HLS transcode details.

## Next Task

Assign `TIRP-020`.

Required context:

```text
docs/workstreams/transcode-interface-and-runtime-plan-deepening/CONTEXT.jsonl
docs/adr/0038-playback-planning-and-transcode-policy-seams.md
docs/adr/0045-ffmpeg-hardware-pipeline-planner.md
docs/adr/0052-hls-runtime-and-media-engine-boundary.md
```

Required validation:

```text
cargo nextest run -p nako-transcode hls audio --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Stop Conditions

Return to planner coordination if:

- `nako-transcode` would need a direct dependency on `nako-playback`;
- the task grows into HDR tone mapping, subtitle burn-in, broad hardware
  matrices, HLS lifecycle consolidation, or resource admission unification;
- public API DTO or generated contract changes become necessary;
- existing user changes appear in files you need to edit.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include changed files, tests run, Interface changes, and evidence anchors.
