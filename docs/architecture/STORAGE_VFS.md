# Storage And VFS Architecture

Last updated: 2026-05-30

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
| Remote storage boundary | Shipped foundation | `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/` | Broaden backend support and durable backend health/circuit breakers. |
| WebDAV read path | Partial | `docs/workstreams/storage-vfs/`; remote storage lanes | Harden retries, cache, and operator diagnostics. |
| Source locator | Shipped foundation | `CONTEXT.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/` | Watcher/debounce productization and repair workflows. |
| Source fingerprint | Shipped first slice | `CONTEXT.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/` | Optional partial/full hash escalation policy. |
| Remote probe staging | Shipped foundation | `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/` | Per-backend staging budgets and diagnostics. |
| Remote FFmpeg input staging | Shipped foundation | `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md` | Per-backend staging budgets and diagnostics. |
| VFS cache | Shipped diagnostics foundation | `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/` | Repair diagnostics and cache operator actions. |
| Library file writes | Partial | addon/library-file-write and NFO workstreams | Capability-specific write/link/backup policy. |
| Mount hang protection | Shipped first slice | `docs/workstreams/storage-vfs-resilience-and-source-identity/` | Durable backend health and circuit-breaker lane. |

## Workstream Evidence

Use `docs/architecture/WORKSTREAM_LINKS.md#storage-and-vfs` as the consolidated
index for storage/VFS workstreams. Keep this document focused on capability
state and risk, not copied task evidence.

## Completed Work Lanes

### storage-vfs-resilience-and-source-identity

Status: Completed as of 2026-05-30.

Shipped:

- layered redaction-safe **Source Fingerprint** evidence;
- strong-evidence move/rename reconciliation;
- reviewable **Source Duplicate Relationship** records;
- redaction-safe storage failure classification;
- bounded process-local read/probe/stage backoff;
- Admin diagnostics for catalog governance, VFS cache/staging cleanup pressure,
  and storage backend health.

## Next Work Lanes

- `proposed:remote-storage-health-and-circuit-breaker`: durable backend health,
  backend-specific circuit-breaker policy, and operator controls.
- `proposed:vfs-cache-repair-diagnostics`: cache repair previews, refresh
  actions, and stale-cache operator remediation.
- `proposed:source-fingerprint-escalation-policy`: opt-in partial/full hash
  escalation for ambiguous source identity cases.
- `proposed:storage-vfs-postgresql-runtime-harness`: runtime parity evidence
  for PostgreSQL storage/source identity query paths.

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
