# Fearless Future Architecture Refactor — Milestones

Status: Complete
Last updated: 2026-05-23

## M0 — Scope And Boundary Freeze

Status: Complete

Exit criteria:

- Problem, target state, and non-goals are explicit.
- The architecture hotspot map is recorded.
- The reference policy is explicit.
- The first executable slices are named.

Primary evidence:

- `docs/workstreams/fearless-future-architecture-refactor/DESIGN.md`
- `docs/workstreams/fearless-future-architecture-refactor/TODO.md`

## M1 — Runtime And Persistence Control Planes

Status: Complete

Exit criteria:

- `nako-server` runtime ownership is split into narrower modules.
- `nako-db` backend ownership is split into clearer modules.
- Redundant forwards and pass-through helpers are removed.
- The new shapes are covered by targeted tests.

Primary gates:

- `cargo check -p nako-server --tests`
- `cargo check -p nako-db --tests`
- focused `cargo nextest run` commands for the touched modules

## M2 — API Boundary Split

Status: Complete

Exit criteria:

- Admin and Public DTO surfaces are explicit.
- Redaction remains local to the API boundary.
- DTOs no longer mirror internal database state for convenience.

Primary gates:

- `cargo check -p nako-api --tests`
- focused `cargo nextest run -p nako-api` commands for the touched contracts

## M3 — VFS And Inference Boundary Split

Status: Complete

Exit criteria:

- Library-file-write policy is separate from low-level VFS primitives.
- Local inference and naming stay explainable and conservative.
- NFO, naming, and inference tests prove the new shape.

Primary gates:

- `cargo check -p nako-vfs --tests`
- `cargo check -p nako-library --tests`
- focused naming/inference/NFO nextest commands

## M4 — Docker Validation And Deletion Sweep

Status: Complete

Exit criteria:

- Docker-backed validation is documented and usable.
- Replaced paths are deleted unless a named follow-on exists.
- Workspace and closeout gates pass.
- `WORKSTREAM.json` reflects the final state.

Primary gates:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode container`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts`
- `git diff --check`

Closeout evidence:

- Deletion sweep found no remaining replaced helper paths requiring immediate
  removal. Remaining `local_inference.rs` width is a named follow-up candidate.
- Workspace, container, and PostgreSQL closeout gates passed on 2026-05-23.
