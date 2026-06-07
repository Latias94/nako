# Code Reading Notes

Date: 2026-06-07

The VFS cache repair action-plan semantics are duplicated inside
`crates/nako-server/src/app/storage.rs`.

The duplicated mapping appears in:

- `VfsCacheRepairActionPlanReport::from_repair`
- `push_remediation_action_group`

Both places map the same `VfsCacheRepairAction` values to the same:

- `VfsCacheRepairActionPlanStatus`
- `VfsCacheRepairActionPlanReason`
- `VfsCacheRepairActionBoundary`
- optional executable route

The refactor should extract that mapping into one helper and leave the public
behavior unchanged.

