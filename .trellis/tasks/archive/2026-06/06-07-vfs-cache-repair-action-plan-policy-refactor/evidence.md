# Evidence: VFS cache repair action-plan policy refactor

## Implementation Summary

The shared VFS cache repair action-plan semantics now live in one helper in
`crates/nako-server/src/app/storage.rs`.

The refactor keeps the semantic fields aligned across:

* direct repair planning via `VfsCacheRepairActionPlanReport::from_repair`
* target preview planning via `VfsCacheRepairActionPlanReport::from_target_preview_repair`
* remediation aggregation via `push_remediation_action_group`

The `RefreshCache` executable route remains context-specific:

* direct repair planning uses `LatestRefreshCache`
* target preview and remediation grouping use `TargetRefreshCache`

## Files Changed

* `crates/nako-server/src/app/storage.rs`

## Verification

* PASS: `cargo fmt --all`
* PASS: `cargo nextest run -p nako-server vfs_cache_repair --no-fail-fast`
* PASS: `cargo nextest run -p nako-api admin_contract --no-fail-fast`
* PASS: `cargo fmt --all -- --check`
* PASS: `git diff --check`
* PASS: `python ./.trellis/scripts/task.py validate .trellis/tasks/06-07-vfs-cache-repair-action-plan-policy-refactor`

## Notes

* The refactor preserves the existing Admin contract route key for remediation
  refresh actions.
* No public DTO, route shape, or redaction behavior changed.
