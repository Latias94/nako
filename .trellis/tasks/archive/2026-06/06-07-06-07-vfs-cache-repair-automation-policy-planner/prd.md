# VFS Cache Repair Automation Policy Planner

## Goal

Add an internal, redaction-safe, non-mutating VFS cache repair automation policy
planner. The planner prepares Nako for future scheduled repair automation by
classifying current unresolved repair targets into eligible and blocked groups,
without enqueueing jobs or touching storage backends.

## What I Already Know

- VFS cache repair already has target inventory, remediation planning, manual
  durable enqueue, manual execution, retry, scheduler execution, and Admin Jobs
  diagnostics.
- Jellyfin models maintenance as explicit scheduled tasks, but Nako should build
  on its own durable job and redaction-safe control-plane boundary.
- The safe first slice is a dry-run planner, not automatic execution.

## Requirements

- Add an internal app-level policy report in `crates/nako-server/src/app/storage.rs`.
- Default policy is disabled. Disabled policy must report unresolved pressure but
  no eligible targets.
- Enabled policy may mark only `refresh_cache` targets as eligible.
- Targets requiring backend configuration, manual failure inspection, or no
  action must be blocked with stable typed reasons.
- Report a non-mutating automation boundary:
  - reads existing repair targets;
  - may start durable jobs in a future caller only when enabled;
  - does not refresh cache, delete/purge/invalidate cache, mutate backend
    configuration, or write library files.
- Reuse existing redaction-safe target reports; do not expose raw URI/path,
  backend URL, credentials, etags, fingerprints, URI digests, or raw errors.
- Do not add Admin API, generated contract, frontend, schema, scheduler loop, or
  runtime behavior in this slice.

## Acceptance Criteria

- [x] Disabled policy reports zero eligible targets and blocks refreshable
      targets as policy disabled.
- [x] Enabled policy reports refreshable targets as eligible.
- [x] Enabled policy blocks non-refresh targets with typed reasons and no
      backend calls.
- [x] Planner scans unresolved failures through the existing target authority and
      remains redaction-safe.
- [x] Focused tests pass.
- [x] Formatting and `git diff --check` pass.

## Definition Of Done

- Code, tests, task evidence, campaign research, and relevant architecture/spec
  updates are committed together.
- No runtime automation, route, schema, or public contract change ships in this
  slice.

## Out Of Scope

- Automatic enqueue or execution.
- New Admin API or Admin Web UI.
- Cache purge/delete/invalidation execution.
- Backend configuration mutation.
- Library file writes.
- Source duplicate reconciliation.
