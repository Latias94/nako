# Playback Streaming Workstream

## Status

Completed for M7.

M7 owns the hardening work that makes remote playback practical after the M6
WebDAV preview: direct response-body streaming, staging disk budget and
cleanup, playback error mapping, remote playback resource budgets, and
multi-library/multi-remote backend configuration.

Top-level tracking:

- [Goal map](../../GOALS.md)
- [Roadmap](../../ROADMAP.md)
- [ADR 0016: remote storage and VFS cache boundaries](../../adr/0016-remote-storage-and-vfs-cache-boundary.md)
- [ADR 0017: playback streaming and remote hardening boundaries](../../adr/0017-playback-streaming-and-remote-hardening-boundaries.md)
- [Milestones](MILESTONES.md)
- [TODO](TODO.md)
- [Phase 7.0 design baseline](PHASE7_0_PLAYBACK_STREAMING_DESIGN_BASELINE.md)
- [Phase 7.1 remote direct body streaming](PHASE7_1_REMOTE_DIRECT_BODY_STREAMING.md)
- [Phase 7.2 staging manifest foundation](PHASE7_2_STAGING_MANIFEST_FOUNDATION.md)
- [Phase 7.3 playback error mapping](PHASE7_3_PLAYBACK_ERROR_MAPPING.md)
- [Phase 7.4 remote playback resource budgets](PHASE7_4_REMOTE_PLAYBACK_RESOURCE_BUDGETS.md)
- [Phase 7.4.1 NFO storage boundary](PHASE7_4_1_NFO_STORAGE_BOUNDARY.md)
- [Phase 7.5 multi-library backends](PHASE7_5_MULTI_LIBRARY_BACKENDS.md)
- [Phase 7.6 stabilization audit](PHASE7_6_STABILIZATION_AUDIT.md)

## Goals

- Stream remote direct-play response bodies without buffering selected ranges
  into memory.
- Add a staging manifest, disk budget, and cleanup worker for remote probe,
  remux, and HLS inputs.
- Map playback and storage failures into stable HTTP/API error categories.
- Add resource classes for remote playback stream and staging work.
- Replace the single-library WebDAV preview shape with explicit multi-library
  and multi-remote backend configuration.
- Keep playback handlers thin and keep backend-specific logic behind `taru-vfs`
  and application services.

## Non-Goals

- No remote write/delete support.
- No direct FFmpeg remote URL input until a separate credential and timeout
  design is accepted.
- No adaptive bitrate HLS ladder in the first M7 slice.
- No client UI work before server playback contracts stabilize.
- No S3-compatible backend unless it is needed to validate the multi-backend
  configuration model after WebDAV hardening.

## Boundary Rules

- `taru-vfs` owns storage capability checks, remote byte access, backend
  retry/timeout behavior, and staging primitives.
- `taru-server::app` owns playback decisions, resource-budget acquisition,
  staging manifest coordination, and API-safe error categories.
- `taru-server::http` should only translate an app plan into HTTP headers and
  response bodies.
- `taru-transcode` should keep FFmpeg command planning independent from remote
  backend credentials.
- Staging paths and logs must never include plaintext credentials.

## Resource Classes

M7 introduces or reserves these resource classes:

- `playback.remote.stream`: direct playback response bodies backed by remote
  range reads.
- `playback.remote.stage`: remote reads that materialize local probe/remux/HLS
  inputs.
- `playback.remote.cleanup`: staging manifest cleanup and disk-budget
  enforcement, if cleanup needs explicit throttling.

These are distinct from `storage.remote.list`, `storage.remote.read`,
`storage.remote.stage`, webhook, automation, addon, and transcode CPU/GPU
budgets.
