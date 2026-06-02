# Provider Review Global Queue Search - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from `admin-candidate-review-list-navigation` closeout.
Item-scoped Candidate Review discovery/navigation exists, but operators still
need a global Admin queue/search surface for cross-item triage.

## Active Task

- Task ID: `PRGQ-020`
- Owner: codex
- Files: `crates/nako-core`, `crates/nako-db`, `crates/nako-api`,
  `crates/nako-server`, and
  `docs/workstreams/provider-review-global-queue-search`
- Validation: `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`;
  focused `nako-db` queue query tests if repository contract changes;
  `cargo fmt --all -- --check`; `git diff --check`
- Status: READY
- Evidence: `docs/workstreams/provider-review-global-queue-search/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Start with a read-only Admin API global queue before Web.
- Queue rows are summaries for triage/navigation, not full Candidate Review
  detail duplication.
- HTTP must not load broad rows and filter them in memory; the repository owns
  filters and pagination.
- Batch governance, status mutation, apply mutation, Public Client API, and
  related hierarchy application remain out of scope.

## Blockers

- None for `PRGQ-020`.

## Next Recommended Action

- Run `PRGQ-020`: add the read-only Admin API global Candidate Review queue
  route, repository query contract, redaction-safe DTOs, and route/query tests.
