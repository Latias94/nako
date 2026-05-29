# Storage And VFS Architecture

Last updated: 2026-05-29

This document maps Nako's storage and VFS architecture for agents working on
scan, probe, playback, imports, sidecar writes, and remote storage.

## Target Chain

```text
Library Config
  -> StorageBackend / VFS capabilities
  -> Source Locator
  -> Source Fingerprint / duplicate evidence
  -> probe/scan/playback staging
  -> manifest-backed cleanup and diagnostics
```

Storage is fallible product behavior. Local disks, NAS mounts, WebDAV, SMB/NFS,
and rclone-like mounts can be slow, stale, or unavailable.

## Progress Matrix

| Capability | Status | Authority | Next Lane |
| --- | --- | --- | --- |
| Local storage backend | Shipped | `docs/adr/0002-internal-vfs-before-os-mounting.md` | Keep local behavior as the compatibility baseline. |
| Remote storage boundary | Shipped foundation | `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md` | Broaden backend support and stale-cache diagnostics. |
| WebDAV read path | Partial | `docs/workstreams/storage-vfs/`; remote storage lanes | Harden retries, cache, and operator diagnostics. |
| Source locator | Shipped foundation | `CONTEXT.md`; storage/VFS workstreams | Improve move/rename reconciliation. |
| Source fingerprint | Partial | `CONTEXT.md`; catalog/source duplicate lanes | Hash/inode/size/duration evidence policy. |
| Remote probe staging | Shipped foundation | `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md` | Timeout and partial-staging cleanup hardening. |
| Remote FFmpeg input staging | Shipped foundation | `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md` | Per-backend staging budgets and diagnostics. |
| VFS cache | Partial | `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md` | Stale cache semantics for scan and playback. |
| Library file writes | Partial | addon/library-file-write and NFO workstreams | Capability-specific write/link/backup policy. |
| Mount hang protection | Weak | This document | Open `storage-vfs-resilience-and-source-identity`. |

## Next Work Lanes

### storage-vfs-resilience-and-source-identity

Goal: Make storage failures bounded and make source identity robust across
renames, moves, stale caches, and remote storage interruptions.

Scope:

- source fingerprint policy;
- move/rename reconciliation;
- read/probe/stage timeout behavior;
- backend circuit-breaker or backoff state;
- stale cache diagnostics;
- partial staging cleanup.

Exit criteria:

- scan/probe/playback failures classify storage timeout, unavailable, rate
  limited, stale cache, and permission failures distinctly;
- a stuck remote mount cannot block unrelated libraries;
- moved files can preserve metadata/playback state when evidence is strong.

## Risk Register

### OS Mounts Can Block Like Local Files

SMB/NFS/rclone mounts often look like local paths but behave like remote
services. Treating every mounted path as safe local disk can stall scan, probe,
or playback.

Mitigation:

- isolate blocking local filesystem calls behind bounded permits;
- use timeout wrappers around probe/stage workflows;
- do not hold global locks while touching mounted paths.

### Fingerprint Policy Can Be Too Expensive

Hashing entire multi-gigabyte files during scan can hurt NAS and cloud-backed
libraries.

Mitigation:

- prefer layered evidence: size, mtime, path, duration, stream facts, partial
  hash, then full hash only when needed;
- record confidence instead of forcing exact identity for every source.

### Remote Staging Can Leak Disk

Interrupted probe or playback staging can leave large temporary files.

Mitigation:

- keep staging manifests authoritative;
- run startup cleanup;
- record ownership by library/source/session;
- expose Admin diagnostics for stale staging.

## Agent Notes

Before changing scan, probe, playback input staging, or sidecar write behavior,
read ADR 0016 and ADR 0017. Do not bypass VFS with raw `std::fs` in application
logic unless the module is explicitly a local-backend adapter.
