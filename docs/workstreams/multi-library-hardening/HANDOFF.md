# Multi-Library Hardening Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

The historical M8 correctness note has been promoted into a standard execution
workstream. No new Rust implementation has started under this lane.

The concrete risk is Library authority drift: configuration is still the
desired-state input, but workflows need a reconciled persisted Library view
after startup.

## Active Task

- Task ID: MLH-020
- Owner: unassigned
- Files: `crates/taru-server/src/config.rs`,
  `crates/taru-server/src/app/startup.rs`,
  `crates/taru-server/src/app/tests/startup.rs`
- Validation: `cargo check -p taru-server --tests`; `cargo nextest run -p
  taru-server startup --no-fail-fast`
- Status: READY
- Review: required before accepting completion
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Keep Public Client Source Locator redaction in a separate lane.
- Keep Library Access, RBAC, and admin mutation APIs out of this lane.
- Characterize current startup/config behavior before introducing a
  reconciliation boundary.

## Blockers

- None known.

## Next Recommended Action

Run MLH-020. Add or identify focused startup/config tests for configured
libraries, persisted Library rows, duplicate IDs, missing configured libraries,
and one-library fallback helpers.
