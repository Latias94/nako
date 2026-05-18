# Metadata Catalog Commit Atomicity Milestones

Status: Proposed
Last updated: 2026-05-18

## M0 - Scope

Exit criteria:

- Problem and target state are explicit.
- Non-goals are explicit.
- Relevant ADRs and prior workstreams are linked.

Primary evidence:

- `DESIGN.md`
- `TODO.md`

## M1 - Catalog Graph/Search Atomic Commit

Exit criteria:

- Catalog hydration commits graph replacement and search projection through
  one interface.
- The SQLite adapter persists both in one transaction.
- Existing catalog hydration behavior remains covered.

Primary gates:

- `cargo check -p taru-catalog --tests`
- `cargo nextest run -p taru-catalog --no-fail-fast`
- `cargo check -p taru-db --tests`
- targeted `cargo nextest run -p taru-db <filter>`

## M2 - Metadata Refresh Commit Unit Follow-Up

Exit criteria:

- The next unit-of-work shape is documented.
- The lane either continues with a focused metadata refresh commit slice or
  splits that work into a follow-up.

Primary evidence:

- `HANDOFF.md`
- updated `TODO.md`

## M3 - Closeout

Exit criteria:

- Fresh gate evidence is recorded.
- Remaining risks are documented.
- `WORKSTREAM.json` status is updated.
