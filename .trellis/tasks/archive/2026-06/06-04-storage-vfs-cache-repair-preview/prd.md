# storage vfs cache repair preview

## Goal

Expose a redaction-safe VFS cache repair preview in Admin storage diagnostics so
operators can see the latest cache failure posture and recommended action
without adding cache mutation or delete semantics yet.

## What I Already Know

* `crates/nako-vfs/src/lib.rs` already defines `VfsCacheRepairDiagnostic` and
  redacted operator-action text for cache failures and stale fallback cases.
* `crates/nako-vfs/src/cache.rs` and the database backends already persist VFS
  cache objects, listings, and failures, but the repository trait has no
  "latest failure" helper yet.
* `crates/nako-server/src/app/storage.rs` already builds Admin storage
  diagnostics and has access to the shared store.
* `crates/nako-api/src/admin/storage.rs` already exposes `AdminVfsCacheSummary`
  inside `AdminStorageStagingSummary`.
* `docs/architecture/STORAGE_VFS.md` lists `vfs-cache-repair-operator-actions`
  as the next lane after the diagnostic slice.
* Existing storage Admin routes already follow the pattern of surfacing a
  redacted diagnostic summary without exposing raw paths or secrets.

## Assumptions

* This first slice is preview-only: no cache invalidation, delete, or refresh
  mutation will be added yet.
* The preview should stay global and redacted, derived from the latest VFS cache
  failure currently recorded in the database.
* The preview should live in the existing `storage/staging` Admin diagnostics
  response rather than adding a new route.

## Requirements

* Add a repository seam that can fetch the latest VFS cache failure record.
* Add an Admin DTO that mirrors the redacted repair diagnostic shape needed for
  operator preview.
* Surface the preview in the existing storage staging diagnostics response.
* Keep the payload redacted: no raw local paths, source locators, tokens, or
  backend URLs.
* Add focused tests for serialization and the preview mapping.
* Update the generated Admin contract artifact if the DTO shape changes.

## Acceptance Criteria

* [x] The store can return the latest VFS cache failure record.
* [x] Storage staging diagnostics include a VFS cache repair preview field.
* [x] The preview is redaction-safe and uses the existing repair-classification
  vocabulary.
* [x] Focused `nako-vfs`, `nako-db`, `nako-api`, and `nako-server` checks pass.
* [x] Admin contract artifacts are regenerated or updated if required.

## Definition Of Done

* Code is committed with a Conventional Commit message.
* Validation evidence is recorded in the task files.
* Any reusable convention learned from the preview is written back to spec.
* The task is archived after commit and the session is recorded.

## Out Of Scope

* No cache invalidation or delete endpoint yet.
* No URI-scoped repair action yet.
* No new public client routes.
* No retry queue, batch repair workflow, or background repair job.

## Technical Approach

Add a `get_latest_vfs_cache_failure` helper to the VFS cache repository
interface and its sqlite/postgres/memory implementations. Use that to derive a
redacted preview DTO in `nako-api`, and thread it into the existing
`AdminStorageStagingSummary.vfs_cache` payload. Keep the route surface stable
and preserve the current diagnostics-first behavior.

## Verification

* PASS: `cargo fmt --all -- --check`
* PASS: `git diff --check`
* PASS: `cargo check -p nako-core -p nako-vfs -p nako-db -p nako-api -p nako-server --tests`
* PASS: `cargo nextest run -p nako-db vfs_cache --no-fail-fast`
* PASS: `cargo nextest run -p nako-db sqlite_vfs_staging_contract_round_trips_listing_failures_and_summary --no-fail-fast`
* PASS: `cargo nextest run -p nako-api admin_vfs_cache --no-fail-fast`
* PASS: `cargo nextest run -p nako-api admin_contract --no-fail-fast`
* PASS: `cargo nextest run -p nako-server admin_v1_storage_staging_lists_filters_and_redacts_paths --no-fail-fast`
* PASS: `cargo nextest run -p nako-vfs cache --no-fail-fast`

## Spec Update

Updated `.trellis/spec/nako-api/backend/admin-and-public-contracts.md` with the
Admin VFS cache repair preview contract: preview-only scope, DTO fields,
redaction rules, generated contract requirements, and focused test gates.
