# Storage And VFS Architecture

Last updated: 2026-06-04

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
| Remote storage boundary | Shipped durable health foundation | `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/`; `docs/workstreams/remote-storage-health-and-circuit-breaker/` | Open follow-ons for cache repair, fingerprint escalation, playback artifact I/O pressure, scan scheduling, or PostgreSQL runtime harness work. |
| WebDAV read path | Partial | `docs/workstreams/storage-vfs/`; remote storage lanes | Harden retries, cache, and operator diagnostics. |
| Source locator | Shipped foundation | `CONTEXT.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/` | Watcher/debounce productization and repair workflows. |
| Source fingerprint | Shipped escalation policy seam | `CONTEXT.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/`; `.trellis/tasks/06-04-06-04-source-fingerprint-escalation-policy-first-slice/` | Optional hash execution, operator queue, and diagnostics remain follow-ons. |
| Remote probe staging | Shipped foundation | `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/` | Per-backend staging budgets and diagnostics. |
| Remote FFmpeg input staging | Shipped foundation | `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md` | Per-backend staging budgets and diagnostics. |
| VFS cache | Shipped diagnostics foundation plus action preview | `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/`; `.trellis/tasks/06-04-06-04-vfs-cache-repair-action-preview-first-slice/` | Executable cache refresh/remediation actions and broader previews. |
| Library file writes | Partial | addon/library-file-write and NFO workstreams | Capability-specific write/link/backup policy. |
| Mount hang protection | Shipped durable circuit foundation | `docs/workstreams/storage-vfs-resilience-and-source-identity/`; `docs/workstreams/remote-storage-health-and-circuit-breaker/` | OS-level mount stalls still need bounded adapters and operator guidance; do not claim syscall preemption. |

## Workstream Evidence

Use `docs/architecture/WORKSTREAM_LINKS.md#storage-and-vfs` as the consolidated
index for storage/VFS workstreams. Keep this document focused on capability
state and risk, not copied task evidence.

## Completed Work Lanes

### remote-storage-health-and-circuit-breaker

Status: Closed as of 2026-05-31.

Shipped:

- durable **Storage Backend Health** records and repository parity;
- runtime **Storage Circuit Breaker** admission for bounded storage work;
- redaction-safe Admin diagnostics and operator reset;
- generated Admin TypeScript contract refresh for the new DTOs and routes.

Follow-ons remain separate: cache repair, source fingerprint escalation,
playback artifact I/O scheduling, scan scheduling, and PostgreSQL runtime
harness evidence.

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

### vfs-cache-repair-diagnostics

Status: Minimal diagnostic slice shipped as of 2026-06-02; structured action
preview shipped as of 2026-06-04.

Shipped:

- VFS cache repair diagnostics classify fresh cache, stale fallback repair,
  retryable refresh failures, operator-action failures, and unknown failures;
- diagnostics are derived from existing redaction-safe storage failure classes
  and never include source locators, raw provider errors, etags, fingerprints,
  or local paths;
- Admin repair previews now include a stable `recommended_action` enum for UI
  and operator routing while preserving display-oriented `operator_action`
  prose;
- no storage schema, Admin API, playback artifact pressure, or scan scheduling
  expansion was added in this slice.

## Next Work Lanes

- `proposed:vfs-cache-repair-operator-actions`: executable refresh actions,
  stale-cache operator remediation, and broader URI-scoped previews.
- `proposed:hls-artifact-io-pressure-enforcement`: playback/storage follow-on
  for HLS segment read/write pressure, storage health coordination, and
  redaction-safe diagnostics. Open only after HLS progressive-readiness gates
  are stable.
- `proposed:source-fingerprint-hash-execution`: opt-in partial/full hash
  execution, operator queueing, and diagnostics for ambiguous source identity
  cases. The current escalation policy seam is advisory only and does not read
  source bytes.
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
- record confidence and escalation recommendations instead of forcing exact
  identity for every source.

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
