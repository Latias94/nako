# Multi-Library Hardening Handoff

Status: Completed
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

- Task ID: none
- Owner: planner
- Files: `docs/workstreams/multi-library-hardening`
- Validation: closeout evidence recorded in `EVIDENCE_AND_GATES.md`
- Status: DONE
- Review: no blocking findings
- Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`

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
- Startup now rejects duplicate configured library IDs, duplicate configured
  local roots, and unsupported WebDAV root schemes before reconciliation.

## Blockers

- None known.

## Next Recommended Action

Stop this lane. Follow-on work for Library Access, admin mutation, or additional
backend-root policy belongs in a new workstream.
