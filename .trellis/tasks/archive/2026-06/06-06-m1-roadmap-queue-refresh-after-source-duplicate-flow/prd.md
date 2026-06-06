# M1 Roadmap Queue Refresh After Source Duplicate Flow

## Goal

Refresh the Product-Operator M1 roadmap and lane queue after the source
duplicate reconciliation operator flow landed, so future sessions choose the
next real M1 gap instead of reopening completed source hash, M1 smoke, or
source duplicate work.

## What I Already Know

- M1 is the current roadmap anchor: a self-hosted, video-first,
  single-admin/operator journey from Media Library configuration through scan,
  catalog/media browse, playback, and Admin diagnostics/repair.
- `m1-operator-journey-smoke` is archived as completed.
- `scan-originated-source-hash-triggering` is archived as completed.
- Source duplicate reconciliation backend plan/apply slices are archived as
  completed.
- `source-duplicate-reconciliation-operator-flow` is archived as completed and
  exposes the Admin Web source duplicate operator flow.
- The current top-level roadmap and lane queue still list several of those
  completed slices as candidate next actions.

## Requirements

- Update `docs/ROADMAP.md` so the M1 queue separates completed convergence
  slices from the next executable queue.
- Update `docs/GOALS.md` so the current M1 planning goal records the
  post-source-duplicate queue refresh and names the next executable queue.
- Update `docs/architecture/LANES.md` so lane routing no longer points
  `storage-vfs` or `web-product` at the completed source duplicate operator
  flow.
- Keep historical evidence and archived task references intact.
- Define the next M1 queue around:
  - `m1-release-ladder-runner`;
  - `media-web-library-browse-and-player-smoke`, only as a targeted follow-on
    if the runner or existing smoke exposes a browser/player blocker;
  - `admin-web-feature-data-adapter-deepening`, starting from source duplicate
    reconciliation if another operator flow needs the same pattern.
- Keep the task docs-only. Do not change Rust, TypeScript, schema, API,
  generated contracts, runtime behavior, or release artifacts.

## Acceptance Criteria

- [x] `docs/ROADMAP.md` marks M1 smoke, scan-originated source hash
      triggering, source duplicate backend plan/apply, and source duplicate
      Admin Web operator flow as completed evidence.
- [x] `docs/ROADMAP.md` names a fresh next executable M1 queue that starts
      with `m1-release-ladder-runner`.
- [x] `docs/GOALS.md` records the queue refresh deliverable and evidence.
- [x] `docs/architecture/LANES.md` routes storage-vfs away from the completed
      source duplicate operator flow and routes operations-release/control-plane
      toward the release ladder runner.
- [x] Trellis task context files contain relevant spec/evidence entries.
- [x] `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-m1-roadmap-queue-refresh-after-source-duplicate-flow`
      passes.
- [x] `git diff --check` passes for touched files.

## Definition Of Done

- Planning docs are updated and validated.
- Evidence records the validation commands and results.
- Work is committed and pushed.
- The Trellis task is archived.

## Technical Approach

Make the current-plan sections authoritative without deleting historical
roadmap content. The update should prefer a small, explicit completed-evidence
table plus a smaller next queue over rewriting long historical roadmap entries.

## Decision

Use a docs-only queue refresh rather than opening another implementation task
from the stale queue.

Consequences:

- Future agents can immediately select the release ladder runner as the next
  high-value M1 convergence slice.
- Completed source duplicate and source hash work remains discoverable as
  evidence, but no longer appears as candidate work.
- The Admin Web data-adapter deepening opportunity stays visible as an
  architecture follow-on instead of blocking release validation.

## Out Of Scope

- No code changes.
- No release runner implementation in this task.
- No live browser or package/container validation in this task.
- No source duplicate relationship mutation changes.
- No broad roadmap renumbering or milestone rewrite.

## Technical Notes

- Primary sources:
  - `docs/ROADMAP.md`
  - `docs/GOALS.md`
  - `docs/architecture/LANES.md`
  - `.trellis/tasks/archive/2026-06/06-06-m1-operator-journey-smoke/`
  - `.trellis/tasks/archive/2026-06/06-06-scan-originated-source-hash-triggering/`
  - `.trellis/tasks/archive/2026-06/06-06-admin-source-duplicate-reconciliation-plan-api/`
  - `.trellis/tasks/archive/2026-06/06-06-admin-source-duplicate-reconciliation-apply-first-slice/`
  - `.trellis/tasks/archive/2026-06/06-06-source-duplicate-reconciliation-operator-flow/`
  - `.trellis/tasks/archive/2026-06/06-06-06-06-current-function-architecture-audit/prd.md`

## Verification Evidence

- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-m1-roadmap-queue-refresh-after-source-duplicate-flow`
  passed with 4 implement context entries and 4 check context entries.
- `git diff --check -- .trellis/tasks/06-06-m1-roadmap-queue-refresh-after-source-duplicate-flow docs/ROADMAP.md docs/GOALS.md docs/architecture/LANES.md`
  passed. Git reported LF-to-CRLF working-copy warnings for the existing docs,
  but no whitespace errors.
- `rg -n "source-duplicate-reconciliation-operator-flow|m1-operator-journey-smoke|scan-originated-source-hash-triggering|m1-release-ladder-runner|admin-web-feature-data-adapter-deepening" docs/ROADMAP.md docs/GOALS.md docs/architecture/LANES.md .trellis/tasks/06-06-m1-roadmap-queue-refresh-after-source-duplicate-flow/prd.md`
  confirmed completed slices and next executable queue references.
