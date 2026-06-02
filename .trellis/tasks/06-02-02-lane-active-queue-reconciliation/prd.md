# Lane Active Queue Reconciliation

## Goal

Reconcile `docs/architecture/LANES.md` after the 01a-01f parallel worktree
plan completed, so future planner and lane terminals no longer treat completed
tasks as active work.

## What I already know

- `06-02-01-parallel-worktree-development-plan` and children 01a-01f are marked
  completed in Trellis.
- `docs/ROADMAP.md` says no active architecture focus is currently selected.
- `docs/architecture/LANES.md` still lists 01a-01f in the `Active Queue`.
- `00-bootstrap-guidelines` remains in progress and should stay separate from
  implementation lane planning.

## Requirements

- Update only planner-facing queue state in `docs/architecture/LANES.md`.
- Replace completed 01a-01f active rows with a truthful idle/current-state
  summary.
- Preserve historical evidence and lane registry content below the queue.
- Mention that 00 bootstrap is still active as documentation/spec setup, not an
  implementation lane.
- Do not create new 02 implementation tasks or choose a new architecture focus
  in this cleanup.

## Acceptance Criteria

- [x] `docs/architecture/LANES.md` no longer lists completed 01a-01f tasks as
  active.
- [x] The file points planners to candidate follow-ons without selecting one as
  active.
- [x] `docs/ROADMAP.md` and `docs/GOALS.md` remain consistent with the lane
  active queue.
- [x] `git diff --check` passes.

## Definition of Done

- Trellis task context is configured.
- Docs diff is narrow and reviewable.
- Task validation passes.
- Commit uses a Conventional Commit message.

## Out of Scope

- Filling remaining `.trellis/spec` packages.
- Creating new 02 implementation tasks.
- Reopening closed workstreams.
- Changing ADRs, roadmap milestones, or implementation code.

## Technical Notes

- Worktree: `F:\SourceCodes\Rust\nako-worktrees\02a-lane-active-queue-reconciliation`
- Branch: `task/02a-lane-active-queue-reconciliation`
- Mainline baseline: `1ffba98d docs(trellis): seed core package guidelines`
