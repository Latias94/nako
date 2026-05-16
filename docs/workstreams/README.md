# Workstreams

Workstreams group related milestones, TODOs, phase notes, and design context.
They are not ownership silos; they are long-running areas of architectural
attention.

## Current Workstreams

- [metadata-catalog](metadata-catalog/README.md): proposed M27 media-library
  domain expansion, covering source relationships, metadata authority, NFO,
  artwork, and search breadth.
- [transcode-runtime](transcode-runtime/README.md): completed M25 playback and
  transcode runtime productization, covering playback service decomposition,
  FFmpeg-backed hardware capability probing, acceleration selection, resource
  budgets, session lifecycle, and client-facing playback contracts.
- [server-architecture-hardening](server-architecture-hardening/README.md):
  completed M24 server composition, application service, runtime supervisor,
  repository boundary, and obsolete-helper cleanup work.
- [runtime-foundation](runtime-foundation/README.md): completed M15-M19 database and
  runtime hardening, covering SQLite concurrency, migration execution, secret
  redaction, hardware capability selection, and cross-cutting operational
  boundaries.
- [playback-streaming](playback-streaming/README.md): completed M7 remote
  direct-body streaming, staging budget/cleanup, playback error mapping,
  remote playback resource budgets, and multi-library configuration work.
- [metadata-operations](metadata-operations/README.md): completed M13-M18 metadata
  maintenance jobs, diagnostics filtering, raw cache retention, and provider
  health visibility.
- [storage-vfs](storage-vfs/README.md): completed M6 remote storage, VFS cache,
  remote staging, playback policy, and WebDAV preview work.
- [addons-automation](addons-automation/README.md): completed M5 webhook,
  automation, addon manifest, provider, and trust-boundary work.
- [server-foundation](server-foundation/README.md): completed backend
  foundation, catalog, metadata, playback, transcode, VFS, and historical
  planning hub.

## When To Split A Workstream

Split a workstream when one of these becomes true:

- it has independent milestones that can progress without blocking the active
  backend foundation;
- it needs its own ADR cluster or validation matrix;
- its TODO file becomes too broad to guide the next implementation goal;
- the same docs are repeatedly edited for unrelated domains.

Expected future splits:

- `clients`

Keep unsplit domains in `server-foundation` until a split reduces real
coordination cost. Avoid splitting merely because a domain exists conceptually.

## Standard Files

A substantial workstream should have:

- `README.md`: purpose, status, goals, non-goals, links to active phases.
- `MILESTONES.md`: ordered outcomes with deliverables and exit criteria.
- `TODO.md`: task-level checklist grouped by subsystem.
- `PHASE*.md`: phase-specific design and validation notes when needed.
