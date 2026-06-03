# Next Architecture Parallel Wave

## Goal

Open the next four independent implementation lanes after scan staging pressure
admission, keeping storage/library/control-plane work moving without forcing one
large serial branch.

## Child Tasks

- `06-03-05a-staging-budget-per-backend-policy`
- `06-03-05b-scan-scheduler-library-fairness`
- `06-03-05c-storage-runtime-postgres-parity-harness`
- `06-03-05d-library-watcher-debounce-intake-stability`

## Coordination Rules

- Each child uses its own git worktree under
  `F:/SourceCodes/Rust/nako-worktrees/`.
- Each child branch is based on `main` at `d64470c6` or later.
- Child tasks should avoid broad API churn unless their PRD explicitly says so.
- If two lanes touch the same function, the later finisher should rebase/merge
  main and adapt rather than hiding conflicts in a broad refactor.

## Done

- All child task directories have PRD and context JSONL.
- All child task.json files identify branch and worktree path.
- All child worktrees are created from current `main`.
