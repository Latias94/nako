# Multi-Library Hardening TODO

Status: Proposed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

- [x] MLH-010 [owner=codex] [deps=none] [scope=docs/workstreams/multi-library-hardening]
  Goal: Promote the historical M8 notes into a standard workstream with
  problem, target state, non-goals, gates, and first executable task.
  Validation: `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `PHASE8_0_CORRECTNESS_BASELINE.md`.
  Handoff: Continue with MLH-020 characterization before changing startup
  reconciliation behavior.

## M1 - Current Behavior Characterization

- [x] MLH-020 [owner=codex] [deps=MLH-010] [scope=crates/taru-server/src/config.rs,crates/taru-server/src/app/startup.rs,crates/taru-server/src/app/tests/startup.rs]
  Goal: Characterize current configured-library startup behavior, duplicate ID
  rejection, root updates, missing persisted libraries, and one-library
  fallback helpers.
  Validation: `cargo check -p taru-server --tests`; focused
  `cargo nextest run -p taru-server startup --no-fail-fast`.
  Review: `review-workstream` before accepting completion.
  Evidence: startup/config tests and this workstream's evidence file.
  Handoff: Startup/config behavior is now test-visible. Continue with MLH-030
  to introduce an explicit reconciliation boundary.

## M2 - Reconciliation Boundary

- [x] MLH-030 [owner=codex] [deps=MLH-020] [scope=crates/taru-server,crates/taru-db,crates/taru-core]
  Goal: Implement one startup Library reconciliation boundary that persists
  configured desired state, reports reconciliation outcomes, and lets startup
  and downstream workflows rely on database Library authority after startup.
  Validation: `cargo check -p taru-server --tests`; `cargo check -p taru-db
  --tests`; `cargo nextest run -p taru-server startup --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Review: boundary shape reviewed against ADR 0019; no blocking findings.
  Evidence: `crates/taru-server/src/app/library_reconciliation.rs`,
  `crates/taru-server/src/app/startup.rs`,
  `crates/taru-server/src/app/tests/startup.rs`.
  Handoff: Continue with MLH-040 cleanup after callers narrow broad config
  lookups where database Library authority is now available.

## M3 - Workflow Cleanup And Docs

- [x] MLH-040 [owner=codex] [deps=MLH-030] [scope=crates/taru-server,docs]
  Goal: Remove or narrow obsolete one-library config helpers and update docs so
  scan, NFO, metadata, jobs, and diagnostics share the same Library authority.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run -p taru-server
  --no-fail-fast`; `git diff --check`.
  Review: required before closeout.
  Evidence: scan, metadata, NFO, storage diagnostics, and startup root-policy
  tests now cover reconciled Library authority after startup.
  Handoff: Continue with MLH-050 closeout review and split Library Access/admin
  mutation follow-ons only if the review finds they are necessary.

## M4 - Closeout

- [ ] MLH-050 [owner=planner] [deps=MLH-040] [scope=docs/workstreams/multi-library-hardening]
- [x] MLH-050 [owner=planner] [deps=MLH-040] [scope=docs/workstreams/multi-library-hardening]
  Goal: Close the lane or split narrower follow-ons.
  Validation: `verify-rust-workstream` records fresh final gate evidence.
  Review: no blocking review findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Future Library Access, user sharing, or admin mutation work should
  open its own lane.
