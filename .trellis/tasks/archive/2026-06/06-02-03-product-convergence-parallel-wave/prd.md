# Product Convergence Parallel Wave

## Goal

Open the next four-lane development wave after the completed 01a-01f queue and
the archived Trellis spec bootstrap. This wave should move Nako from strong
architecture foundations toward product convergence without starting another
global refactor.

## Requirements

- Keep `main` as the planner/integration baseline.
- Create four independent child tasks and worktrees.
- Make every child lane own a bounded feature outcome plus one architecture
  deepening pressure point.
- Avoid assigning overlapping write-heavy scopes unless the task explicitly
  declares shared-scope coordination.
- Require every worker to return status, changed files, validation, concerns,
  and follow-ons.

## Child Tasks

- `06-02-03a-media-web-playback-first-watch-flow`: browser Media Web first watch
  flow using public client/playback contracts.
- `06-02-03b-playback-runtime-resource-admission`: playback runtime resource
  admission and operator-visible pressure semantics.
- `06-02-03c-provider-governance-audit-public-contract`: provider governance
  audit/undo and public contract planning or implementation slice.
- `06-02-03d-storage-control-plane-operational-hardening`: storage/VFS and
  control-plane operational hardening slice.

## Acceptance Criteria

- [x] Four child Trellis tasks exist.
- [x] Four task branches and worktrees exist under `F:/SourceCodes/Rust/nako-worktrees`.
- [x] Each child task has a PRD and curated `implement.jsonl` / `check.jsonl`.
- [x] Each child task records its branch and intended worktree path.
- [x] The parent task remains a coordination task, not an implementation lane.

## Definition of Done

- Task docs are committed on `main`.
- Worktrees are created from the committed `main` baseline.
- Prompts are provided for each lane terminal.
- No implementation code is changed by this planner setup.

## Out of Scope

- No implementation work in the planner terminal.
- No schema migrations or ADR rewrites in this setup task.
- No automatic launch of Codex workers unless the user explicitly asks.

## Technical Notes

- Source architecture authority: `docs/architecture/LANES.md`.
- Current queue state: 01a-01f completed and merged; no active implementation
  lane is selected.
- Planner should use `integrate-lane-results` after workers report completion.
