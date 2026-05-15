# Storage and VFS Milestones

## M6.0: Remote Storage and VFS Design Baseline

Outcome: Taru has a documented remote-storage plan, local-path dependency
audit, and milestone split before adding a remote backend.

Status: completed.

Deliverables:

- ADR 0016 for remote storage and VFS cache boundaries.
- Dedicated `storage-vfs` workstream.
- Local-path dependency audit for VFS, scan/probe, direct play, remux, and HLS.
- M6 milestone split and validation strategy.
- Roadmap and goal map updates.

Exit criteria:

- WebDAV is selected as the first remote backend preview.
- Probe/transcode local-path requirements are explicit.
- VFS cache and catalog truth are separated in the design.
- Docs-only validation passes.

## M6.1: WebDAV Read-Only VFS Backend

Outcome: Taru can list, stat, and range-read media objects from one configured
WebDAV backend without storing plaintext credentials.

Status: completed.

Deliverables:

- WebDAV backend configuration with secret references.
- `taru-vfs` WebDAV backend implementing `stat`, `list`, and `open_range`.
- Conservative timeout, retry, and rate-limit policy.
- Tests with a mocked local WebDAV server.

Exit criteria:

- WebDAV `stat`, `list`, and `open_range` work against a mocked local WebDAV
  server.
- A WebDAV directory can be scanned through `VfsLibraryScanner`.
- Range reads do not require a local path hint.
- Credentials are omitted from source locators and resolved only through secret
  references.

## M6.2: Directory and Stat Cache

Outcome: Remote directory and object metadata can be cached and refreshed
without making scan tombstones destructive on transient remote failures.

Status: completed.

Deliverables:

- Cache records for URI, kind, size, modified time, etag/fingerprint, TTL, and
  last error.
- Refresh policy for expensive listings and rate-limited backends.
- Tests for cache reuse, invalidation, and transient failure behavior.

Exit criteria:

- Cache staleness is visible and auditable.
- Catalog source state remains separate from VFS cache state.
- Transient remote failures do not tombstone sources as missing files.

## M6.3: Remote Probe Staging

Outcome: ffprobe can run against remote sources through explicit local staging.

Status: completed.

Deliverables:

- Staging service for remote probe inputs.
- Deterministic staging paths and reuse validation by size/etag/fingerprint.
- Disk budget and cleanup policy.
- Tests proving remote sources can be probed without `LocalFsBackend`.

Exit criteria:

- `FfprobeMediaProbe` receives a local staged path for remote inputs.
- Staging failures are safe, retryable where appropriate, and inspectable.

## M6.4: Remote Playback Policy

Outcome: Remote sources have explicit direct-play, remux, and HLS behavior.

Status: completed.

Deliverables:

- VFS-backed direct range streaming for remote sources where possible.
- Remote remux/HLS staging policy for FFmpeg inputs.
- Initial playback decision boundary for remote storage capabilities.
- Tests for range streaming, remux staging, and HLS staging.

Exit criteria:

- Direct play does not require a raw local path for remote range-readable
  sources.
- Remux/HLS use staging instead of passing remote credentials to FFmpeg.
- Remaining production WebDAV config and remote failure surfacing gaps are
  documented for M6.5.

## M6.5: Remote Storage Stabilization

Outcome: The remote-storage preview is documented, bounded, and ready for user
testing.

Status: proposed.

Deliverables:

- Local setup docs for WebDAV preview.
- HTTP API and known-limitation updates.
- Validation matrix for scan, probe, direct play, remux, HLS, cache, and
  credential handling.

Exit criteria:

- Workspace validation gates pass.
- Known M6 limitations are documented.
- The next implementation goal is explicit.
