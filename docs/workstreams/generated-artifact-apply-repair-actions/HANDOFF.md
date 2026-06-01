# Generated Artifact Apply Repair Actions — Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is newly opened. The recovery queue and Web recovery route are
read-only and closed in prior lanes. Operators can inspect recovery entries and
open the existing Metadata Authority apply plan route, but there is no
recovery-scoped repair action contract yet.

## Active Task

- Task ID: `GAARA-020`
- Owner: codex
- Files: `crates/nako-core/src/automation.rs`, `crates/nako-api/src/admin/automation.rs`, `crates/nako-server/src/app/automation.rs`, `crates/nako-server/src/http/admin.rs`, `crates/nako-server/src/app/tests/automation.rs`, `web/src/api/admin`, `web/src/features/admin`, `web/src/test`, and this workstream
- Validation: focused server nextest for idempotent replay and stale-target rejection; add Web/Rust gates only if the task changes those surfaces
- Status: READY
- Evidence: `docs/workstreams/generated-artifact-apply-repair-actions/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Opened a new `library-metadata-control-plane` follow-on instead of reopening
  GAOR or WAGR.
- First task is a seam proof, not a mutation implementation.
- Existing Metadata Authority apply and bulk apply are the preferred execution
  kernels.
- Read-only explorer audit concluded no new metadata mutation core is needed;
  a narrow wrapper is justified only for recovery-context guards or one-click
  repair UX.

## Blockers

- None for `GAARA-020`.

## Next Recommended Action

- Run `GAARA-020`: prove whether repair needs no backend mutation, a narrow
  recovery wrapper, or Web-only repair preparation over existing apply routes.
