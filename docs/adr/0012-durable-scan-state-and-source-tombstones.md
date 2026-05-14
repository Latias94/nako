# 0012: Persist Scan State and Source Tombstones

Status: accepted

## Context

Large local libraries and future remote libraries need scans to be repeatable,
auditable, and incremental. A scan that only lists files and upserts media
sources cannot distinguish unchanged files, missing files, and transient VFS
errors.

## Decision

Each index run creates a durable scan snapshot. Directory listings are recorded
as directory snapshots, and media file state is recorded as source state with
URI, source ID, size, modified timestamp, etag, fingerprint, last seen scan ID,
and tombstone status.

Missing sources are tombstoned instead of deleted immediately. Local VFS exposes
a lightweight size/mtime fingerprint as the first incremental signal; backends
can provide stronger content or provider fingerprints later.

## Consequences

- Repeated scans are idempotent and can report disappeared sources.
- Remote storage can later plug in etag, cursor, and expensive-listing rules.
- Media items and sources remain queryable after a file disappears, which helps
  audit, repair, and future rename detection.
- The first implementation does not do full rename/move detection yet.

## Alternatives Considered

- Delete missing media sources immediately: rejected because it loses audit
  state and makes transient storage failures destructive.
- Hash every file during scan: rejected because it is too expensive for large
  libraries and remote storage.

## Related Workstreams

- Server Foundation Phase 3.6
- Server Foundation Phase 4.0
