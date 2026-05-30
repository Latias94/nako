# Storage/VFS Resilience And Source Identity

Status: Completed
Last updated: 2026-05-30

This workstream deepens Nako's storage/VFS and **Media Source** identity
behavior before larger remote-library, watch-folder, provider, and sidecar-write
work depends on today's loose fingerprint and failure semantics.

The lane is intentionally non-Web and non-HLS. It may touch playback input
staging only where the shared VFS/source identity contract requires it, and any
HLS-specific runtime behavior stays with the active HLS lanes.

## Purpose

Nako already has a good VFS foundation, staging manifests, source tombstones,
and **Source Duplicate Relationship** records. The remaining problem is that
source identity evidence and storage failure behavior are still too shallow:

- **Source Fingerprint** values are optional strings produced by adapters or
  tests, not a host-owned evidence policy.
- move/rename reconciliation is still a documented pressure, not a workflow.
- OS mounts and remote backends can stall or fail in ways that are not yet
  classified consistently across scan, probe, NFO, staging, and diagnostics.
- stale-cache, timeout, rate-limit, permission, and partial-staging behavior
  should be observable without leaking **Source Locators** or host paths.

## Architecture Links

- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/adr/0002-internal-vfs-before-os-mounting.md`
- `docs/adr/0012-durable-scan-state-and-source-tombstones.md`
- `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`
- `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`

## Final Status

SVRS-020 through SVRS-060 are complete. The lane is closed.

The shipped first slice deepened source identity evidence, move/rename
reconciliation, duplicate-source review state, storage failure classification,
bounded process-local read/probe/stage backoff, and redaction-safe Admin
diagnostics.

Follow-on scope is split to proposed lanes for watcher/debounce,
backend-specific circuit breakers, expensive hash escalation, and PostgreSQL
runtime harness coverage. Do not reopen this lane for HLS/runtime feature work.
