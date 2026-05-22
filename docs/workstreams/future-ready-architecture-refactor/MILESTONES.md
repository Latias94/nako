# Future-Ready Architecture Refactor — Milestones

Status: Completed
Last updated: 2026-05-20

## M0 — Scope, Priority, And Evidence Freeze

Status: completed.

Exit criteria:

- Workstream docs exist and agree.
- Priorities are ordered.
- Non-goals and deletion rules are explicit.
- First executable task is chosen.

Primary evidence:

- `docs/workstreams/future-ready-architecture-refactor/DESIGN.md`
- `docs/workstreams/future-ready-architecture-refactor/TODO.md`

## M1 — Persistence And PostgreSQL-Ready Architecture

Status: completed.

Exit criteria:

- Current persistence shape is inventoried.
- Target persistence architecture is selected and documented.
- ADR changes are drafted or updated if crate boundaries, migration policy, or
  transaction semantics change.
- Backend-neutral persistence contract tests exist.
- SQLite implementation passes the contract tests.
- PostgreSQL readiness proof is implemented or split into a follow-on with
  clear gates.

Primary gates:

- `cargo check -p nako-core --tests`
- `cargo check -p nako-db --tests`
- `cargo nextest run -p nako-db --no-fail-fast`
- `git diff --check`

## M2 — Runtime And Domain Seam Deepening

Status: completed.

Exit criteria:

- `NakoApp` delegates construction to cohesive runtime modules where the
  modules hide real policy/construction complexity.
- Local Inference is separated from Media Source discovery.
- Metadata provider output has a provider-neutral candidate seam or a focused
  follow-on with acceptance criteria.
- Search semantics are deeper than the current storage trait wrapper or split
  into a dedicated follow-on with proof requirements.

Primary gates:

- `cargo check -p nako-server --tests`
- `cargo check -p nako-library --tests`
- `cargo check -p nako-metadata --tests`
- `cargo check -p nako-search --tests`
- focused `cargo nextest run` commands for touched crates.

## M3 — API, Generated Contract, And Repository Hygiene

Status: completed.

Exit criteria:

- Admin API read models remain explicit and redacted after persistence/runtime
  refactors.
- Public Client API and `nako-client-protocol` remain free of admin/storage
  internals.
- Generated frontend/SDK artifacts are reproducible and ignored where they
  should not be tracked.

Primary gates:

- `cargo check -p nako-api --tests`
- `cargo check -p nako-server --tests`
- OpenAPI/SDK leakage tests.
- frontend or SDK `npm run verify` commands when touched.

## M4 — Deletion Sweep And Closeout

Status: completed.

Exit criteria:

- Obsolete production paths introduced or replaced by this lane are deleted.
- Any remaining broad work is split into active follow-on workstreams.
- Workstream docs, roadmap/goal docs, and evidence agree.
- Final verification gates are fresh.

Primary gates:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast` unless a narrower gate is
  explicitly justified.
- `git diff --check`

Closeout evidence:

- FRA-130 deleted the `nako-api` root compatibility re-export shim and updated
  callers to explicit API boundary modules.
- FRA-140 verified the complete lane with `cargo check --workspace --tests`
  and `cargo nextest run --workspace --no-fail-fast`.
