# Remote Storage Health And Circuit Breaker

Status: Active
Last updated: 2026-05-30

This workstream deepens Nako's storage/VFS resilience beyond process-local
backoff. The target is a durable, operator-visible **Storage Backend Health**
contract that scan, probe, playback staging, and diagnostics can share without
turning transient WebDAV/NAS/mount failures into hidden global stalls.

First executable task: `RSHC-020`.

Planner-approved lane: `storage-vfs`.

Read before implementation:

- `CONTEXT.md`
- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/remote-storage-health-and-circuit-breaker/CONTEXT.jsonl`

Do not implement playback staging changes or Admin reset routes before the
durable health repository contract is proven.
