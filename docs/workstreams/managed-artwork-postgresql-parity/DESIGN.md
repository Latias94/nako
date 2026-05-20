# Managed Artwork PostgreSQL Parity Design

Status: Completed
Last updated: 2026-05-20

## Why This Lane Exists

Managed Artwork already had a mature SQLite-backed runtime and several closed
feature lanes. PostgreSQL production readiness reached the point where core
repository families could be proven through backend-neutral contracts, but
Managed Artwork remained SQLite-only above the PostgreSQL test store until this
lane closed.

PGR-090 in M62 decided not to absorb Managed Artwork parity into the M62
closeout because the subsystem is not a small schema tail. It crosses durable
jobs, Addon Side Effects, artifact storage, Selected Artwork public identity,
Admin diagnostics, cleanup/remediation policy, and thumbnail serving.

## Target State

- Backend-neutral contracts cover the Managed Artwork repository behaviors that
  must be supported when PostgreSQL is selected.
- PostgreSQL schema/migrations cover Addon Artwork Candidates, Managed Artwork
  Ingest, Managed Artwork Artifacts, Selected Artwork, gallery/lifecycle views,
  cleanup/remediation state, and any required artifact metadata.
- Server runtime either supports Managed Artwork on PostgreSQL end-to-end or
  disables Managed Artwork routes/workers with explicit safe diagnostics until
  parity is complete.
- Redaction guarantees remain intact: no storage URI, `managed-artwork://...`,
  local path, raw source URL, cache URI, content hash, or secret is leaked by
  public/Admin DTOs.
- Local and optional PostgreSQL verification gates are documented.

## In Scope

- `ArtworkCandidateRepository` parity.
- `ManagedArtworkRepository` parity.
- PostgreSQL schema/migration design for Managed Artwork tables.
- Backend-neutral DB contracts for candidate acceptance, ingest claim/commit,
  fail/requeue, selected artwork publish/unpublish, gallery snapshots,
  lifecycle cleanup, and redacted remediation inputs where repository-owned.
- Server behavior for enabling/disabling Managed Artwork under PostgreSQL.

## Out Of Scope

- New provider artwork search/scraping.
- New image processing formats or thumbnail eviction policy.
- Changing the public Selected Artwork identity model.
- Moving artifact bytes into PostgreSQL.

## Architecture Notes

- Keep artifact bytes in the Taru-owned artifact store; PostgreSQL owns records
  and coordination state, not binary storage.
- Do not copy SQLite migration text into PostgreSQL. Use native `uuid`,
  `boolean`, `jsonb` where structured JSON is queried, and `timestamptz` for
  SQL-owned clocks.
- Contracts should use repository and server public boundaries, not direct SQL
  inspection.
- If full parity is not implemented in one pass, PostgreSQL runtime must remain
  honest by reporting Managed Artwork as unsupported/disabled instead of
  partially enabling routes or workers.

## Closeout State

MAPG closed by implementing PostgreSQL parity for the existing Managed Artwork
runtime state:

- PostgreSQL migration `0002_managed_artwork.sql` now owns `artwork_tasks`,
  `addon_artwork_candidates`, `managed_artwork_ingests`,
  `managed_artwork_artifacts`, and `selected_artworks`.
- `PostgresStore` implements `ArtworkTaskRepository`,
  `ArtworkCandidateRepository`, and `ManagedArtworkRepository` without
  unsupported stubs.
- Backend-neutral contracts prove Addon Artwork Candidate intake, legacy
  Artwork Task queue round-trips, candidate acceptance and durable ingest/job
  creation, ingest claim/commit/fail/startup-recovery/requeue, Selected
  Artwork publication, gallery snapshots, and lifecycle cleanup behavior on
  SQLite and PostgreSQL.
- `DatabaseBackendCapabilities::postgres_supported_scope().managed_artwork` is
  true, so server runtime can opt into the Managed Artwork ingest worker under
  PostgreSQL after startup migration.
- Existing Admin/Public redaction tests continue to prove that raw source
  URLs, storage handles, `managed-artwork://...`, cache URIs, local paths, and
  content hash values are not exposed by DTOs.

## MAPG-010 Inventory Baseline

The existing SQLite implementation is authoritative for Taru behavior but not
for PostgreSQL SQL text. The inventory found these persistence surfaces:

- `ArtworkTaskRepository` persists legacy image task rows in `artwork_tasks`.
  It is not the current Managed Artwork ingest queue, but PostgreSQL should not
  keep a facade stub once Managed Artwork is promoted.
- `ArtworkCandidateRepository` owns Addon Artwork Candidate creation,
  source-level idempotency, status transitions, and item listing over
  `addon_artwork_candidates`.
- `ManagedArtworkRepository` owns candidate acceptance, durable
  `managed_artwork_ingests` job coupling, claim/commit/fail/requeue/recovery,
  artifact records, Selected Artwork publication, gallery snapshots, lifecycle
  cleanup, and redaction-safe summaries over `managed_artwork_ingests`,
  `managed_artwork_artifacts`, and `selected_artworks`.
- Server runtime routes and workers call only repository/service boundaries.
  Artifact bytes remain in `LocalManagedArtworkArtifactStore`; PostgreSQL only
  stores opaque records and coordination state.
- Admin/public DTOs already redact source/storage/cache locators by exposing
  booleans, IDs, dimensions, media type, and public image routes instead of raw
  `source_uri`, `storage_uri`, `managed-artwork://...`, local paths, or content
  hashes.

First contract slice: implement PostgreSQL schema and backend-neutral contracts
for Addon Artwork Candidate intake and Managed Artwork candidate acceptance /
ingest queue creation before enabling claim/commit or runtime capability.
