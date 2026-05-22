# 0017: Define Playback Streaming and Remote Hardening Boundaries

## Status

Proposed

## Context

M6 proved the first remote-storage slice: WebDAV read-only access, VFS
directory/stat cache, probe staging, remote playback policy, and preview
configuration. That work intentionally left several production hardening gaps
outside the `storage-vfs` milestone:

- remote direct play can use VFS byte ranges, but the selected range is still
  materialized as an in-memory `Vec<u8>` before the HTTP response body;
- remux and HLS stage full remote objects before FFmpeg receives a local input
  path;
- deterministic staging paths exist, but there is no persistent staging
  manifest, disk budget, or cleanup worker;
- playback/storage failures are mapped coarsely at the HTTP boundary;
- resource budgets exist for FFmpeg CPU/GPU work, but not for remote direct
  stream bodies or remote staging reads;
- server configuration supports a single library with a WebDAV preview overlay,
  not multiple libraries or multiple remote backend instances.

These gaps now cut across storage, playback, transcode, server configuration,
HTTP error behavior, and validation. Keeping them only in `storage-vfs` would
make that workstream too broad.

## Decision

Remote playback hardening will move into a dedicated
`playback-streaming` workstream for M7.

`nako-vfs` remains the storage boundary for object metadata, capability checks,
range reads, staging, and backend-specific retry/timeout behavior. Server
handlers must not grow WebDAV-specific or future S3-specific branches.

Direct playback will evolve from materialized range bytes toward an async body
stream abstraction. The application layer should still produce response
metadata such as status, content range, content length, content type, and
entity validators, but the HTTP layer should be able to stream the selected
remote byte range without buffering the whole selected range in memory.

Remote remux and HLS will continue to use local staging before FFmpeg until a
separate ADR validates direct remote FFmpeg inputs. Before expanding remote
transcode use, Nako must add a staging manifest and cleanup boundary. The
manifest should track at least source URI, staging purpose, local path, size,
fingerprint or etag, state, last access time, expiration, and validation
evidence. Cleanup must enforce a configured disk budget and run on startup and
as a bounded background task.

Playback-facing error mapping will become typed enough to distinguish local
missing files, remote not found, authentication/authorization failure, timeout,
transient backend failure, stale-cache fallback, range unsupported, staging
budget exhaustion, staging validation mismatch, and FFmpeg failure. HTTP
responses should expose stable error codes without leaking credentials or raw
backend internals.

Remote playback work will introduce playback-specific resource classes. These
are separate from VFS listing/stat refresh classes and from FFmpeg CPU/GPU
classes:

- `playback.remote.stream`: direct HTTP response bodies sourced from remote
  range reads;
- `playback.remote.stage`: remote reads that materialize local probe, remux, or
  HLS inputs;
- `playback.remote.cleanup`: staging manifest cleanup and disk-budget
  enforcement, if cleanup needs explicit throttling.

Multi-library and multi-remote backend configuration must become an explicit
configuration model instead of scaling the current `[library.webdav]` preview
overlay. The model should allow multiple named libraries, backend-specific
configuration blocks, secret references, per-backend timeout/retry policy, and
stable source URI roots.

## Consequences

- M7 has an explicit implementation track for remote playback hardening rather
  than continuing to overload the M6 storage workstream.
- Direct play can become memory-bounded for large remote ranges.
- Staging cleanup and disk limits become inspectable instead of depending on
  ad hoc deterministic paths.
- Playback clients will receive more precise and stable failure responses.
- The configuration model can grow toward multiple libraries without baking a
  one-off WebDAV preview shape into long-term APIs.
- Additional application and test plumbing is required before remote playback
  can be considered production-ready.

## Alternatives Considered

- Keep remote playback hardening in `storage-vfs`: rejected because streaming
  bodies, transcode staging, HTTP errors, and multi-library config cross beyond
  storage ownership.
- Pass remote URLs and credentials directly to FFmpeg now: deferred because it
  would delegate credential handling, retry, timeout, and error mapping to an
  external process before Nako owns those boundaries.
- Rely on OS mounts for remote playback: rejected for the same reason as ADR
  0016; mounts hide remote failure modes and resource costs from Nako.
- Build client UI first: deferred until server playback contracts and media
  URLs are stable enough for clients to consume.

## Related Workstreams

- `docs/workstreams/playback-streaming/`
- `docs/workstreams/storage-vfs/`
- `docs/workstreams/server-foundation/`
