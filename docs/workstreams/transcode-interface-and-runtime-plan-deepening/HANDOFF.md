# Transcode Interface And Runtime Plan Deepening - Handoff

Status: Closed
Last updated: 2026-05-31

## Current State

`TIRP-010`, `TIRP-020`, `TIRP-030`, and `TIRP-040` are complete. The lane has a
transcode-owned HLS runtime plan Interface and a curated FFmpeg execution
planner Interface before HDR `HTP-030` adds color pipeline and FFmpeg filter
pressure.

`audio-compatibility-downmix-normalization` and this transcode Interface lane
are closed. Keep HDR/tone-map, hardware matrix expansion, HLS lifecycle
consolidation, and resource admission unification split from this Interface
ratchet.

## Next Task

No task remains in this workstream.

Recommended next planner action: start HDR `HTP-030` from current `main` after
syncing or recreating the HDR worktree. The implementation should use
`HlsRuntimePlan`, `FfmpegExecutionPlanner`, `HlsExecutionPlanRequest`, and
transcode-owned policy/profile values rather than reintroducing server-side raw
FFmpeg request assembly.

Required context:

```text
docs/workstreams/transcode-interface-and-runtime-plan-deepening/CONTEXT.jsonl
docs/adr/0038-playback-planning-and-transcode-policy-seams.md
docs/adr/0045-ffmpeg-hardware-pipeline-planner.md
docs/adr/0052-hls-runtime-and-media-engine-boundary.md
docs/workstreams/transcode-interface-and-runtime-plan-deepening/EVIDENCE_AND_GATES.md
```

Closeout validation:

```text
python -m json.tool docs/workstreams/transcode-interface-and-runtime-plan-deepening/WORKSTREAM.json
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-transcode remux --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Stop Conditions

Return to planner coordination if:

- `nako-transcode` would need a direct dependency on `nako-playback`;
- follow-on work grows into subtitle burn-in, broad hardware matrices, HLS
  lifecycle consolidation, or resource admission unification without a new
  planner-approved workstream;
- public API DTO or generated contract changes become necessary;
- existing user changes appear in files you need to edit.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include changed files, tests run, Interface changes, and evidence anchors.
