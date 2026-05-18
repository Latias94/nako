# Workstreams

Workstreams group related milestones, TODOs, phase notes, and design context.
They are not ownership silos; they are long-running areas of architectural
attention.

## Current Workstreams

- [android-client-foundation](android-client-foundation/README.md): completed
  Android-first client foundation work, covering native Android implementation
  order, playback-first mobile scope, Public Client API connection/browse/search
  loops, playback decision/request construction, Media3 playback smoke,
  playback session boundary, and follow-on API gaps under ADR 0026.
- [android-material-expressive-ui](android-material-expressive-ui/README.md):
  active Android UI rewrite lane for the V2 Material 3 Expressive direction,
  covering dark-first dynamic color, artwork-led media surfaces, restrained
  motion, adaptive phone/tablet chrome, and clean Compose UI boundaries.
- [architecture-review-followups](architecture-review-followups/README.md):
  proposed planning and routing lane for the 2026-05-18 architecture review
  findings, covering metadata/catalog atomicity, metadata merge-policy
  unification, Media Library source-of-truth, Public Client Source Locator
  redaction, Addon side-effect seams, playback request identity, and transcode
  diagnostics follow-ups.
- [core-architecture-deepening](core-architecture-deepening/README.md):
  completed architecture-first execution lane for the fearless refactor across
  NFO import atomicity, Library scan source commits, workflow-port narrowing,
  playback/transcode profile identity, hardware capability diagnostics, Addon
  Sidecar alignment, and deletion of replaced shallow paths.
- [metadata-catalog-commit-atomicity](metadata-catalog-commit-atomicity/README.md):
  completed execution lane for deepening metadata/catalog commit consistency,
  starting with an atomic Catalog Item Graph and Search Projection commit before
  deciding whether to fold the broader metadata refresh unit of work into the
  same lane.
- [metadata-merge-policy-unification](metadata-merge-policy-unification/README.md):
  completed execution lane for unifying Canonical Metadata merge authority across
  NFO import, provider refresh, and hierarchy confirmation while keeping NFO XML
  preservation and provider breadth out of scope.
- [multi-library-hardening](multi-library-hardening/README.md): proposed
  execution lane for hardening Media Library config/database source of truth,
  startup reconciliation, and removal of remaining one-library authority
  shortcuts after the M8 correctness baseline.
- [public-client-source-locator-redaction](public-client-source-locator-redaction/README.md):
  proposed Public Client API follow-up for auditing and removing or redacting
  raw Source Locator exposure from protocol DTOs, OpenAPI, SDKs, and HTTP docs
  while preserving internal storage/playback locators.
- [addon-token-grants-side-effects](addon-token-grants-side-effects/README.md):
  completed ARF-006 follow-up for Addon Token issuance, rotation,
  Library-Scoped Addon Grants, and Taru-mediated Addon Side Effect intake before
  metadata, artwork, subtitle, or Library File Write behavior is enabled.
- [addon-protected-writes](addon-protected-writes/README.md):
  completed follow-on split from the Addon Token Grants Side Effects closeout,
  proving concrete Taru-owned Canonical Metadata `metadata_write` application
  with explicit apply outcome, Addon metadata source attribution, idempotency,
  redaction, and catalog/search refresh.
- [addon-managed-artwork-artifacts](addon-managed-artwork-artifacts/README.md):
  proposed follow-on for `artwork_write`, Artwork Candidate, Managed Artwork,
  Taru-Managed Artifact intake, fetch/cache policy, resource budgets, and safe
  diagnostics.
- [addon-library-file-write-policy](addon-library-file-write-policy/README.md):
  proposed follow-on for addon-initiated subtitle, NFO, and sidecar-asset
  Library File Write behavior through Taru target derivation, storage/VFS,
  backup policy, and redacted write reports.
- [admin-catalog-governance-read-model](admin-catalog-governance-read-model/README.md):
  completed M60 Admin API read-model work, covering a redacted catalog
  governance queue for unknown and low-confidence Media Items without changing
  the Public Client API.
- [admin-operations-read-models](admin-operations-read-models/README.md):
  completed M57-M59 Admin API read-model batch, covering redacted event outbox
  list/filter, storage staging/cache diagnostics, and sanitized server config
  diagnostics without changing the Public Client API.
- [admin-playback-runtime-diagnostics](admin-playback-runtime-diagnostics/README.md):
  completed M56 Admin API read-model work, covering safe playback runtime
  diagnostics for hardware acceleration policy/selection, FFmpeg capability
  evidence, transcode budgets, remote playback budgets, and staging cleanup
  configuration without changing the Public Client API.
- [admin-playback-session-read-model](admin-playback-session-read-model/README.md):
  completed M55 Admin API read-model work, covering safe playback session
  list/filter support for the web console without exposing transcode output
  paths or changing the Public Client API.
