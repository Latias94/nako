# Future-Ready Architecture Refactor

Status: Completed
Last updated: 2026-05-20

This workstream owns the next fearless architecture refactor after the
2026-05-20 architecture review. Nako has no production compatibility burden
yet, so this lane prefers the clean target architecture over preserving MVP
shortcuts, duplicate paths, or SQLite-only assumptions.

Closeout: M61 completed on 2026-05-20. The implementation shipped the
PostgreSQL-ready `nako-db` facade plus SQLite-owned adapter modules, backend
contract tests, server composition extraction, Local Inference Engine,
Metadata Candidate Graph, semantic search projection, Admin/API/generated
contract hygiene, and a final deletion sweep.

The central theme is future readiness:

- make the persistence seam PostgreSQL-ready instead of SQLite-shaped;
- keep module interfaces deep and workflow-shaped;
- split or delete shallow pass-through modules;
- keep Public Client API and Admin API read models explicit;
- prepare metadata, search, Addon Sidecar, AI automation, and playback growth
  without copying Jellyfin or Plex internals.

Authoritative docs:

- [Design](DESIGN.md)
- [Task ledger](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)
- [Machine-readable summary](WORKSTREAM.json)

## Priority Order

1. **Persistence and PostgreSQL readiness** — remove the `SqliteStore` god
   adapter shape, define real transaction/unit-of-work seams, and separate
   backend-neutral persistence contracts from SQLite implementation details.
2. **Server composition and workflow ports** — keep `NakoApp` thin by grouping
   storage, metadata, playback, automation, and admin runtimes behind deeper
   modules.
3. **Local Inference Engine** — separate source discovery from provisional
   hierarchy and Local Inference Evidence planning.
4. **Metadata Candidate Graph** — let TMDB, Douban, Bangumi, NFO, Addons, and
   future AI suggestions produce provider-neutral candidates before Canonical
   Metadata acceptance.
5. **Search semantics** — deepen `nako-search` from a shallow trait crate into
   explicit browse/search projection semantics.
6. **Admin/API/read-model hygiene and deletion sweep** — keep admin DTOs,
   generated contracts, frontend scaffolds, and obsolete code paths clean.

## Closeout Evidence

- `docs/adr/0029-postgresql-ready-persistence-boundary.md`
- `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`
- `crates/nako-db/src/contract_tests.rs`
- `crates/nako-db/src/facade.rs`
- `crates/nako-db/src/sqlite/`
- `crates/nako-db/src/postgres.rs`
- `crates/nako-server/src/app/composition.rs`
- `crates/nako-library/src/local_inference.rs`
- `crates/nako-core/src/media/candidate.rs`
- `crates/nako-search/src/lib.rs`
- `docs/workstreams/future-ready-architecture-refactor/EVIDENCE_AND_GATES.md`

Final closeout validation:

- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Operating Rule

Do not keep old and new production paths alive for comfort. A task is complete
only when the old shallow path is deleted or the next deletion owner is named
with a concrete expiry gate.
