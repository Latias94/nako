# Architecture State Reading

## Sources

- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `.trellis/tasks/archive/2026-06/06-07-vfs-cache-repair-job-diagnostics-projection/prd.md`
- `.trellis/tasks/archive/2026-06/06-07-vfs-cache-repair-job-diagnostics-projection/evidence.md`

## Findings

- The shipped diagnostics projection fulfilled the older generic follow-on for
  VFS cache repair job diagnostics on Admin Job list/detail/cancel shapes.
- The projection is intentionally narrow: it exposes redaction-safe job status,
  typed safe summaries, and stable redacted failure facts only for
  `JobKind::VfsCacheRepair`.
- Raw `input_json`, `summary_json`, durable `error`, `StorageUri`, local path,
  backend URL, credentials, etags, fingerprints, URI digests, and cache payload
  material remain private.
- `STORAGE_VFS.md` still lists "broader operator diagnostics" as a VFS cache
  follow-on even though the repair-job projection is now shipped.
- `CONTROL_PLANE.md` still lists "broader operator diagnostics for repair jobs"
  as a follow-on under the VFS repair durable job section.
- `WORKSTREAM_LINKS.md` still has `proposed:vfs-cache-repair-diagnostics`,
  which now collides with the archived completed diagnostics task.

## Decision

Update the maps to name the shipped projection explicitly and narrow future
follow-ons to realtime diagnostics, incident bundles, automated repair policy,
and destructive/configuration workflows. This preserves the redaction boundary
and avoids claiming broader runtime observability than the code currently
provides.