- [durable-job-runtime-admin-read-model](durable-job-runtime-admin-read-model/README.md):
  completed M54 server-side architecture work, covering durable job lifecycle
  centralization and the first Admin API v1 Jobs/Tasks read model.
- [nfo-backup-retention-diagnostics](nfo-backup-retention-diagnostics/README.md):
  completed M50 NFO backup retention and diagnostics work, covering bounded
  keep-latest pruning for local NFO sidecar backups, internal/admin backup
  diagnostics, and public client protocol boundary protection.
- [nfo-sidecar-backup-policy](nfo-sidecar-backup-policy/README.md): completed M49
  NFO sidecar backup policy work, covering same-directory local backup before
  forced sidecar overwrite, explicit VFS backup requests, internal backup
  diagnostics, and separation between XML preservation and storage persistence
  mechanics.
- [nfo-storage-write-policy](nfo-storage-write-policy/README.md): completed M48
  NFO storage write policy work, covering local atomic sidecar writes, explicit
  VFS write modes, internal NFO export diagnostics, and separation between XML
  preservation and storage persistence mechanics.
- [admin-web-console](admin-web-console/README.md): proposed web admin console
  planning work, covering Taru's administration-first web surface, media
  governance page families, Admin API implications, brand direction, and a
  v0.dev-oriented context document.
- [nfo-round-trip-preservation](nfo-round-trip-preservation/README.md):
  completed M47 NFO Round Trip preservation work, covering preservation-aware
  movie NFO update, unknown XML field retention, conflict reporting, forced
  export over existing sidecars, and import/export round trip preservation
  before VFS file write/link policy work.
- [catalog-hydration-lookup-deepening](catalog-hydration-lookup-deepening/README.md):
  completed M42 catalog hydration seam work, covering a workflow-level
  `CatalogHydrationPort`, hidden lookup internals, and narrower metadata/NFO
  test surfaces without public API or schema changes.
- [durable-job-recovery](durable-job-recovery/README.md): completed M41 durable
  job recovery work, covering startup recovery for unfinished queued/running
  jobs, server startup reporting, and removal of an unused old catalog search
  projection seam.
- [metadata-refresh-seam](metadata-refresh-seam/README.md): completed M40
  metadata refresh seam work, covering refresh workflow ports, provider runtime
  boundary review, fake-port behavior tests, and preservation of existing
  provider behavior.
- [repository-seam-deepening](repository-seam-deepening/README.md): completed M39
  repository seam work, covering `CatalogHydrationPort`, catalog hydration
  snapshot/lookup/commit behavior, and metadata/NFO caller-bound narrowing.
- [server-runtime-deepening](server-runtime-deepening/README.md): completed M38
  startup/runtime architecture work, covering `ServerStartupWorkflow`, startup
  reports, durable job runtime supervision, and first migration of library scan
  and metadata background jobs.
- [client-cli](client-cli/README.md): completed M37 client entrypoint work,
  covering the Apache-2.0 Rust client CLI, `taru-client` consumption, public
  API command scope, streaming request construction, token redaction, and
  dependency boundaries that keep AGPL server/internal crates out of clients.
- [client-sdk-contract](client-sdk-contract/README.md): completed M36 client
  SDK contract work, covering protocol-owned public route inventory,
  TypeScript/OpenAPI/Rust SDK inventory reuse, Apache-2.0 client boundary
  preservation, and Rust SDK streaming request builders.
- [rust-client-sdk](rust-client-sdk/README.md): completed M35 Rust client SDK
  foundation, covering the Apache-2.0 `taru-client` crate, protocol DTO reuse,
  clean dependency boundary, async JSON client methods, mock transport tests,
  route inventory checks, and SDK docs.
- [typescript-sdk-package](typescript-sdk-package/README.md): completed M34
  TypeScript SDK package hardening, covering the private `sdk/typescript`
  package, local TypeScript tooling, strict compile gate, repeatable generation
  command, and Rust generator/package sync test.
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

- SDK package publishing, client streaming/download helpers, Dart/Flutter SDK,
  Rust CLI, or concrete Flutter/web app work after the public protocol and
  first TypeScript/Rust SDK foundations stabilize.

Keep unsplit domains in `server-foundation` until a split reduces real
coordination cost. Avoid splitting merely because a domain exists conceptually.

## Standard Files

A substantial workstream should have:

- `README.md`: purpose, status, goals, non-goals, links to active phases.
- `MILESTONES.md`: ordered outcomes with deliverables and exit criteria.
- `TODO.md`: task-level checklist grouped by subsystem.
- `PHASE*.md`: phase-specific design and validation notes when needed.
