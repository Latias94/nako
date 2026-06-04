# Parallel Worktree Development Plan

## Goal

Prepare a small set of Trellis tasks that can be developed in separate Git
worktrees under `E:\Rust\nako-worktrees` without treating legacy workstreams as
active workflow state.

## What I Already Know

* Active workflow authority has moved to `.trellis/tasks/`, `.trellis/spec/`,
  and `.trellis/workspace/`.
* `docs/workstreams/` has 278 historical directories. `WORKSTREAM.json` statuses
  are `closed`, `complete`, or `completed`; 10 older directories have no
  `WORKSTREAM.json`.
* ADR and architecture docs still reference workstream evidence heavily, so
  workstreams should be frozen as history rather than deleted.
* `E:\Rust\nako` is a junction to `E:\Rust\taru`; `git worktree list` reports
  the canonical main worktree as `E:/Rust/taru`.
* Current bootstrap/Trellis changes are uncommitted. Create feature worktrees
  only after the bootstrap commit lands on `main`.

## Requirements

* Do not create new `docs/workstreams/*` directories.
* Keep one parent Trellis planning task with child tasks for parallel lanes.
* Keep each child task bounded by one architecture lane where possible.
* Identify shared surfaces that should serialize work: `nako-api` generated
  contracts, Admin Web generated contract output, schema migrations, ADR edits,
  and cross-lane server HTTP changes.
* Do not start implementation from a dirty main worktree.

## Proposed Parallel Queue

| Task | Lane | Suggested branch | Suggested worktree | Parallel guidance |
| --- | --- | --- | --- | --- |
| `01a-archive-legacy-workstreams-and-trellis-lane-routing` | architecture-planning | `task/01a-archive-legacy-workstreams` | `E:\Rust\nako-worktrees\01a-archive-legacy-workstreams` | Merge first if possible; docs-only cleanup reduces later ambiguity. |
| `01b-admin-settings-api-backed-restoration` | web-product / operations-release | `task/01b-admin-settings-api-restoration` | `E:\Rust\nako-worktrees\01b-admin-settings-api-restoration` | Do not run beside another task changing `nako-api` Admin contracts. |
| `01c-provider-review-related-hierarchy-application` | library-metadata-control-plane | `task/01c-provider-related-hierarchy` | `E:\Rust\nako-worktrees\01c-provider-related-hierarchy` | Prefer backend-first planning; avoid concurrent Admin Web/generated-contract work with 01b. |
| `01d-hls-artifact-io-pressure-enforcement` | playback-transcode | `task/01d-hls-artifact-io-pressure` | `E:\Rust\nako-worktrees\01d-hls-artifact-io-pressure` | Good parallel candidate with 01b/01e if server shared surfaces stay scoped. |
| `01e-storage-vfs-cache-repair-diagnostics` | storage-vfs | `task/01e-vfs-cache-repair-diagnostics` | `E:\Rust\nako-worktrees\01e-vfs-cache-repair-diagnostics` | Good parallel candidate with 01b/01f; coordinate with 01d only on playback input staging. |
| `01f-durable-job-priority-policy-and-scheduler-migration` | control-plane | `task/01f-durable-job-priority-policy` | `E:\Rust\nako-worktrees\01f-durable-job-priority-policy` | Good parallel candidate if it stays scheduler/runtime focused and does not modify provider review semantics. |

## Recommended Waves

* Wave 0: finish and commit Trellis bootstrap plus 01a docs cleanup.
* Wave 1: run 01b, 01d, 01e, and 01f in separate worktrees.
* Wave 2: run 01c after deciding whether it needs Admin API/Web surfaces; if
  yes, serialize it after 01b to avoid generated contract conflicts.

## Acceptance Criteria

* [ ] Legacy workstreams are marked as historical evidence, not active task
      authority.
* [ ] Each child task has a PRD with worktree path, lane, scope, conflict notes,
      acceptance criteria, and definition of done.
* [ ] Each child task has `implement.jsonl` and `check.jsonl` with only specs,
      ADRs, architecture docs, or legacy evidence links.
* [ ] No implementation task is started until the bootstrap commit is on `main`.

## Out of Scope

* Deleting historical workstream directories.
* Creating actual Git worktrees before the baseline commit is clean.
* Implementing feature code from the planner task.

## Technical Notes

Example worktree command after committing bootstrap:

```powershell
git worktree add -b task/01b-admin-settings-api-restoration E:\Rust\nako-worktrees\01b-admin-settings-api-restoration main
```

Run the command from the canonical repository root or the junction path, but
record the resulting path in the child task before starting implementation.
