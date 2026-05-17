# Workstreams

Workstreams group related milestones, TODOs, phase notes, and design context.
They are not ownership silos; they are long-running areas of architectural
attention.

## Current Workstreams

- [sdk-client-scaffold](sdk-client-scaffold/README.md): completed M33 SDK
  generation and client integration scaffold, covering a dependency-free
  TypeScript/Web/CLI SDK generator, auth/error/version handling, public route
  method inventory, and static leakage checks.
- [openapi-client-contract](openapi-client-contract/README.md): completed M32
  OpenAPI and Public Client SDK contract foundation, covering the first public
  OpenAPI v1 artifact, bearer-auth/error/version schema, route inventory, and
  leakage checks for future Flutter, web, CLI, and SDK work.
- [access-boundary-auth](access-boundary-auth/README.md): completed M31
  inbound HTTP access-boundary work, covering bearer-token auth, public/admin
  route protection, local development config, and separation from addon/
  webhook/provider outbound integration secrets.
- [public-api-contract](public-api-contract/README.md): completed M30 public
  API versioning and error envelope hardening, covering public v1
  compatibility, stable error code vocabulary, pagination/envelope rules, and
  public route evidence for future Flutter, web, CLI, and SDK clients.
- [public-client-api](public-client-api/README.md): completed M29 public
  client API contract work, covering the permissive protocol DTO expansion,
  browse/search/list/detail wire contracts, and playback decision response
  migration for future Flutter, web, and CLI clients.
- [crate-boundary-hardening](crate-boundary-hardening/README.md): completed
  M28 crate boundary and public protocol hardening, covering the permissive
  public client protocol boundary, core/module deepening, library/NFO workflow
  splits, and playback seam clarification.
- [metadata-catalog](metadata-catalog/README.md): M27 media-library domain
  expansion, covering the completed video-first domain baseline,
  schema/repository slice, local inference, provisional hierarchy,
  provider/NFO expansion, and metadata authority.
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

- client SDK generation or concrete Flutter/web app work after the public
  protocol contract stabilizes.

Keep unsplit domains in `server-foundation` until a split reduces real
coordination cost. Avoid splitting merely because a domain exists conceptually.

## Standard Files

A substantial workstream should have:

- `README.md`: purpose, status, goals, non-goals, links to active phases.
- `MILESTONES.md`: ordered outcomes with deliverables and exit criteria.
- `TODO.md`: task-level checklist grouped by subsystem.
- `PHASE*.md`: phase-specific design and validation notes when needed.
