# Metadata Application Cross-Path Audit - Milestones

Status: Complete
Last updated: 2026-05-26

## MACPA-M1 - Provider Refresh Reviewed

Provider refresh applies fetched provider facts through
`MetadataMergePolicy::from_locks_and_mode` and returns a provider refresh commit
for the caller to persist.

## MACPA-M2 - Hierarchy Confirmation Reviewed

Hierarchy confirmation validates structural changes separately, then applies
metadata through `MetadataMergePolicy::for_source_refresh_mode` and hydrates the
catalog after repository writes.

## MACPA-M3 - Boundary Confirmed

No pure-core extraction is needed yet. A new core application command/result
would be justified only when multiple crates need a common apply report without
database/catalog dependencies.
