# Managed Artwork PostgreSQL Parity — Milestones

Status: Proposed
Last updated: 2026-05-20

## M0 — Scope And Split Baseline

Exit criteria:

- PGR-090 split decision is recorded.
- Existing Managed Artwork tables/repositories/server routes are inventoried.
- The first backend-neutral contract slice is selected.

## M1 — Repository Contract Slices

Exit criteria:

- Addon Artwork Candidate and Managed Artwork ingest acceptance contracts pass
  for SQLite and PostgreSQL opt-in.
- Ingest claim/commit/fail/requeue contracts pass for SQLite and PostgreSQL
  opt-in.
- Selected Artwork publication/gallery/lifecycle contracts pass for SQLite and
  PostgreSQL opt-in.

## M2 — Runtime Support Boundary

Exit criteria:

- Managed Artwork is either fully enabled on PostgreSQL or explicitly disabled
  with safe diagnostics.
- Public/Admin redaction tests pass.
- PostgreSQL verification commands are documented.
