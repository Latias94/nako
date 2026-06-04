# VFS Cache URI-Scoped Previews

## Goal

Extend the shipped latest-failure VFS cache repair preview/action-plan surface
into a redaction-safe target-scoped preview. Operators should be able to inspect
which repair target a plan applies to without exposing raw cache URI, source
locator, local path, backend URL, etag, fingerprint, credential, or raw backend
error body.

## What I Already Know

* VFS cache diagnostics, structured `recommended_action`, latest-failure
  refresh, and latest action plan are already shipped.
* `docs/architecture/STORAGE_VFS.md` now names URI-scoped previews and broader
  non-destructive remediation planning as the next VFS cache lane.
* Current Admin repair surfaces are latest-failure scoped:
  * `GET /admin/v1/storage/staging` includes `vfs_cache.repair`;
  * `GET /admin/v1/storage/vfs-cache/repair/action-plan` returns the latest
    action plan;
  * `POST /admin/v1/storage/vfs-cache/repair/refresh-cache` executes only the
    latest unresolved `refresh_cache` recommendation.
* Raw VFS cache failures contain `uri`, `operation`, `failed_at_ms`, `error`,
  and stored failure authority. The Admin contract explicitly forbids exposing
  raw cache URI, source locator, local path, backend URL, etag, fingerprint,
  token, credential, or raw backend error body.
* There is not yet a public-safe target handle or route for selecting a specific
  VFS cache repair target.

## Requirements

* Preserve the existing latest-failure preview, action plan, and refresh route.
* Keep the first URI-scoped slice read-only.
* Make target scope explicit without exposing raw URI or backend identity.
* Keep any target list bounded and paginated.
* Return the same repair diagnostic/action-plan vocabulary used by the latest
  action plan.
* Implement Option A first: a bounded target inventory with opaque target refs,
  plus a target-scoped preview that resolves those refs server-side.
* Target refs must be opaque, deterministic for the current unresolved failure
  record, and safe to show in Admin responses.
* Avoid cache purge/delete/invalidation, retry queues, durable jobs, and
  executable URI-scoped mutation in this task.

## Acceptance Criteria

* [ ] Operators can request or inspect a target-scoped repair preview without
      seeing raw URI/path/backend details.
* [ ] The preview response includes enough safe scope to distinguish targets
      operationally, such as scheme, operation, failed time, failure class, and
      an opaque target reference.
* [ ] Target selection is deterministic and rejects stale/unknown target refs
      without echoing unsafe input.
* [ ] Any new Admin DTO/route is generated from `nako-api` and reflected in both
      Admin TypeScript contract files.
* [ ] Focused tests cover redaction, target selection, stale/unknown target
      handling, Admin-only access, and no mutation.

## Definition of Done

* Focused `nako-api` contract/serialization tests pass.
* Focused `nako-server` route/service tests pass.
* Cross-crate `cargo check -p nako-api -p nako-server --tests` passes; broaden
  to `nako-core`/`nako-db` if repository contracts change.
* `cargo fmt --all -- --check` and `git diff --check` pass.
* Architecture/spec notes are updated if a reusable redacted target-handle
  pattern is introduced.

## Out of Scope

* Executing refresh for a selected target.
* Cache purge/delete/invalidation.
* Durable repair jobs or retry queues.
* Web UI implementation.
* Source fingerprint hash execution, playback artifact pressure, scan
  scheduling, or PostgreSQL runtime harness work.

## Feasible MVP Directions

### Option A: Bounded Target Inventory With Opaque Target Refs (Recommended)

Add a read-only Admin surface that lists recent unresolved VFS cache repair
targets with an opaque `target_ref`. A target-scoped preview can resolve that
opaque ref server-side and return the same redaction-safe diagnostic/action-plan
shape for that specific target.

Pros: closest to URI-scoped previews while preserving redaction; gives operators
a concrete target selector; can stay non-destructive. Cons: likely requires a
bounded repository/list service and careful stale-ref validation.

### Option B: Backend/Operation-Scoped Preview Only

Expose only coarse scopes such as source scheme, operation, and failure class,
without individual target refs.

Pros: lower leakage risk and smaller implementation. Cons: not truly
URI-scoped; weak operator value when multiple failures share the same coarse
scope.

### Option C: Schema-Backed Repair Target Records

Introduce durable repair target records or stable handles before adding preview
routes.

Pros: strongest long-term model for queues and future remediation. Cons: much
larger slice with schema, repository parity, and migration/testing overhead.

## Decision (ADR-lite)

Context: The current repair action plan is latest-failure scoped. A true
URI-scoped preview cannot expose raw cache URI, source locator, local path,
backend URL, etag, fingerprint, credential, or raw backend error body. Operators
still need a safe way to select a specific repair target when multiple cache
failures exist.

Decision: Implement Option A, a bounded Admin repair target inventory with
opaque `target_ref` values and a read-only target-scoped preview that resolves
those refs server-side. The first slice remains non-destructive and does not
execute refresh by selected target.

Consequences: This gives operators a concrete target selector while preserving
redaction. It may require adding bounded repository list/lookup behavior for VFS
cache failures. Schema-backed repair target records and URI-scoped executable
refresh remain follow-ons.

## Technical Approach

* Add Admin DTOs for a bounded repair target list and target-scoped preview.
* Add a safe target reference strategy that does not expose raw URI or backend
  identity.
* Add server storage service methods that list unresolved repair targets and
  resolve target refs before constructing existing repair diagnostic/action-plan
  output.
* Add Admin routes under `/admin/v1/storage/vfs-cache/repair/*` and generated
  contract entries.
* Add repository/query support only if existing VFS cache failure access cannot
  support bounded target selection.

## Technical Notes

* Existing latest repair code:
  * `crates/nako-server/src/app/storage.rs`
  * `crates/nako-server/src/http/admin.rs`
  * `crates/nako-api/src/admin/storage.rs`
  * `crates/nako-api/src/admin_contract.rs`
* Existing storage failure record shape:
  * `crates/nako-core/src/vfs_cache.rs`
* Related specs and architecture:
  * `docs/architecture/STORAGE_VFS.md`
  * `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
  * `.trellis/spec/nako-vfs/backend/index.md`
  * `.trellis/spec/nako-core/backend/index.md`
  * `.trellis/spec/nako-db/backend/index.md`
