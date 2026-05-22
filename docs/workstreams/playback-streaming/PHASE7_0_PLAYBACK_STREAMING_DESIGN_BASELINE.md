# Phase 7.0: Playback Streaming Design Baseline

Status: completed.

## Goal

Define Nako's playback-streaming and remote hardening strategy before changing
runtime code. This phase is intentionally docs-only: it records the
architecture decision, splits M7 milestones, and moves the M6 deferred
playback tasks into a dedicated workstream.

## Completed Shape

- Added ADR 0017 for playback streaming and remote hardening boundaries.
- Created the `playback-streaming` workstream.
- Split M7 into remote body streaming, staging manifest/cleanup, playback error
  mapping, remote playback resource budgets, multi-library config, and
  stabilization phases.
- Updated roadmap, goal map, ADR index, and workstream index.

## M6 Starting Point

M6 left Nako in a useful preview state:

- WebDAV can list, stat, scan, probe, direct-play, remux, and HLS through VFS
  boundaries.
- Directory/stat cache state is separate from catalog source truth.
- Probe, remux, and HLS can stage remote objects before local-path-only tools
  run.
- WebDAV preview configuration resolves credentials from environment
  references.

The remaining gaps are now playback hardening concerns:

- Direct remote playback buffers selected range bytes in memory.
- Remote remux and HLS stage full objects without a persistent manifest or disk
  budget.
- Staging cleanup is not yet modeled as an auditable service.
- Playback error mapping is too coarse for remote storage failure modes.
- Remote playback stream/stage work is not separately budgeted.
- Multi-library and multi-remote backend configuration is not first-class.

## M7 Implementation Sequence

1. M7.1 remote direct body streaming.
2. M7.2 staging manifest, disk budget, and cleanup.
3. M7.3 playback error taxonomy and HTTP mapping.
4. M7.4 remote playback resource budgets.
5. M7.5 multi-library and multi-remote backend configuration.
6. M7.6 playback streaming stabilization.

## Design Commitments

- Keep backend-specific byte access behind `nako-vfs`.
- Keep HTTP handlers as translation layers from app plans to responses.
- Stream remote direct-play bodies before broad remote playback testing.
- Add staging budget and cleanup before expanding remote transcode usage.
- Keep FFmpeg on local staged inputs until direct remote FFmpeg inputs have a
  separate accepted design.
- Make multi-library config explicit rather than extending the single
  `[library.webdav]` preview shape ad hoc.

## Non-Goals

- No runtime code changes in this phase.
- No S3-compatible backend yet.
- No direct remote URL input to FFmpeg yet.
- No adaptive bitrate ladder yet.
- No client UI work.

## Validation

Expected coverage for this docs-only phase:

- ADR index links ADR 0017.
- Workstream index links `playback-streaming`.
- Roadmap and goal map mark M7.0 completed and M7.1 as the next implementation
  goal.
- `git diff --check` passes.
