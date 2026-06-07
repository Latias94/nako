# Admin Web VFS Cache Repair Automation Commands

## Context

The overnight Jellyfin comparison highlighted that operator-visible maintenance
is a product capability, not only a backend scheduler detail. Nako now has
explicit Admin API routes for VFS cache repair automation dry-run and enqueue:

- `POST /admin/v1/storage/vfs-cache/repair/automation/plan`
- `POST /admin/v1/storage/vfs-cache/repair/automation/jobs`

Admin Web still only exposes latest refresh and first-target enqueue. The next
safe slice is to wire these new explicit commands into the existing Storage
Staging route without adding recurring scheduling or background execution.

## Goal

Expose VFS cache repair automation dry-run and explicit enqueue in Admin Web,
using generated Admin contract types and the existing `AdminApiClient` /
`AdminDataSource` boundaries.

## Requirements

- Add missing generated TypeScript contract types for the VFS cache repair
  automation request/response DTOs from `nako-api`.
- Regenerate both generated Admin contract files:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Add `AdminApiClient` methods for automation plan and automation enqueue.
- Add `AdminDataSource` methods and deterministic mock fallback for automation
  dry-run. Mutation must not fabricate success when live API is unavailable.
- Extend `StorageStagingPage` to show:
  - automation policy status;
  - eligible target count;
  - blocked target count/reasons;
  - boundary booleans;
  - a live-only explicit enqueue automation command.
- Keep the UI an operator control surface, not a scheduler:
  - no recurring interval;
  - no startup worker;
  - no direct job execution;
  - no cache purge/delete/invalidation controls.
- Do not render raw `StorageUri`, local paths, backend URLs, credentials, etags,
  fingerprints, URI digests, raw backend errors, durable input JSON, or cache
  payloads.

## Validation

- `npm run check --prefix apps/admin-web`
- `npm run test --prefix apps/admin-web -- --runInBand`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-08-06-08-admin-web-vfs-cache-repair-automation-commands`

## Non-Goals

- No recurring VFS repair scheduler policy.
- No direct repair job execution from Admin Web automation enqueue.
- No backend API behavior changes beyond generated Admin TypeScript contract
  type coverage.
- No broad Admin Web redesign.
