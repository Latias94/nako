# Playback Transcode Jellyfin-Class Hardening - Handoff

Status: Active
Last updated: 2026-06-01
Current tasks: `PTJCH-310`

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

`PTJCH-120` is complete and merged as commit `9f841951`.

`PTJCH-130` is complete and merged as commit `bb3835e0`.

`PTJCH-210` is complete. HLS artifact authority remains in
`crates/nako-transcode/src/artifact.rs`: request variant identity reconstructs
the artifact manifest, and manifest sequence patterns now define the
serveable allow-list for playlists, media-group playlists, segments, init
files, audio sidecars, and subtitle sidecars. See
`worker-notes/PTJCH-210.md`.

`PTJCH-220` is complete. Playback Runtime now owns the HLS supersede flow more
explicitly: candidate discovery/cancellation is centralized in runtime
control, supersede admission waits briefly for the replaced local runner to
release its permit, and active HLS playback sessions linked to superseded
transcodes are marked cancelled. The required `nako-server` `hls playback`
gate passed with 153 tests. See `worker-notes/PTJCH-220.md`.

## Next Action

Run `PTJCH-310` - Artifact I/O decision.

Decide whether HLS artifact I/O pressure remains inside this coordination
workstream or should split to a dedicated PAIP follow-on. Keep FFmpeg command
planning and HLS artifact allow-list ownership in `nako-transcode`.

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
