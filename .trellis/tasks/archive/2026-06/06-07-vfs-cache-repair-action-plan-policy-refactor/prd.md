# VFS Cache Repair Action Plan Policy Refactor

## Goal

Refactor the VFS cache repair action-plan logic so the shared decision rules
for action, status, reasons, boundary flags, and executable route live in one
helper instead of being duplicated between remediation planning and direct
repair planning.

## What I Already Know

- `StorageDiagnosticsAppService` currently computes the same `VfsCacheRepair`
  action-plan semantics in two places:
  - `VfsCacheRepairActionPlanReport::from_repair`
  - `push_remediation_action_group`
- The duplicated logic maps the same `VfsCacheRepairAction` values to the same
  status, reasons, boundary flags, and executable route decisions.
- This area is already covered by Admin diagnostics and VFS cache repair tests;
  a refactor should preserve behavior exactly.

## Requirements

- Extract the shared VFS cache repair action-plan semantics into one helper.
- Reuse that helper from both remediation aggregation and target/repair action
  planning.
- Keep the public DTOs, route shapes, and redaction behavior unchanged.
- Keep the action/status/reason/boundary/executable-route outputs identical for
  all existing repair actions.
- Preserve existing refreshable, backend-configuration, and manual-inspection
  distinctions.

## Acceptance Criteria

- [ ] The shared action-plan logic exists in one place.
- [ ] Remediation plan output is unchanged for existing fixtures/tests.
- [ ] Direct repair action-plan output is unchanged for existing fixtures/tests.
- [ ] Focused VFS cache repair tests continue to pass.
- [ ] `cargo fmt --all -- --check` and `git diff --check` pass.

## Definition Of Done

- No behavioral change in VFS cache repair planning.
- Duplicate action-plan mapping logic is removed or reduced to one helper.
- The refactor is covered by existing or updated focused tests.
- The change is committed as a Conventional Commit once verified.

## Technical Approach

Extract a small helper that returns the common plan semantics for a
`VfsCacheRepairAction`, then have both the single-plan and remediation-group
paths use it.

## Out Of Scope

- No new API route.
- No new durable job behavior.
- No new cache mutation behavior.
- No policy/automation enablement.

## Technical Notes

- Main file: `crates/nako-server/src/app/storage.rs`
- Supporting route surface: `crates/nako-server/src/http/admin.rs`
- Relevant domain types: `nako_vfs::VfsCacheRepairAction`,
  `VfsCacheRepairActionPlanStatus`, `VfsCacheRepairActionPlanReason`,
  `VfsCacheRepairActionBoundary`

