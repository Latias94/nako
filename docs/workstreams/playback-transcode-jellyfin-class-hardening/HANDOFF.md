# Playback Transcode Jellyfin-Class Hardening - Handoff

Status: Active
Last updated: 2026-05-31
Current tasks: `PTJCH-120`, `PTJCH-130`

## Current State

This workstream has been opened to coordinate playback/transcode hardening
before parallel Rust implementation starts.

The initial design records six seams:

- Playback Capability Module.
- Transcode Pipeline Capability Module.
- FFmpeg Adapter Module.
- HLS Artifact Authority Module.
- Playback Runtime Module.
- Artifact I/O Policy Module.

No Rust code has been changed by the workstream opening task.

`PTJCH-010` is complete. The seam map, owned/shared scopes, stop conditions,
architecture links, and docs/planning gates are recorded.

`PTJCH-020` is complete. `WORKER_PROMPTS.md` contains ready-to-use prompts,
suggested worktree branches, and validation gates for the first parallel batch.

`PTJCH-110` is complete and merged as commit `0d3bd96f`.

## Next Action

Run the first parallel batch from `WORKER_PROMPTS.md`:

- `PTJCH-120` - Transcode Pipeline Capability.
- `PTJCH-130` - FFmpeg Adapter split.

## Stop Conditions

Return to the planner before implementation if a task needs to:

- Change public API DTOs.
- Change request identity or artifact path format.
- Add schema migrations.
- Move ownership across crate boundaries.
- Put raw FFmpeg argument assembly outside `nako-transcode`.
- Add raw `tokio::spawn` playback/transcode/artifact work without ADR 0053
  review.
- Edit shared server playback files while another playback runtime/artifact
  lane is active.

## Suggested First-Batch Branches

- `work/ptjch-110-playback-capability`
- `work/ptjch-120-transcode-pipeline-capability`
- `work/ptjch-130-ffmpeg-adapter-split`

Do not create these branches until `PTJCH-020` records the final worker prompts
and the planner confirms whether separate worktrees are desired.
