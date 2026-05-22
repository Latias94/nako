# 0016: Define Remote Storage and VFS Cache Boundaries

## Status

Proposed

## Context

Nako's local playback MVP proves scan, probe, browse, direct play, remux, HLS,
and extension surfaces for local filesystem media. The next product boundary is
remote storage. Personal media libraries commonly live on NAS, WebDAV mounts,
or S3-compatible object stores, and Nako should treat those sources as
first-class storage backends instead of pretending every source has a stable
local filesystem path.

The current `nako-vfs` abstraction already has URI schemes, metadata,
capabilities, byte ranges, and a local backend. However, several consumers still
require local paths:

- `nako-media-probe::FfprobeMediaProbe` requires `local_path_hint`.
- `nako-library::LibraryProbeService` passes `VirtualFile.local_path_hint` to
  ffprobe and cannot stage remote objects yet.
- `nako-server::app::local_source_path_and_len` constructs `LocalFsBackend`
  from the configured library root and feeds local paths into direct play,
  remux, and HLS.
- `nako-transcode` command plans accept `PathBuf` inputs for FFmpeg.
- `nako-server::http` streams local files and staged transcode outputs from
  `Path`.

This is acceptable for M4 local playback but would make remote storage fragile
if WebDAV or S3-specific logic is threaded through server handlers.

## Decision

Remote storage work will be owned by a dedicated `storage-vfs` workstream.

M6 starts with a WebDAV read-only preview before S3-compatible storage. WebDAV
is the better first target because it is common for self-hosted NAS setups,
maps naturally to hierarchical media libraries, and exercises the directory,
stat, range-read, timeout, retry, and credential boundaries Nako needs before
object-store-specific behavior.

`nako-vfs` remains the abstraction boundary for storage listing, metadata, and
range reads. Remote backends must return `StorageUri`, `ObjectMetadata`, and
`VirtualFile` records without exposing raw local filesystem paths unless an
explicit staging service has materialized a local copy.

Directory and stat cache state must be modeled as VFS/cache state, not as media
catalog truth. The catalog persists source state and tombstones; the VFS cache
persists remote listing facts, provider fingerprints, etags, cursors when
available, TTLs, and failure state. Cache staleness must not delete media
sources directly.

Probe and FFmpeg workflows must use an explicit staging boundary when a backend
cannot provide a local path. The staging service owns:

- byte-range or full-object reads from remote backends;
- deterministic local cache paths under a configured staging root;
- size/etag/fingerprint validation before reuse;
- cleanup policy and disk budget;
- timeout, retry, and bounded concurrency for remote reads.

Direct playback should prefer range streaming through VFS for remote sources.
Remux and HLS may require local staging until FFmpeg remote input support is
designed and validated.

Remote credentials must be stored as secret references, following ADR 0009.
Job inputs, scan snapshots, source records, logs, and cache records must avoid
plaintext credentials.

## Consequences

- M6 can add WebDAV without changing playback handlers into storage-specific
  code.
- The local-path requirement remains explicit instead of hidden in `PathBuf`
  plumbing.
- Probe, remux, and HLS become consumers of a staging service, which is easier
  to test and budget than ad hoc temporary downloads.
- The VFS cache can support expensive remote listings without weakening scan
  tombstone safety.
- More implementation work is needed before remote playback reaches parity
  with local playback.

## Alternatives Considered

- Mount remote storage with the operating system and keep using local paths:
  rejected because it hides remote latency, auth, retry, cache, and partial
  failure behavior from Nako.
- Implement S3 first: deferred because object-store listing and hierarchy
  semantics are less representative of personal NAS libraries than WebDAV.
- Let ffprobe/FFmpeg read remote URLs directly: deferred because credentials,
  range behavior, timeout policy, and error mapping would be delegated to
  external processes too early.
- Store remote cache data directly in media source records: rejected because
  cache staleness and catalog truth have different lifecycles.

## Related Workstreams

- `docs/workstreams/storage-vfs/`
- `docs/workstreams/server-foundation/`
