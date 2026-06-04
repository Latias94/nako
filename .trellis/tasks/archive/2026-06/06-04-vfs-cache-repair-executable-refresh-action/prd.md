# VFS cache repair executable refresh action

## Goal

Ship the first executable VFS cache repair action by turning the existing
`recommended_action = refresh_cache` preview into a narrow Admin-triggered
refresh. The action should help operators recover stale or retryable VFS cache
state without adding purge/delete semantics, durable job orchestration, schema
changes, or Admin Web UI work.

## Background

The previous VFS cache repair action preview slice added a stable
`recommended_action` enum to redaction-safe repair diagnostics exposed through
`/admin/v1/storage/staging`. `refresh_cache` is currently only guidance. This
slice makes that action executable for the latest safe repair target while
preserving the same redaction and storage-boundary rules.

## Requirements

* Add one Admin-only executable action for VFS cache repair:
  `RefreshCache`.
* The action must only run when the current latest repair diagnostic recommends
  `refresh_cache`.
* The action must refresh the cache by re-running the existing VFS cache wrapper
  operation for the latest persisted failure target:
  object/stat refresh for object failures, listing refresh for listing failures.
* The action response must be structured and redaction-safe. It may expose
  action, operation, classification, success/failure status, retryable/failure
  class facts, and safe messages. It must not expose cache URI, source locator,
  local path, etag, fingerprint, credential, token, raw backend URL, or raw
  backend error.
* A successful refresh does not clear, archive, or rewrite existing VFS cache
  failure rows in this slice. Preview may treat a failure as resolved when the
  matching cached object/listing has `fetched_at_ms >= failed_at_ms`; persisted
  repair lifecycle state is a follow-on.
* The manual refresh is an explicit operator probe. It may bypass storage
  backoff admission for one refresh attempt, but must still record the backend
  success or failure through the existing health path.
* HTTP handlers must stay thin: translate Admin request/response and delegate to
  `StorageDiagnosticsAppService`.
* Keep VFS refresh behavior inside the VFS/cache boundary. Server code must not
  duplicate stale fallback logic or bypass storage backends with raw filesystem
  access.
* Failed refresh attempts must use the existing storage failure classification
  and public HTTP error mapping. Do not add a bespoke unclassified error string
  channel.
* Refresh generated Admin TypeScript contracts if Admin DTO shape changes.
* Update storage/VFS architecture notes only if the shipped status changes from
  preview-only to executable refresh first slice.

## Acceptance Criteria

* [ ] A POST Admin route exists for the executable VFS cache `RefreshCache`
      action and inherits Admin authentication.
* [ ] The route rejects non-admin principals through the existing Admin route
      guard.
* [ ] If no latest repair target exists, the action returns a client-safe
      not-found or invalid-input result.
* [ ] If the latest diagnostic recommends anything other than `refresh_cache`,
      the action rejects without touching the backend.
* [ ] For a retryable object/stat failure, the action calls the underlying
      backend through the cache wrapper, updates cached object state on success,
      and returns a redaction-safe success response.
* [ ] For a retryable listing failure, the action calls the underlying backend
      through the cache wrapper, updates cached listing state on success, and
      returns a redaction-safe success response.
* [ ] Successful refreshes no longer leave the latest resolved failure as an
      actionable `refresh_cache` preview, even though the failure row remains.
* [ ] A failed refresh returns existing storage failure semantics and does not
      leak cache URI, source locator, local path, etag, fingerprint, credential,
      token, backend URL, or raw backend error in the Admin response.
* [ ] No purge/delete action, durable job, schema migration, or Admin Web UI
      business logic is added.

## Technical Approach

Use the existing latest VFS cache failure as the action target authority. The
server app service should derive the current repair diagnostic from that
failure, verify that it recommends `RefreshCache`, then dispatch a typed refresh
operation through the registered cached backend. The cache wrapper should own
the actual `stat` or `list` call and cache upsert/failure-recording behavior.

If the existing cache wrapper does not expose an explicit refresh method, add a
small VFS-owned method that bypasses stale fallback for one requested operation
and refreshes the cache by calling the inner backend. Keep this method scoped to
cache repair and covered by VFS tests.

## Decision (ADR-lite)

Context: The architecture map names executable cache repair as the next VFS
follow-on, but durable repair workflows and URI-scoped batch operations are
larger control-plane work.

Decision: Implement a synchronous Admin-only `RefreshCache` action for the
latest redacted repair target and reuse existing cache refresh behavior. This
is intentionally a first executable slice, not a general repair job system.

Consequences: Operators get a real recovery action for the existing preview
without new persistence or UI complexity. The action is limited to the latest
repair target; broader URI-scoped previews, batching, purge/delete, durable
jobs, and richer UI are explicit follow-ons.

## Out Of Scope

* Purge, delete, invalidate, retry-all, or bulk cache operations.
* Durable jobs, repair queues, cancellation, progress polling, or job history.
* Database schema or repository contract changes.
* Admin Web UI business logic beyond generated contract updates.
* Source fingerprint hash execution.
* Playback artifact I/O pressure policy.
* Storage backend health reset or circuit breaker changes.
* Public Client API changes.

## Technical Notes

* Relevant architecture:
  * `docs/architecture/STORAGE_VFS.md`
  * `docs/architecture/CONTROL_PLANE.md`
  * `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`
  * `docs/adr/0053-application-control-plane-boundary.md`
* Relevant specs:
  * `.trellis/spec/nako-vfs/backend/index.md`
  * `.trellis/spec/nako-server/backend/index.md`
  * `.trellis/spec/guides/index.md`
* Previous slice:
  * `.trellis/tasks/archive/2026-06/06-04-06-04-vfs-cache-repair-action-preview-first-slice/prd.md`
* Expected code areas from initial inspection:
  * `crates/nako-vfs/src/cache.rs`
  * `crates/nako-vfs/src/lib.rs`
  * `crates/nako-server/src/app/storage.rs`
  * `crates/nako-server/src/http/admin.rs`
  * `crates/nako-server/src/http/tests/system.rs`
  * `crates/nako-api/src/admin/storage.rs`
  * generated Admin TypeScript contract if DTOs change.

## Verification Plan

* `cargo fmt --all -- --check`
* `cargo check -p nako-core -p nako-vfs -p nako-api -p nako-server --tests`
* `cargo nextest run -p nako-vfs cache --no-fail-fast`
* `cargo nextest run -p nako-api admin_vfs --no-fail-fast`
* `cargo nextest run -p nako-server storage --no-fail-fast`
* `git diff --check`
* `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-vfs-cache-repair-executable-refresh-action`
