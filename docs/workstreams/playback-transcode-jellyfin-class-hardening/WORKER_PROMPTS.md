# Playback Transcode Jellyfin-Class Hardening - Worker Prompts

Status: Active
Last updated: 2026-05-31

Use these prompts after `PTJCH-010` and `PTJCH-020` are complete. They are
designed for separate Codex terminals or separate worktrees.

## Shared Worker Rules

Before starting any worker task:

- Read `AGENTS.md`, `CONTEXT.md`, `docs/architecture/PLAYBACK.md`,
  `docs/architecture/LANES.md`, this workstream's `DESIGN.md`, `TODO.md`,
  `EVIDENCE_AND_GATES.md`, and `CONTEXT.jsonl`.
- Check `git status --short` and do not revert or delete changes you did not
  make.
- Prefer a separate worktree for each parallel task.
- Keep docs/code original. Jellyfin source is reference pressure only.
- Do not change public DTOs, schema migrations, request identity, artifact path
  format, or shared server playback behavior without planner approval.
- Do not edit shared architecture/workstream status docs from a worker branch
  unless the task explicitly asks for it. Put task notes under
  `docs/workstreams/playback-transcode-jellyfin-class-hardening/worker-notes/`.

Suggested worktree commands from the main Nako checkout:

```text
git worktree add ../nako-worktrees/nako-ptjch-110-playback-capability -b work/ptjch-110-playback-capability
git worktree add ../nako-worktrees/nako-ptjch-120-transcode-pipeline-capability -b work/ptjch-120-transcode-pipeline-capability
git worktree add ../nako-worktrees/nako-ptjch-130-ffmpeg-adapter-split -b work/ptjch-130-ffmpeg-adapter-split
```

## PTJCH-110 - Playback Capability Worker

Paste this into a fresh Codex session:

```text
You are working in F:\SourceCodes\Rust\nako on the Nako workstream
docs/workstreams/playback-transcode-jellyfin-class-hardening.

Execute task PTJCH-110.

Objective:
Deepen Playback Capability conditions and compatibility reason tests inside
crates/nako-playback without adding transcode execution, FFmpeg command
assembly, HLS artifact identity, server lifecycle, API DTO, or database
behavior.

Read first:
- AGENTS.md
- CONTEXT.md
- docs/architecture/PLAYBACK.md
- docs/adr/0038-playback-planning-and-transcode-policy-seams.md
- docs/adr/0044-playback-capability-profile-planner.md
- docs/workstreams/playback-transcode-jellyfin-class-hardening/DESIGN.md
- docs/workstreams/playback-transcode-jellyfin-class-hardening/TODO.md
- docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKER_PROMPTS.md

Owned scope:
- crates/nako-playback
- tests inside crates/nako-playback
- optional task notes at
  docs/workstreams/playback-transcode-jellyfin-class-hardening/worker-notes/PTJCH-110.md

Forbidden scope:
- crates/nako-transcode
- crates/nako-server
- crates/nako-api
- artifact path/request identity formats
- schema migrations

Expected direction:
- Inspect the current PlaybackPlanner and compatibility reason model before
  editing.
- Add table-driven coverage for representative Direct Play, Remux, and HLS
  Transcode decisions where current tests are shallow.
- Keep the Module output as playback-owned decisions and requirements.
- If richer facts are needed from transcode or server runtime, stop and record
  the missing Interface instead of crossing the boundary.

Validation:
- cargo nextest run -p nako-playback --no-fail-fast
- cargo fmt --all -- --check
- git diff --check

Stop and hand back to the planner if the task requires public DTO changes,
new transcode execution fields, server playback edits, request identity
changes, or artifact format changes.
```

## PTJCH-120 - Transcode Pipeline Capability Worker

Paste this into a fresh Codex session:

