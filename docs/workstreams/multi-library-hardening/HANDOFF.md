# Multi-Library Hardening Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

The historical M8 correctness note has been promoted into a standard execution
workstream. MLH-020 characterization is complete. MLH-030 has now introduced a
named startup reconciliation boundary and explicit reconciliation reporting.

The concrete risk is Library authority drift: configuration is still the
desired-state input, but workflows need a reconciled persisted Library view
after startup.

## Active Task

- Task ID: MLH-030
- Owner: codex
- Files: `crates/taru-server`, `crates/taru-db`, `crates/taru-core`
- Validation: `cargo check -p taru-server --tests`; `cargo check -p taru-db
  --tests`; `cargo nextest run -p taru-server startup --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`
- Status: DONE
- Review: required before accepting completion
- Evidence: `EVIDENCE_AND_GATES.md`, `crates/taru-server/src/app/library_reconciliation.rs`

## Decisions Since Last Update

- Keep Public Client Source Locator redaction in a separate lane.
- Keep Library Access, RBAC, and admin mutation APIs out of this lane.
- Characterize current startup/config behavior before introducing a
  reconciliation boundary.
- Current behavior is now explicit: startup upserts all configured libraries,
  configured desired state overwrites an existing Library with the same ID, and
  persisted libraries missing from config are retained.
- The new startup reconciliation boundary is named and test-visible. It reports
  added, updated, unchanged, and retained unconfigured libraries without
  changing the persistence semantics already characterized in MLH-020.

## Blockers

- None known.

## Next Recommended Action

Run MLH-040. Narrow remaining config-driven library lookups where the
reconciled Library rows are now authoritative.
