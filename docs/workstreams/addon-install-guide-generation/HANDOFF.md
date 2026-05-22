# Addon Install Guide Generation Handoff

Status: Completed
Last updated: 2026-05-22

## Current State

AIG-010 through AIG-040 are implemented. Nako now has a server-owned Admin API
route for **Addon Install Guide** generation, Admin Web renders the guide
through the generated Admin API TypeScript contract and data-source seam, and
docs describe the read-only non-Addon-Manager boundary.

## Active Task

- Task ID: none
- Owner: codex
- Files: see `WORKSTREAM.json` and final commit diff.
- Validation:
  - `cargo fmt --all -- --check`
  - `cargo check -p nako-api -p nako-server --tests`
  - focused Rust gates
  - Admin Web gates
  - `git diff --check`
- Status: DONE
- Review: Closeout verification pending final gate rerun after this doc update.
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Server owns guide generation to prevent frontend drift.
- Docker Compose and systemd are text snippets only.
- Secret Reference values are never resolved or echoed.
- Addon Manager lifecycle automation is explicitly out of scope.
- Admin Web renders snippets as inert previews and repeats the lifecycle
  boundary instead of providing install/start controls.

## Blockers

- None.

## Residual Risks

- The generated snippets use placeholders for Addon images/binaries because
  Nako still does not have an Addon discovery/package registry.
- Secret Reference resolution remains a declaration-only placeholder in this
  lane.

## Next Recommended Action

- If lifecycle automation becomes desirable, open a separate Addon Manager
  lane with explicit discovery/package/process-supervision boundaries.
