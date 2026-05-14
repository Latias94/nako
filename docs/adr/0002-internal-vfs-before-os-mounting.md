# 0002: Build an Internal VFS Before OS Mounting

## Status

Proposed

## Context

Taru should eventually support local files, NAS paths, WebDAV, S3-compatible
stores, rclone-style remotes, and possibly other cloud-drive connectors. Treating
all sources as local paths would leak backend-specific behavior into scanning,
probing, metadata, and playback code.

## Decision

Build an internal VFS abstraction in `taru-vfs` and make library scanning,
media probing, and streaming consume that abstraction. OS-level mounting through
FUSE/WebDAV export may be added later, but it is not required for the core
server.

The VFS contract should expose:

- path normalization
- metadata listing and stat
- sequential and range reads
- stable fingerprints when available
- capability flags
- provider-aware caching, retry, and rate-limit hooks

## Consequences

- Remote storage can be supported without pretending it is a normal filesystem.
- Transcode and probe code can make explicit decisions about staging or range
  reads.
- Hard-link and soft-link features must be scoped to local/link-capable
  backends.
- The VFS layer becomes a critical performance and correctness boundary.

## Alternatives Considered

- Require users to mount remotes with rclone/FUSE first: fast to implement, but
  hides backend cost and error semantics from Taru.
- Implement OS-level FUSE first: useful later, but increases platform-specific
  complexity before the internal model is stable.

## Related Workstreams

- `docs/workstreams/server-foundation/`
