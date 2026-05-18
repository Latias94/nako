# Metadata Merge Policy Unification Milestones

Status: Proposed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- NFO XML preservation and provider breadth are explicit non-goals.
- The first executable task is characterization before code movement.
- Relevant ADRs and prior workstreams are linked.

Primary evidence:

- `docs/workstreams/metadata-merge-policy-unification/DESIGN.md`
- `docs/workstreams/metadata-merge-policy-unification/TODO.md`

## M1 - Current Behavior Characterization

Exit criteria:

- Existing provider merge behavior is captured by focused tests.
- Existing NFO import merge behavior is captured by focused tests.
- Cross-source lock behavior is test-visible before refactor.

Primary gates:

- `cargo nextest run -p taru-metadata merge --no-fail-fast`
- `cargo nextest run -p taru-nfo nfo_service --no-fail-fast`

## M2 - Shared Policy Boundary

Exit criteria:

- One policy boundary owns Canonical Metadata field replacement decisions.
- NFO import and provider/hierarchy merge callers use the shared boundary.
- Duplicated NFO field merge loop is removed.
- Source-aware locks remain explicit and tested.

Primary gates:

- `cargo check -p taru-core --tests`
- `cargo check -p taru-metadata --tests`
- `cargo check -p taru-nfo --tests`
- focused `cargo nextest` commands from M1

## M3 - Integration And Documentation

Exit criteria:

- Docs explain the shipped source-aware merge model.
- Evidence records the targeted gates and what they prove.
- Follow-ons are split instead of silently widened.

Primary gates:

- `cargo fmt --all -- --check`
- `git diff --check`

## M4 - Closeout

Exit criteria:

- Fresh verification evidence is recorded.
- Review has no blocking findings.
- `WORKSTREAM.json` and `HANDOFF.md` reflect the final state.

