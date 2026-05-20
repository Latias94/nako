# Managed Artwork PostgreSQL Parity Design

Status: Proposed
Last updated: 2026-05-20

## Why This Lane Exists

Managed Artwork already has a mature SQLite-backed runtime and several closed
feature lanes. PostgreSQL production readiness reached the point where core
repository families can be proven through backend-neutral contracts, but Managed
Artwork remains SQLite-only above the PostgreSQL test store.

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
