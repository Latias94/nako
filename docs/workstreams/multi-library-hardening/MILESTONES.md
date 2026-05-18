# Multi-Library Hardening Milestones

Status: Completed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

Exit criteria:

- M8 historical decisions are linked.
- Config/database Library authority problem is explicit.
- Public locator redaction and Library Access are explicit non-goals.
- First executable task is characterization.

Primary evidence:

- `docs/workstreams/multi-library-hardening/DESIGN.md`
- `docs/workstreams/multi-library-hardening/TODO.md`

## M1 - Current Behavior Characterization

Exit criteria:

- Startup behavior for configured libraries is test-visible.
- Duplicate configured IDs, duplicate roots, missing libraries, and updates have
  expected behavior recorded.
- Existing one-library CLI/app shortcuts are identified before deletion.

Primary gates:

- `cargo check -p taru-server --tests`
- focused `cargo nextest run -p taru-server startup --no-fail-fast`

## M2 - Reconciliation Boundary

Exit criteria:

- Startup uses one reconciliation workflow for configured Library desired
  state.
- Persisted Library rows are updated through an explicit repository boundary.
- Ordinary library-scoped workflows use reconciled Library rows.

Primary gates:

- `cargo check -p taru-server --tests`
- `cargo check -p taru-db --tests`
- focused `cargo nextest run -p taru-server <filter> --no-fail-fast`

## M3 - Workflow Cleanup

Exit criteria:

- Obsolete one-library fallback helpers are removed or narrowed.
- Scan, NFO, metadata, jobs, and storage diagnostics use consistent Library
  authority.
- Docs describe the shipped reconciliation model.

Primary gates:

- `cargo nextest run -p taru-server --no-fail-fast`
- targeted `cargo nextest run -p taru-db <filter> --no-fail-fast`

## M4 - Closeout

Exit criteria:

- Fresh verification evidence is recorded.
- Remaining Library Access, admin mutation, or public contract work is split.
- `WORKSTREAM.json` and `HANDOFF.md` reflect the final state.
