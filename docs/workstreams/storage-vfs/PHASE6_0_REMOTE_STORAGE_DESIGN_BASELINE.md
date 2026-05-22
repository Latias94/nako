# Phase 6.0: Remote Storage and VFS Design Baseline

Status: completed.

## Goal

Define Nako's remote storage strategy before implementing a remote backend.
This phase is intentionally docs-only: it records the architecture decision,
splits M6 milestones, and audits current local-path assumptions.

## Completed Shape

- Added ADR 0016 for remote storage and VFS cache boundaries.
- Created the `storage-vfs` workstream.
- Selected WebDAV as the first remote backend preview.
- Split M6 into WebDAV backend, directory/stat cache, remote probe staging,
  remote playback policy, and stabilization phases.
- Updated roadmap, goal map, ADR index, and workstream index.

## Current Local-Path Dependency Audit

### VFS

`nako-vfs` already has the right first abstraction:

- `StorageUri` stores scheme-qualified source locators.
- `ObjectMetadata` carries kind, length, modified time, etag, fingerprint, and
  capabilities.
- `StorageBackend` exposes `stat`, `list`, `open_range`, `read_to_string`, and
  `write_string`.
- `VirtualFile` can carry a `local_path_hint`.

The gap is that `VirtualFile` has no non-local streaming body yet. A remote
backend can validate range reads, but consumers that need bytes must either get
a reader/stream abstraction or go through staging.

### Scan

`nako-library::VfsLibraryScanner` already scans through `StorageBackend::list`
and stores `StorageUri` locators. This is the least coupled path and should be
kept.

The risk is scan failure semantics. Remote listing failures should be recorded
as remote/VFS failures, not interpreted as all previously known sources
disappearing.

### Probe

`nako-library::LibraryProbeService` opens a source through VFS, then passes
`VirtualFile.local_path_hint` to `nako-media-probe`.

`nako-media-probe::FfprobeMediaProbe` currently rejects inputs without a local
path hint. Remote sources therefore need an explicit staging service before
ffprobe runs.

### Direct Play

`nako-server::app::plan_direct_play` calls `local_source_path_and_len`, which
constructs `LocalFsBackend` from the configured library root and requires a
local path. `nako-server::http` then streams the path through
`stream_local_file_response`.

Remote direct play should instead stream range-readable sources through VFS
where possible.

### Remux and HLS

`nako-server::app::remux_source` and `hls_source` also call
`local_source_path_and_len`, then pass a `PathBuf` input into `nako-transcode`.
`nako-transcode` command planning and runners currently expect FFmpeg input
paths.

Remote remux and HLS should use staging first. Passing remote URLs and secrets
directly to FFmpeg is deferred until credentials, timeout, retry, and error
mapping are designed.

## M6 Implementation Sequence

1. M6.1 WebDAV read-only VFS backend.
2. M6.2 directory and stat cache.
3. M6.3 remote probe staging.
4. M6.4 remote playback policy.
5. M6.5 remote storage stabilization.

## Design Commitments

- WebDAV comes before S3-compatible storage.
- Remote credentials use secret references and never appear in source locators.
- VFS cache state stays separate from catalog source state.
- Probe/transcode local-path requirements are explicit and handled through
  staging.
- Remote direct play prefers VFS range streaming, not local path assumptions.

## Non-Goals

- No runtime code changes in this phase.
- No WebDAV backend implementation yet.
- No database migration for VFS cache yet.
- No remote playback implementation yet.
- No OS mount integration.

## Validation

Expected coverage for this docs-only phase:

- ADR index links ADR 0016.
- Workstream index links `storage-vfs`.
- Roadmap and goal map mark M6.0 completed and M6.1 as the next implementation
  goal.
- `git diff --check` passes.
