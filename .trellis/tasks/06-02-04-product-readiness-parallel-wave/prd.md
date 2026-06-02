# Product Readiness Parallel Wave

## Goal

Open the next four-lane development wave after the completed 03 product
convergence queue. This wave should move Nako toward practical product
readiness with user-visible playback, operator-visible reliability, provider
trust, and addon onboarding slices.

## Requirements

- Keep `main` as the planner and integration baseline.
- Create four independent child tasks and worktrees.
- Prefer product-readiness slices over broad architecture refactors.
- Keep every child lane bounded to one feature outcome plus one architecture
  pressure point.
- Require fresh verification before any child is accepted into `main`.

## Child Tasks

- `06-02-04a-media-web-hls-player-ux-hardening`: browser playback UX and HLS
  engine hardening for the Media Web player.
- `06-02-04b-library-scan-scheduling-storage-admission`: scan scheduling and
  storage-health admission for large-library reliability.
- `06-02-04c-provider-governance-audit-undo`: redaction-safe provider
  governance audit and undo trust slice.
- `06-02-04d-addon-install-health-guide`: addon install guide and health
  readiness for sidecar onboarding.

## Acceptance Criteria

- [ ] Four child Trellis tasks exist.
- [ ] Four task branches and worktrees exist under `F:/SourceCodes/Rust/nako-worktrees`.
- [ ] Each child task has a PRD and curated `implement.jsonl` / `check.jsonl`.
- [ ] Each child task records its branch and intended worktree path.
- [ ] The parent task remains a coordination task, not an implementation lane.

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
- Fresh baseline gates passed on `main` before opening this wave:
  `cargo fmt --all -- --check`, `cargo check --workspace --tests`,
  `cargo nextest run --workspace --no-fail-fast`, `npm run check/test --prefix web`,
  and `npm --prefix apps/admin-web run check/test`.
- Planner should use `integrate-lane-results` after workers report completion.
