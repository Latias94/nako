# Managed Artwork PostgreSQL Parity — Milestones

Status: Completed
Last updated: 2026-05-20

## M0 — Scope And Split Baseline

Exit criteria:

- [x] PGR-090 split decision is recorded.
- [x] Existing Managed Artwork tables/repositories/server routes are
  inventoried.
- [x] The first backend-neutral contract slice is selected.

## M1 — Repository Contract Slices

Exit criteria:

- [x] Addon Artwork Candidate and Managed Artwork ingest acceptance contracts pass
  for SQLite and PostgreSQL opt-in.
- [x] Ingest claim/commit/fail/requeue contracts pass for SQLite and PostgreSQL
  opt-in.
- [x] Selected Artwork publication/gallery/lifecycle contracts pass for SQLite and
  PostgreSQL opt-in.

## M2 — Runtime Support Boundary

Exit criteria:

- [x] Managed Artwork is either fully enabled on PostgreSQL or explicitly disabled
  with safe diagnostics.
- [x] Public/Admin redaction tests pass.
- [x] PostgreSQL verification commands are documented.
