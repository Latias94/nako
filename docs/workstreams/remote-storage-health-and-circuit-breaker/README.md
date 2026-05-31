# Remote Storage Health And Circuit Breaker

Status: Closed
Last updated: 2026-05-31

This workstream deepens Nako's storage/VFS resilience beyond process-local
backoff. The target is a durable, operator-visible **Storage Backend Health**
contract that scan, probe, playback staging, and diagnostics can share without
turning transient WebDAV/NAS/mount failures into hidden global stalls.

Closed result: durable **Storage Backend Health** records, repository parity,
runtime **Storage Circuit Breaker** admission, redaction-safe Admin
diagnostics, and operator reset are shipped.

Planner-approved lane: `storage-vfs`.

Read before implementation:

- `CONTEXT.md`
- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/remote-storage-health-and-circuit-breaker/CONTEXT.jsonl`

Follow-ons for cache repair, source fingerprint escalation, playback artifact
I/O scheduling, scan scheduling, and PostgreSQL runtime harness work should use
new planner-approved workstreams.
