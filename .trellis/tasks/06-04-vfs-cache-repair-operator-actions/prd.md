# VFS Cache Repair Operator Actions

## Goal

Turn the existing VFS cache repair diagnostics into a clearer operator action
surface without reopening already shipped refresh-cache behavior. The next
slice should help an administrator understand which remediation is executable,
which is plan-only, and which scope the action applies to, while preserving the
current redaction boundary around cache URIs, source locators, local paths,
backend URLs, etags, fingerprints, credentials, and raw backend errors.

## What I Already Know

* `docs/architecture/STORAGE_VFS.md` marks VFS cache diagnostics plus action
  preview as shipped, with follow-ons for executable refresh/remediation actions
  and broader URI-scoped previews.
* `AdminVfsCacheRepairAction` already contains `none`, `refresh_cache`,
  `fix_backend_configuration`, and `inspect_failure`.
* `/admin/v1/storage/vfs-cache/repair/refresh-cache` already exists and returns
  `AdminVfsCacheRefreshResponse`.
* `StorageAppService::refresh_latest_vfs_cache_repair` already refreshes the
  latest unresolved retryable failure, rejects non-refresh recommendations,
  resolves the preview when refresh succeeds, and uses recorded failure
  authority to avoid ambiguous backend targeting.
* Existing tests cover refresh success, non-refresh rejection, authority
  mismatch, ambiguous local target rejection, Admin-only HTTP access, and
  redaction of raw target details.

## Scope Correction

The initial candidate was described as an "executable refresh action" follow-on.
Repo inspection shows that slice has already landed. This task must not add a
second refresh-cache route or duplicate the existing latest-failure refresh
service.

The remaining product question is how much operator-action depth to add around
the existing action vocabulary.

## Requirements

* Preserve the existing refresh-cache route behavior and tests.
* Keep all operator action responses redaction-safe.
* Make the action boundary explicit enough that `fix_backend_configuration` and
  `inspect_failure` are not mistaken for destructive or automatic mutations.
* Prefer a small Admin-only surface that composes with `nako-api` DTOs and thin
  `nako-server` HTTP handlers.
* Avoid storage schema changes unless a confirmed MVP requires history, queueing,
  or explicit target selection beyond the latest unresolved failure.
* Implement Option A first: an Admin-only action plan for the latest unresolved
  VFS cache repair diagnostic.
* The plan must classify whether the recommended action is executable now or
  plan-only, and include redaction-safe readiness/reason codes.
* The plan must point executable `refresh_cache` recommendations at the existing
  refresh route rather than introducing another refresh mutation.

## Acceptance Criteria

* [ ] Existing refresh-cache behavior remains covered and unchanged.
* [ ] Operators can distinguish executable actions from plan-only actions.
* [ ] Responses do not expose cache URI, source locator, local path, backend URL,
      etag, fingerprint, token, credential, or raw backend error body.
* [ ] Any new Admin route or DTO is generated from `nako-api` and covered by
      focused contract tests.
* [ ] Focused `nako-server` tests cover auth/admin rejection and the relevant
      action/plan outcome.
* [ ] The plan response has deterministic behavior for no failure, executable
      refresh, plan-only backend-configuration remediation, and inspect-failure
      diagnostics.

## Definition of Done

* Focused unit/integration tests are added or updated where behavior changes.
* `cargo nextest` focused gates pass for changed crates.
* `cargo fmt --all -- --check` or `cargo fmt --all` is run when practical.
* Admin Web generated contract is refreshed if DTO or route inventory changes.
* Architecture/spec notes are updated if the operator-action boundary becomes a
  reusable convention.

## Out of Scope (Current)

* Adding another implementation of the already shipped latest-failure
  `refresh_cache` action.
* Cache purge/delete/invalidation actions.
* Durable repair jobs, operator queues, or scheduler-backed remediation.
* Web UI implementation.
* Playback artifact pressure, scan scheduling, source fingerprint hash
  execution, or PostgreSQL runtime harness work.

## Feasible MVP Directions Considered

### Option A: Action Plan Endpoint (Recommended)

Add or deepen an Admin-only plan response for the latest unresolved repair
diagnostic. It would classify whether the recommended action is executable now
(`refresh_cache`) or plan-only (`fix_backend_configuration`,
`inspect_failure`), include redaction-safe readiness/reason codes, and point the
operator at the existing refresh route when applicable.

Pros: low risk, clarifies the action seam, no destructive mutation, no schema
change. Cons: still only scopes to the latest unresolved failure.

### Option B: Explicit URI-Scoped Preview

Add an explicit request/response surface that previews remediation for a
redacted or server-resolved cache target rather than only "latest". The action
execution can remain limited to existing refresh semantics.

Pros: closer to the architecture note about URI-scoped previews. Cons: target
identity is sensitive, so this needs careful capability design and may require
new redacted handles or repository lookup semantics.

### Option C: Broaden Executable Remediation

Make additional actions executable beyond refresh, such as configuration-fix
acknowledgement or inspect-failure workflow state.

Pros: more complete operator workflow. Cons: likely needs product semantics,
state persistence, and stronger audit/authorization boundaries, so it is larger
than the current slice.

## Decision (ADR-lite)

Context: The already shipped route
`/admin/v1/storage/vfs-cache/repair/refresh-cache` covers the executable
refresh-cache mutation for the latest unresolved retryable cache failure.
Operators still need a stable way to understand whether the current
`recommended_action` is executable or advisory before invoking remediation.

Decision: Implement Option A, an Admin-only latest-repair action plan endpoint
or response surface. It should return a redaction-safe plan with action status,
readiness, reason codes, and the existing refresh route key/path when
`refresh_cache` is executable. `fix_backend_configuration` and
`inspect_failure` remain plan-only in this slice.

Consequences: This keeps the MVP small and non-destructive, avoids schema and
durable job work, and clarifies the operator action seam. The trade-off is that
explicit URI-scoped previews remain a later follow-on.

## Technical Approach

* Add `nako-api` Admin DTOs for a VFS cache repair action plan.
* Add an Admin route inventory entry generated from `nako-api`.
* Add a thin `nako-server` Admin handler that delegates to storage app service
  logic and maps to `nako-api` DTOs.
* Reuse the existing latest-failure diagnostic and refresh route; do not add a
  new mutation for `refresh_cache`.
* Keep VFS domain vocabulary in `nako-vfs`; put Admin wire readiness/reason
  types in `nako-api`.

## Implementation Plan

* PR1: Add DTO/contract route for latest VFS cache repair action plan.
* PR2: Add storage service planning logic and Admin HTTP handler.
* PR3: Add focused tests for no-failure, executable refresh, plan-only actions,
  non-admin rejection, generated contract output, and redaction.

## Technical Notes

* Existing service boundary: `crates/nako-server/src/app/storage.rs`.
* Existing Admin route boundary: `crates/nako-server/src/http/admin.rs`.
* Existing DTO/contract boundary: `crates/nako-api/src/admin/storage.rs` and
  `crates/nako-api/src/admin_contract.rs`.
* Existing VFS diagnostic vocabulary: `crates/nako-vfs/src/lib.rs` and
  `crates/nako-vfs/src/cache.rs`.
* Existing focused tests: `crates/nako-server/src/app/tests/storage.rs`,
  `crates/nako-server/src/http/tests/system.rs`, and
  `crates/nako-api/src/admin/storage.rs`.
* Related architecture/specs:
  * `docs/architecture/STORAGE_VFS.md`
  * `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
  * `.trellis/spec/nako-vfs/backend/index.md`
