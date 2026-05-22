# PostgreSQL Production Readiness

Status: Completed
Last updated: 2026-05-20

This workstream owns M62: moving PostgreSQL from the M61 job-lease proof into
a production-ready database backend shape.

Closeout: M62 completed on 2026-05-20 for the supported PostgreSQL backend
scope. The implementation shipped explicit SQLite/PostgreSQL runtime backend
selection, a backend-adapter facade dispatch, broad backend-neutral contract
families, PostgreSQL migration/schema parity for the supported runtime
surfaces, sanitized database diagnostics, SQLite assumption cleanup, and
repeatable SQLite/PostgreSQL verification gates. Managed Artwork PostgreSQL
parity was intentionally split to
`docs/workstreams/managed-artwork-postgresql-parity/`, which later closed with
runtime capability support.

M61 made Nako PostgreSQL-ready by introducing the `NakoDatabase` facade,
SQLite-owned adapter modules, backend-neutral job lease contract tests, and an
optional PostgreSQL proof harness. M62 turns that architectural proof into a
real backend plan and implementation lane.

Authoritative docs:

- [Design](DESIGN.md)
- [Task ledger](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)
- [Machine-readable summary](WORKSTREAM.json)

## Priority Order

1. **Contract-test matrix first** — define the backend-neutral behavior Nako
   expects before copying SQL or expanding migrations.
2. **Schema and migration parity** — make PostgreSQL schema ownership explicit
   and progressively align the tables needed by proven contracts.
3. **Runtime backend selection** — let server configuration choose SQLite or
   PostgreSQL through `NakoDatabase` without leaking adapter details.
4. **SQLite assumption cleanup** — remove facade/server assumptions that are
   only valid for SQLite URLs, row codecs, SQL clocks, JSON text, or test
   setup.
5. **Repeatable verification** — keep SQLite always-on and PostgreSQL opt-in
   with documented local/CI commands.

## Operating Rule

PostgreSQL production readiness is not a paper exercise. Each slice must add a
contract, migration parity, runtime behavior, or verification gate that makes
the final backend state more true. Avoid fake generic layers that hide
unimplemented dialect differences.
