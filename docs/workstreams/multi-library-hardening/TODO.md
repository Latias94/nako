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

- [ ] MLH-020 [owner=unassigned] [deps=MLH-010] [scope=crates/taru-server/src/config.rs,crates/taru-server/src/app/startup.rs,crates/taru-server/src/app/tests/startup.rs]
  Goal: Characterize current configured-library startup behavior, duplicate ID
  rejection, root updates, missing persisted libraries, and one-library
  fallback helpers.
  Validation: `cargo check -p taru-server --tests`; focused
  `cargo nextest run -p taru-server startup --no-fail-fast`.
  Review: `review-workstream` before accepting completion.
  Evidence: startup/config tests and this workstream's evidence file.
  Handoff: Continue with MLH-030 only after the expected reconciliation
  behavior is test-visible.

## M2 - Reconciliation Boundary

- [ ] MLH-030 [owner=unassigned] [deps=MLH-020] [scope=crates/taru-server,crates/taru-db,crates/taru-core]
  Goal: Implement one startup Library reconciliation boundary that persists
  configured desired state and lets workflows use database Library authority
  after startup.
  Validation: `cargo check -p taru-server --tests`; `cargo check -p taru-db
  --tests`; focused `cargo nextest` filters for reconciliation behavior.
  Review: `review-workstream` for boundary shape and multi-library correctness.
  Evidence: reconciliation service/repository paths and focused tests.
  Handoff: Continue with MLH-040 cleanup after callers use the new boundary.

## M3 - Workflow Cleanup And Docs

- [ ] MLH-040 [owner=unassigned] [deps=MLH-030] [scope=crates/taru-server,docs]
  Goal: Remove or narrow obsolete one-library config helpers and update docs so
  scan, NFO, metadata, jobs, and diagnostics share the same Library authority.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run -p taru-server
  --no-fail-fast`; `git diff --check`.
  Review: `review-workstream` before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, updated docs.
  Handoff: Split Library Access/admin mutation follow-ons if they become
  necessary.

## M4 - Closeout

- [ ] MLH-050 [owner=planner] [deps=MLH-040] [scope=docs/workstreams/multi-library-hardening]
  Goal: Close the lane or split narrower follow-ons.
  Validation: `verify-rust-workstream` records fresh final gate evidence.
  Review: no blocking review findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Future Library Access, user sharing, or admin mutation work should
  open its own lane.
