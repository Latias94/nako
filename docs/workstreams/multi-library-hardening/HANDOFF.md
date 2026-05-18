# Multi-Library Hardening Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

The historical M8 correctness note has been promoted into a standard execution
workstream. MLH-020 characterization is complete.

The concrete risk is Library authority drift: configuration is still the
desired-state input, but workflows need a reconciled persisted Library view
after startup.

## Active Task

- Task ID: MLH-030
- Owner: unassigned
- Files: `crates/taru-server`, `crates/taru-db`, `crates/taru-core`
- Validation: `cargo check -p taru-server --tests`; `cargo check -p taru-db
  --tests`; focused `cargo nextest` filters for reconciliation behavior
- Status: READY
- Review: required before accepting completion
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Keep Public Client Source Locator redaction in a separate lane.
- Keep Library Access, RBAC, and admin mutation APIs out of this lane.
- Characterize current startup/config behavior before introducing a
  reconciliation boundary.
- Current behavior is now explicit: startup upserts all configured libraries,
  configured desired state overwrites an existing Library with the same ID, and
  persisted libraries missing from config are retained.

## Blockers

- None known.

## Next Recommended Action

Run MLH-030. Introduce a named reconciliation boundary that preserves the
characterized behavior or deliberately changes it with updated tests and docs.
