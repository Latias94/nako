# Managed Artwork PostgreSQL Parity

Status: Completed
Last updated: 2026-05-20

This completed follow-on owns PostgreSQL parity for the full Managed Artwork
subsystem that M62 deliberately split out of `postgresql-production-readiness`.

The M62 PostgreSQL lane proves the core media, metadata, playback, event,
Addon, Automation, and runtime repository families needed to make PostgreSQL a
production-shaped backend. Managed Artwork is larger than a single repository
slice: it spans Addon Artwork Candidates, ingest jobs, artifact storage,
Selected Artwork publication, galleries, lifecycle cleanup, drift diagnostics,
remediation, thumbnail variants, and redacted public/Admin serving. Keeping it
as an explicit follow-on prevents partial PostgreSQL enablement while preserving
M62 closeout truthfulness.

Authoritative docs:

- [Design](DESIGN.md)
- [Task ledger](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)
- [Machine-readable summary](WORKSTREAM.json)

Parent decision:

- `docs/workstreams/postgresql-production-readiness/` PGR-090
