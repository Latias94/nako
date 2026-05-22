# nako-api Module Split Milestones

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

Exit criteria:

- Workstream docs define module boundaries, non-goals, and validation gates.
- `docs/GOALS.md` still names M46 as the active/recommended next architecture
  goal until closeout.

## M1 - Behavior-Preserving Module Split

Exit criteria:

- `crates/nako-api/src/lib.rs` only declares modules and compatibility
  re-exports.
- Public Client mapping functions live in `public_client`.
- Admin/internal DTOs live outside `public_client`.
- Metadata diagnostics and maintenance DTOs live outside `public_client`.
- Extension/addon/webhook/automation DTOs live outside `public_client`.
- Existing root-level imports in `nako-server` still compile.

## M2 - Contract And Workspace Validation

Exit criteria:

- `nako-api` unit tests pass.
- `nako-api` examples compile.
- TypeScript SDK package still type-checks.
- Workspace check and nextest pass.
- `git diff --check` has no whitespace errors.
- `docs/GOALS.md` records M46 completion and next recommended goal.

Completion notes:

- M0, M1, and M2 are complete.
- Root-level compatibility exports are intentionally retained.
- Server call-site import cleanup remains a low-priority follow-on, not a
  correctness blocker.
