# Workstreams

Workstreams group related milestones, TODOs, phase notes, and design context.
They are not ownership silos; they are long-running areas of architectural
attention.

## Current Workstreams

- [fearless-architecture-deepening](fearless-architecture-deepening/README.md):
  completed M63 architecture-first lane for deepening Addon Side Effect
  Modules, Addon metadata commit atomicity, Library ingestion workflow seams,
  playback/transcode request identity, hardware diagnostics, search semantics,
  and test locality before new feature breadth hardens shallow Interfaces.
- [postgresql-production-readiness](postgresql-production-readiness/README.md):
  completed M62 execution lane that turned PostgreSQL from the M61 job-lease
  proof into a production-shaped backend through backend-neutral contracts,
  migration/schema parity, runtime backend selection, SQLite assumption
  cleanup, and repeatable verification.
- [managed-artwork-postgresql-parity](managed-artwork-postgresql-parity/README.md):
  completed follow-on split from M62 PGR-090 for PostgreSQL parity across
  Managed Artwork candidates, ingest jobs, artifacts, Selected Artwork,
  galleries, lifecycle cleanup, drift/remediation diagnostics, thumbnail
  variants, and redaction-safe runtime enablement.
- [future-ready-architecture-refactor](future-ready-architecture-refactor/README.md):
  completed M61 fearless architecture refactor lane for PostgreSQL-ready
  persistence, deeper runtime/domain/search/API seams, and deletion of
  redundant MVP paths before Taru's SQLite, metadata, Addon, AI automation,
  and client contracts harden.
- [android-client-foundation](android-client-foundation/README.md): completed
  Android-first client foundation work, covering native Android implementation
  order, playback-first mobile scope, Public Client API connection/browse/search
  loops, playback decision/request construction, Media3 playback smoke,
  playback session boundary, and follow-on API gaps under ADR 0026.
- [android-client-qa-harness](android-client-qa-harness/README.md): completed
  Android client testing lane for local emulator smoke checks, screenshot
  evidence, repeatable fixture/state assumptions, and developer-friendly QA
  commands for parallel Android work.
- [android-developer-validation-entrypoint](android-developer-validation-entrypoint/README.md):
  closed Android developer validation lane for one local handoff command that
  composes JVM tests, debug assemble, and smoke regression evidence.
- [android-smoke-regression-harness](android-smoke-regression-harness/DESIGN.md):
  closed local Android smoke regression lane for composing stable emulator
  fixture states and preserving failure handoff evidence.
- [android-material-expressive-ui](android-material-expressive-ui/README.md):
  completed Android UI rewrite lane for the V2 Material 3 Expressive direction,
  covering dark-first dynamic color, artwork-led media surfaces, restrained
  motion, adaptive phone/tablet chrome, and clean Compose UI boundaries.
- [android-api-contract-integration](android-api-contract-integration/DESIGN.md):
  closed Android Public Client API integration lane for productizing and
  smoke-proving Person Detail from Cast & Crew, with broad relationship indexes
  split to `android-relationship-indexes`.
- [android-relationship-indexes](android-relationship-indexes/DESIGN.md):
  active follow-on for deciding and productizing Android People, Tags, and
  Genres index pages without local filtering or Admin/internal API use.
- [architecture-review-followups](architecture-review-followups/README.md):
  completed planning and routing lane for the 2026-05-18 architecture review
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
- [multi-library-hardening](multi-library-hardening/README.md): completed
  execution lane for hardening Media Library config/database source of truth,
  startup reconciliation, and removal of remaining one-library authority
  shortcuts after the M8 correctness baseline.
- [public-client-source-locator-redaction](public-client-source-locator-redaction/README.md):
  completed Public Client API follow-up for auditing and removing or redacting
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
  completed follow-on for the first safe `artwork_write` runtime path, covering
  MediaItem-targeted Addon Artwork Candidate proposals without exposing raw
  source URLs as public client artwork.
- [managed-artwork-ingest-selection](managed-artwork-ingest-selection/README.md):
  completed follow-on for accepting Artwork Candidates into internal
  Taru-managed ingest state through a redacted Admin API command and durable
  `managed_artwork_ingest` job, without public artwork publication.
- [managed-artwork-fetch-artifact-storage](managed-artwork-fetch-artifact-storage/README.md):
  completed follow-on for processing queued managed artwork ingest jobs through
  Taru-owned fetch/content validation and internal artifact byte storage before
  public image serving, thumbnails, or selected artwork publication.
- [managed-artwork-public-serving-selection](managed-artwork-public-serving-selection/README.md):
  completed follow-on for publishing stored Managed Artwork Artifacts as Selected
  Artwork and exposing first-party Public Client image references without
  leaking raw source URLs, cache URIs, storage URIs, or local paths.
