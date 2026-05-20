# Managed Artwork PostgreSQL Parity — Handoff

Status: Completed
Last updated: 2026-05-20

## Current State

This lane was opened as the explicit follow-on from PGR-090 in the PostgreSQL
Production Readiness workstream and is now closed.

PostgreSQL Managed Artwork parity is implemented for the existing runtime
model: Addon Artwork Candidates, legacy Artwork Tasks, Managed Artwork ingest
jobs, artifacts, Selected Artwork, gallery snapshots, lifecycle cleanup, and
capability-driven worker enablement.

## Next Recommended Action

No MAPG work remains. If follow-on work is needed, open a new lane rather than
reopening this one:

1. Add repeatable PostgreSQL CI service orchestration for ignored contract
   gates.
2. Harden image-processing policy or artifact-store backend choices if product
   requirements change.
3. Add performance/operational tuning for high-volume Managed Artwork
   galleries and cleanup.

## Blockers

None known.

## Notes

- PostgreSQL parity was proven with an ephemeral local PostgreSQL 17 cluster and
  ignored `postgres_managed_artwork_contract` nextest cases.
- Runtime capability is now capability-driven; the server no longer blocks
  Managed Artwork ingest workers just because the configured backend is
  PostgreSQL.
- Admin/Public DTO redaction remains enforced by existing API and server tests.
