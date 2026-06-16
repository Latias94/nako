# Backend self-hosted maturity execution plan

## Goal

Create the backend-first execution plan for Nako self-hosted maturity and seed
the first implementation queue. This task turns the existing media-server
maturity roadmap into concrete backend slices that can be implemented,
verified, committed, and closed through Trellis without reopening old
workstreams.

## What I Already Know

* `docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md`
  defines the broad maturity roadmap.
* `docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md` classifies which
  competitor-facing capabilities Nako should build, integrate, defer, or avoid.
* `docs/ROADMAP.md` currently frames M1 through M5 as operator release,
  large-library reliability, playback/transcode maturity, metadata governance,
  and Addon ecosystem maturity.
* The current backend foundation is strong enough that the next work should
  close product loops rather than add broad new feature categories.
* The first backend wave should keep Addon lifecycle mostly deferred and focus
  on operator readiness, API scale, playback reasons, intake reliability,
  access policy, and remote endpoint diagnostics.

## Requirements

* Write a new backend execution plan under `docs/plans/`.
* Keep the existing broad maturity roadmap intact.
* Define staged backend implementation units with requirements, decisions,
  file areas, tests, verification, and scope boundaries.
* Identify the first batch of Trellis-ready implementation slices.
* Preserve Nako domain language from `CONTEXT.md`.
* Keep GPL reference repositories as behavioral references only.
* Do not change production code in this planning task.

## Acceptance Criteria

* [x] A new markdown plan exists in `docs/plans/` with required plan metadata.
* [x] The plan has requirements, key technical decisions, implementation units,
      scope boundaries, risks, acceptance examples, and source references.
* [x] The plan distinguishes first-wave backend work from deferred Addon,
      relay, TV-client, offline artifact, and broad frontend work.
* [x] The plan names deletion and fearless-refactor policy without authorizing
      untested removals.
* [x] `implement.jsonl` and `check.jsonl` contain real spec/research context and
      no placeholder line.
* [x] Task validation and diff checks pass.

## Definition of Done

* Planning documents are written.
* Trellis task context validates.
* `git diff --check` passes.
* No production Rust, generated API, or frontend runtime file is changed.
* A conventional commit records the planning work.

## Out of Scope

* Implementing the first backend slice in this task.
* Creating a Plex-style relay or central account service.
* Implementing Addon Manager process supervision.
* Changing public API DTOs, database schema, generated contracts, or runtime
  behavior.
* Editing GPL reference source or copying implementation details from it.

## Technical Notes

Primary plan artifact:

* `docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`

Planning anchors:

* `docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md`
* `docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md`
* `docs/plans/PRODUCT_STRATEGY_IMPLEMENTATION_BACKLOG.md`
* `docs/ROADMAP.md`
* `docs/architecture/CONTROL_PLANE.md`
* `docs/architecture/PLAYBACK.md`
* `docs/architecture/LIBRARY_PIPELINE.md`
* `docs/architecture/STORAGE_VFS.md`
* `docs/architecture/STATE_ACCESS.md`
* `docs/architecture/OPERATIONS_RELEASE.md`

## Open Questions

None for this planning task. The selected first batch is backend-first and keeps
Addon lifecycle to wave two unless a wave-one contract blocks it.

## Implementation Summary

* Added a backend self-hosted maturity execution plan under `docs/plans/`.
* Created Trellis task context for the planning slice.
* Replaced default context placeholders with spec, architecture, and planning
  anchors for future implement/check agents.
* Seeded the first backend execution queue:
  `backend-readiness-control-plane-audit`,
  `public-library-browse-scale-contract`,
  `playback-reason-public-contract`,
  `intake-scheduler-repair-workflow-audit`, and
  `remote-endpoint-readiness-diagnostics`.

## Verification Evidence

* `python ./.trellis/scripts/task.py validate .trellis/tasks/06-16-06-16-backend-self-hosted-maturity-execution-plan`: passed.
* `git diff --check`: passed.
* Production Rust, generated API, and frontend runtime files were not changed.
