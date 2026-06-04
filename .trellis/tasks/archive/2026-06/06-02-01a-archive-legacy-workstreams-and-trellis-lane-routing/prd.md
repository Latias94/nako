# Archive Legacy Workstreams And Trellis Lane Routing

## Goal

Make the documentation authority model explicit: Trellis owns active workflow
state, while `docs/workstreams/` remains a historical evidence archive for ADR
and architecture traceability.

## Requirements

* Add a top-level migration notice to `docs/workstreams/README.md`.
* Update architecture routing docs so active work uses Trellis tasks instead of
  new workstream directories.
* Preserve ADR, architecture, and workstream evidence links.
* Do not delete historical workstreams in this task.
* Do not create one Trellis task per historical workstream.

## Acceptance Criteria

* [ ] `docs/workstreams/README.md` says new work uses `.trellis/tasks/`.
* [ ] `docs/architecture/WORKSTREAM_LINKS.md` describes workstreams as legacy
      evidence and tells new work to open Trellis tasks.
* [ ] `docs/architecture/LANES.md` routes active execution by Trellis task and
      still preserves lane/worktree ownership boundaries.
* [ ] Remaining references to workstreams are either historical evidence links
      or explicit proposed follow-on names.

## Definition of Done

* Docs-only diff.
* No workstream directories deleted.
* `rg "Open a new workstream|Active workstream|workstream task ledger" docs/architecture docs/workstreams`
  is reviewed and remaining hits are intentional historical language.

## Worktree

Suggested path: `E:\Rust\nako-worktrees\01a-archive-legacy-workstreams`

Suggested branch: `task/01a-archive-legacy-workstreams`

This task can merge before the feature wave because it reduces planner
ambiguity and has low code conflict risk.
