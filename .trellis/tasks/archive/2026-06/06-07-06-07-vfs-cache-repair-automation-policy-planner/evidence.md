# Evidence: VFS Cache Repair Automation Policy Planner

## Changes

- Added an internal `VfsCacheRepairAutomationPolicy` dry-run planner in
  `nako-server::app::storage`.
- The planner scans unresolved VFS cache repair targets through the existing
  redaction-safe target authority and classifies targets as eligible or blocked.
- Disabled policy blocks every target with `PolicyDisabled`.
- Enabled policy marks only `refresh_cache` targets eligible and blocks backend
  configuration / manual inspection / no-action targets with stable typed
  reasons.
- The automation boundary is explicitly non-mutating: the planner reads repair
  targets and never refreshes cache, starts jobs by itself, purges/deletes/
  invalidates cache entries, mutates backend configuration, or writes library
  files.
- Added focused app tests covering disabled policy, enabled eligible target
  reporting, and enabled non-refresh blocking without backend calls or job
  creation.
- Updated Storage/VFS and Control Plane maps plus `nako-server` quality spec so
  future automation work starts from the dry-run planner instead of jumping
  straight to execution.

## Verification

- `cargo nextest run -p nako-server vfs_cache_repair_automation_policy --no-fail-fast`
  passed.
- `cargo check -p nako-server --tests` passed.
- `cargo fmt --all -- --check` passed.

## Scope Notes

- No Admin API, generated contract, frontend, schema, scheduler loop, runtime
  automation, automatic enqueue, or storage mutation behavior changed.
- Raw `StorageUri`, local paths, backend URLs, credentials, raw errors, etags,
  fingerprints, URI digests, and job payloads remain outside the planner
  surface.
