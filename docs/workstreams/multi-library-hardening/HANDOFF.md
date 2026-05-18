# Multi-Library Hardening Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

The historical M8 correctness note has been promoted into a standard execution
workstream. MLH-020 characterization is complete. MLH-030 introduced a named
startup reconciliation boundary and explicit reconciliation reporting. MLH-040
has now narrowed scan, metadata, NFO, and storage diagnostics to use the
reconciled persisted Library view where Library fields are authoritative.

The concrete risk is Library authority drift: configuration is still the
desired-state input and still owns backend credentials/local physical roots,
but workflows now load Library names, roots, profiles, and scan/local metadata
options from the database after startup reconciliation.

## Active Task

- Task ID: MLH-050
- Owner: codex
- Files: `docs/workstreams/multi-library-hardening`, `crates/taru-server`
- Validation: `cargo check -p taru-server --tests`; `cargo check -p taru-db
  --tests`; `cargo nextest run -p taru-server --no-fail-fast`; `cargo fmt
  --all -- --check`; `git diff --check`
- Status: READY
- Review: run `review-workstream` before closeout
- Evidence: `EVIDENCE_AND_GATES.md`, MLH-040 tests and journal

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
- Scan jobs, metadata maintenance, metadata refresh, and NFO import now load
  persisted Library records through `LibraryRepository` instead of rebuilding
  Library options from broad server config.
- Storage diagnostics now enumerates reconciled persisted libraries. It still
  resolves backend credentials and local physical roots from config because
  those secrets and host paths are not part of the persisted Library authority.

## Blockers

- None known.

## Next Recommended Action

Run MLH-050. Review the lane, record any residual risks, and either close this
workstream or split follow-ons for Library Access/admin mutation work.
