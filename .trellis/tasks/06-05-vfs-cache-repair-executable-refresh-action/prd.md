# VFS Cache Repair Executable Refresh Action

## Goal

Enable an Admin operator to execute `refresh_cache` for a selected VFS cache
repair target from the existing opaque target inventory. This closes the gap
between target-scoped preview and latest-failure refresh without adding purge,
delete, retry queue, or durable repair job behavior.

## Requirements

- Add an Admin-only target-scoped refresh route under the existing
  `/admin/v1/storage/vfs-cache/repair/targets/{target_ref}` surface.
- Resolve `target_ref` only through the current server-side opaque HMAC target
  lookup; never accept or expose a raw URI, Source Locator, backend URL, local
  path, etag, fingerprint, credential, or raw backend error.
- Execute only when the selected target is still unresolved and its repair
  diagnostic recommends `refresh_cache`.
- Reuse the existing VFS cache refresh behavior and stored failure authority so
  ambiguous local targets and mismatched backend authority remain rejected
  before any backend call.
- Keep refresh non-destructive: it may refresh VFS cache entries and update
  storage health through existing backend behavior; it must not delete cache
  rows, write library files, mutate backend configuration, start durable jobs,
  or create retry queues.
- Update target preview/action-plan output so a refreshable target advertises
  the new target-scoped executable route.
- Keep latest-failure refresh route behavior compatible.
- Update Admin contract route inventory and generated TypeScript contracts.

## Acceptance Criteria

- [x] A refreshable target preview returns an executable `refresh_cache` plan
  whose route key/path point at the target-scoped refresh route.
- [x] `POST /admin/v1/storage/vfs-cache/repair/targets/{target_ref}/refresh-cache`
  refreshes the selected unresolved cache target and clears that target from
  subsequent target inventory when the cache entry proves newer than the
  failure.
- [x] Stale, unknown, invalid, or already-resolved target refs return not found
  without echoing the unsafe input or raw target data.
- [x] Non-refresh recommendations fail without backend calls.
- [x] Non-admin sessions are forbidden for the new target-scoped mutation.
- [x] Responses and errors do not contain raw URI/path/backend/etag/fingerprint
  or secret-bearing values.
- [x] `cargo fmt --all`, focused `nako-api` Admin contract tests, and focused
  `nako-server` VFS cache repair tests pass.

## Definition of Done

- Tests added or updated for app service behavior, HTTP route behavior, Admin
  route inventory, and redaction.
- Generated Admin Web TypeScript contract files refreshed from `nako-api`.
- Trellis specs updated if the task creates a durable contract change; otherwise
  record why no new spec is needed.

## Technical Approach

Add a target-scoped app service method that validates and resolves the opaque
target ref by scanning unresolved failures, then passes the matched
`VfsCacheFailure` into a shared refresh helper. The shared helper will contain
the existing latest-failure validation and backend-authority resolution.

Add a new Admin route:

`POST /admin/v1/storage/vfs-cache/repair/targets/{target_ref}/refresh-cache`

The response can reuse `AdminVfsCacheRefreshResponse` because the wire result is
the same redaction-safe refresh action report. The action plan executable route
will use a templated path containing `{target_ref}` and the stable route key.

## Decision (ADR-lite)

**Context**: The previous target preview route intentionally stayed read-only
because selected-target mutation semantics were not yet designed. The current
task supplies that dedicated design slice.

**Decision**: Implement only selected-target `refresh_cache` execution, backed
by existing HMAC target refs, stored failure authority, and cache refresh
behavior.

**Consequences**: Operators can remediate a specific unresolved retryable VFS
cache failure without being forced onto the latest failure. Broader remediation
such as purge/delete, retry queues, durable repair jobs, backend configuration
changes, and UI workflow remains explicit follow-on work.

## Out of Scope

- Cache purge, delete, invalidation, or row cleanup.
- Durable repair queues or background jobs.
- Backend configuration mutation.
- Library file writes.
- Public Client API exposure.
- Admin Web UI screens or frontend workflow beyond generated contract refresh.

## Research References

- [`research/vfs-cache-selected-target-refresh.md`](research/vfs-cache-selected-target-refresh.md)
  records the local code/spec audit and implementation boundary.

## Technical Notes

- Relevant specs:
  - `.trellis/spec/nako-vfs/backend/index.md`
  - `.trellis/spec/nako-api/backend/index.md`
  - `.trellis/spec/nako-server/backend/index.md`
  - `.trellis/spec/guides/index.md`
- Relevant architecture:
  - `CONTEXT.md`
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`
- Existing app service methods:
  - `list_vfs_cache_repair_targets`
  - `preview_vfs_cache_repair_target`
  - `refresh_latest_vfs_cache_repair`