- [managed-artwork-artifact-lifecycle-cleanup](managed-artwork-artifact-lifecycle-cleanup/README.md):
  completed follow-on for Managed Artwork Artifact lifecycle diagnostics,
  orphan cleanup dry-run, Selected Artwork retention protection, and protected
  cleanup without leaking storage URIs or local paths.
- [managed-artwork-artifact-store-drift-inventory](managed-artwork-artifact-store-drift-inventory/README.md):
  completed follow-on for bounded, redacted Admin diagnostics of drift between
  active Managed Artwork Artifact DB records and files under the local artifact
  root, without deletion or repair.
- [managed-artwork-remediation-policy](managed-artwork-remediation-policy/README.md):
  completed follow-on for redacted Managed Artwork remediation planning and
  confirmed cleanup of safe untracked artifact files, without missing-artifact
  repair or Selected Artwork management.
- [managed-artwork-thumbnail-variants](managed-artwork-thumbnail-variants/README.md):
  completed follow-on for bounded on-demand Selected Artwork image variants,
  redacted public/Admin variant contracts, and cache validators that do not
  expose artifact storage or content hashes.
- [managed-artwork-gallery-candidate-management](managed-artwork-gallery-candidate-management/README.md):
  completed follow-on for redacted Admin item-scoped artwork galleries, candidate
  comparison, and explicit Selected Artwork management without exposing raw
  candidate sources, storage handles, paths, cache URIs, or content hashes.
- [selected-artwork-unpublish-delete-policy](selected-artwork-unpublish-delete-policy/README.md):
  completed follow-on for explicit Selected Artwork unpublish behavior, Public
  Client image visibility after unpublish, and artifact retention/delete
  boundaries without exposing storage handles, paths, source URLs, cache URIs,
  or content hashes.
- [managed-artwork-ingest-runtime-controls](managed-artwork-ingest-runtime-controls/README.md):
  completed follow-on for redacted Admin retry/requeue controls around Managed
  Artwork ingest failures without conflating fetch execution, publication,
  cleanup, repair, or cancellation.
- [managed-artwork-module-deepening](managed-artwork-module-deepening/README.md):
  completed architecture follow-on for deepening Managed Artwork app/db/api
  Modules around candidates, artifacts, Selected Artwork, variants,
  lifecycle/remediation, and redaction-preserving seams without adding provider
  search, Public Client gallery, thumbnail eviction, repair/re-ingest, or new
  runtime retry/cancel semantics.
- [job-runtime-worker-control-plane](job-runtime-worker-control-plane/README.md):
  completed architecture follow-on for the first durable job worker/control-plane
  slice, covering opt-in supervised Managed Artwork ingest execution and typed
  startup recovery while splitting cancellation, generic leases, retry/backoff,
  and other job-kind migrations.
- [durable-job-ownership-leases](durable-job-ownership-leases/README.md):
  completed architecture follow-on for durable job worker identity, fenced
  ownership leases, heartbeats, cancel-request semantics, lease-aware startup
  recovery, shared leased runtime execution, and truthful redacted Admin
  cancellation requests.
- [worker-job-cancellation-checkpoints](worker-job-cancellation-checkpoints/README.md):
  completed follow-on for turning durable running-job cancel requests into
  cooperative worker-side cancellation checkpoints and fenced terminal
  acknowledgement across runtime, metadata maintenance, library scan/probe, and
  NFO app boundaries while splitting retry/backoff, lease stealing, child
  process cancellation, and per-sidecar NFO checkpoints.
- [nfo-sidecar-cancellation-checkpoints](nfo-sidecar-cancellation-checkpoints/README.md):
  completed follow-on for adding per-sidecar cooperative cancellation to NFO
  import/export service loops without making `taru-nfo` depend on server
  runtime types or mixing retry/backoff, lease policy, or child-process
  cancellation into the NFO boundary.
- [addon-library-file-write-policy](addon-library-file-write-policy/README.md):
  completed follow-on for the first addon-initiated Library File Write path,
  proving MediaSource-targeted Taru-owned NFO Export through Taru target
  derivation, storage/VFS, backup policy, redacted write reports, and
  idempotent replay.
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
- [admin-web-console](admin-web-console/README.md): completed web admin console
  baseline, covering Taru's administration-first web surface, media governance
  page families, Admin API implications, brand direction, the `apps/admin-web`
  scaffold, and the live/mock Admin API data-source boundary.
- [admin-api-typescript-contract](admin-api-typescript-contract/README.md):
  completed follow-on for generating or mechanically synchronizing the
  `/admin/v1/*` TypeScript contract consumed by `apps/admin-web` while keeping
  it separate from the Public Client SDK and `taru-client-protocol`.
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