```text
You are working in F:\SourceCodes\Rust\nako on the Nako workstream
docs/workstreams/playback-transcode-jellyfin-class-hardening.

Execute task PTJCH-120.

Objective:
Deepen stage-aware Transcode Pipeline Capability matching for hardware and
fallback requirements inside nako-transcode without changing playback policy,
server runtime behavior, public API DTOs, or raw FFmpeg command assembly.

Read first:
- AGENTS.md
- CONTEXT.md
- docs/architecture/PLAYBACK.md
- docs/adr/0045-ffmpeg-hardware-pipeline-planner.md
- docs/adr/0046-ffmpeg-probe-inventory.md
- docs/adr/0047-cpu-transcode-readiness.md
- docs/adr/0048-playback-transcode-startup-degradation.md
- docs/workstreams/transcode-capability-inventory-matrix/
- docs/workstreams/playback-transcode-jellyfin-class-hardening/DESIGN.md
- docs/workstreams/playback-transcode-jellyfin-class-hardening/TODO.md
- docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKER_PROMPTS.md

Owned scope:
- crates/nako-transcode/src/pipeline.rs
- crates/nako-transcode/src/hardware.rs
- crates/nako-transcode/src/probe.rs
- closely related tests inside crates/nako-transcode
- optional task notes at
  docs/workstreams/playback-transcode-jellyfin-class-hardening/worker-notes/PTJCH-120.md

Forbidden scope:
- crates/nako-playback policy changes
- crates/nako-server runtime/session changes
- FFmpeg command argument assembly
- public API DTOs
- schema migrations

Expected direction:
- Inspect current TranscodePipelinePlan, hardware capability report, and probe
  facts before editing.
- Make capability matching stage-aware where the current model is too shallow.
- Preserve the separation between requirement planning, hardware inventory,
  and FFmpeg command execution.
- Add focused tests that explain fallback/degradation instead of asserting only
  that a plan exists.

Validation:
- cargo nextest run -p nako-transcode pipeline hardware probe --no-fail-fast
- cargo fmt --all -- --check
- git diff --check

Stop and hand back to the planner if the task requires PlaybackPlanner changes,
server HLS/remux lifecycle changes, public DTO changes, request identity
changes, or new raw FFmpeg arguments.
```

## PTJCH-130 - FFmpeg Adapter Worker

Paste this into a fresh Codex session:

```text
You are working in F:\SourceCodes\Rust\nako on the Nako workstream
docs/workstreams/playback-transcode-jellyfin-class-hardening.

Execute task PTJCH-130.

Objective:
Split FFmpeg Adapter internals so command planning breadth can grow without
creating one large EncodingHelper-style Module. Preserve external behavior and
keep low-level FFmpeg builder details inside nako-transcode.

Read first:
- AGENTS.md
- CONTEXT.md
- docs/architecture/PLAYBACK.md
- docs/adr/0045-ffmpeg-hardware-pipeline-planner.md
- docs/adr/0052-hls-runtime-and-media-engine-boundary.md
- docs/workstreams/transcode-interface-and-runtime-plan-deepening/
- docs/workstreams/playback-transcode-jellyfin-class-hardening/DESIGN.md
- docs/workstreams/playback-transcode-jellyfin-class-hardening/TODO.md
- docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKER_PROMPTS.md

Owned scope:
- crates/nako-transcode/src/ffmpeg.rs
- crates/nako-transcode/src/execution.rs
- new private/internal modules under crates/nako-transcode/src/ if the local
  pattern supports them
- closely related tests inside crates/nako-transcode
- optional task notes at
  docs/workstreams/playback-transcode-jellyfin-class-hardening/worker-notes/PTJCH-130.md

Forbidden scope:
- crates/nako-server raw FFmpeg assembly
- crates/nako-playback policy changes
- public DTOs
- schema migrations
- request identity or artifact path format changes

Expected direction:
- Inspect current FFmpeg request, command plan, execution policy, HLS, and
  remux planning code before editing.
- Split by responsibility only where it reduces real complexity: input mapping,
  codec/filter options, mux/output options, seek/restart options, or artifact
  output shaping.
- Keep public exports curated. Server code should continue to call high-level
  execution planning Interfaces.
- Add command-plan tests that prove behavior is unchanged while internals are
  easier to extend.

Validation:
- cargo nextest run -p nako-transcode ffmpeg hls --no-fail-fast
- cargo nextest run -p nako-transcode remux --no-fail-fast
- cargo fmt --all -- --check
- git diff --check

Stop and hand back to the planner if the task requires server playback edits,
pipeline capability semantics changes, public API changes, request identity
changes, or HLS artifact format changes.
```

## Integration Notes

- `PTJCH-110`, `PTJCH-120`, and `PTJCH-130` may run in parallel after this file
  is current.
- Merge order should prefer the least shared surface first: `PTJCH-110`, then
  `PTJCH-120`, then `PTJCH-130`. Re-run the task gates after each merge.
- Do not start `PTJCH-210` or `PTJCH-220` until the first batch has either
  merged or been explicitly split into separate workstreams.
