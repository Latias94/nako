# Source Fingerprint Scheduling Architecture Map Reconciliation

## Goal

Reconcile the remaining architecture maps after the source fingerprint hash
scheduling diagnostic planner shipped, so future agents do not keep treating
the completed first slice as an open implementation lane.

## Requirements

- Update `docs/architecture/LANES.md` to reflect that source fingerprint hash
  scheduling diagnostics are shipped and that remaining work is queue/API/
  persistence/operator integration.
- Update `docs/architecture/WORKSTREAM_LINKS.md` to replace stale
  `proposed:source-fingerprint-hash-scheduling-and-diagnostics` language with
  the new queue/operator integration follow-on.
- Keep the already updated `docs/architecture/STORAGE_VFS.md` and
  `docs/architecture/LIBRARY_PIPELINE.md` wording consistent with those maps.
- Do not change Rust code, public API contracts, database schema, queue
  behavior, or source reconciliation behavior in this docs-only slice.

## Acceptance Criteria

- [ ] `LANES.md` no longer says storage-vfs is idle after only the hash
      execution kernel or recommends the completed scheduling diagnostics lane.
- [ ] `WORKSTREAM_LINKS.md` no longer lists the completed scheduling
      diagnostics first slice as the proposed next lane.
- [ ] Remaining follow-on wording points to queue-backed scan/operator
      execution, Admin/Public API exposure, persistence, and automatic
      reconciliation only as future work.
- [ ] Trellis context validates and `git diff --check` passes.

## Definition Of Done

- Docs changes are committed.
- Task evidence records verification commands.
- Task is archived.

## Technical Notes

- This follows commit `ed4036e4` and archive commit `28d0a9e3`.
- Relevant maps:
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/LANES.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
