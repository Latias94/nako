# Storage and VFS Workstream

## Status

Active for M6.

This workstream owns Taru's storage abstraction beyond the local filesystem:
remote VFS backends, directory/stat cache policy, remote byte-range reads,
local staging for probe/transcode, and remote playback constraints.

Top-level tracking:

- [Goal map](../../GOALS.md)
- [Roadmap](../../ROADMAP.md)
- [ADR 0002: internal VFS before OS mounting](../../adr/0002-internal-vfs-before-os-mounting.md)
- [ADR 0012: durable scan state and source tombstones](../../adr/0012-durable-scan-state-and-source-tombstones.md)
- [ADR 0016: remote storage and VFS cache boundaries](../../adr/0016-remote-storage-and-vfs-cache-boundary.md)
- [Phase 6.0 design baseline](PHASE6_0_REMOTE_STORAGE_DESIGN_BASELINE.md)

## Goals

- Make remote storage first-class without relying on OS mounts.
- Keep storage-specific code behind `taru-vfs` and staging services.
- Add a read-only WebDAV backend before S3-compatible storage.
- Persist remote directory/stat cache facts without confusing cache state with
  catalog truth.
- Stage remote objects explicitly for ffprobe, remux, and HLS when local paths
  are required.
- Preserve bounded timeout, retry, rate-limit, and concurrency behavior.

## Non-Goals

- No remote write/delete support in the first M6 slice.
- No S3-compatible backend before the WebDAV preview proves the VFS boundary.
- No OS-level mount integration.
- No adaptive bitrate ladder work; that belongs to playback-streaming later.
- No client UI work.

## Boundary Rules

- `StorageUri` remains the stable locator stored by catalog and source state.
- Remote credentials are secret references, never plaintext source locators.
- VFS directory/stat cache records are cache state, not media catalog truth.
- Probe and FFmpeg callers must not assume every source has a local path.
- Staged local files must be deterministic, bounded by disk budget, and
  validated by size/etag/fingerprint before reuse.

## Resource Classes

M6 introduces or reserves these resource classes:

- `storage.remote.list`: remote directory listing and stat refresh.
- `storage.remote.read`: remote byte-range or object reads.
- `storage.remote.stage`: local staging for probe/transcode inputs.

These classes are separate from scan, metadata, webhook, automation, addon, and
transcode CPU/GPU budgets.
