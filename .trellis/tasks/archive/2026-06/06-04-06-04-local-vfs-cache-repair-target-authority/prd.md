# Local VFS cache repair target authority

## Goal

Make executable VFS cache repair refresh target selection authority-aware, so a
new cache failure recorded for a configured library can be refreshed through the
correct backend even when URI/root prefix matching would be ambiguous.

## Background

The previous task shipped
`POST /admin/v1/storage/vfs-cache/repair/refresh-cache` for the latest repair
diagnostic whose `recommended_action` is `RefreshCache`. The action is
redaction-safe and does not accept raw URI input from clients.

The current backend lookup still derives authority from the latest failure URI.
That is intentionally conservative: if a local target could match multiple
configured local backends, the action returns a safe conflict. The follow-up is
to make the durable failure row carry safe backend authority for new failures,
so the server can stop guessing from URI prefixes.

## What I Already Know

* `CachedStorageBackend` records cache failures before the outer
  `LibraryStorageBackend` can attach backend identity.
* `LibraryStorageBackend` knows both `library_id` and `backend_key`.
* Local VFS URIs are backend-root-relative and use the same `local` scheme
  across libraries, so `local:///...` is not enough to select a configured
  backend.
* Existing Admin repair responses must remain redaction-safe.
* The previous task intentionally avoided schema changes. This task may add a
  narrow schema/repository extension because target authority is durable state.
* Full VFS cache object/listing partitioning is a larger follow-on. This slice
  only fixes repair target authority for failures.

## Requirements

* Extend VFS cache failure records with optional safe target authority:
  `library_id` and `backend_key`.
* Preserve compatibility with existing failure rows that lack authority.
* Inject authority when the server constructs a per-library cached backend.
* Keep generic VFS cache users working with `None` authority by default.
* Update SQLite and PostgreSQL schemas, migrations, row mappers, repository
  adapters, and contract tests for the new fields.
* Make `StorageBackendRegistry::backend_for_vfs_cache_failure` prefer persisted
  authority over URI/root matching.
* Validate persisted authority before use:
  * missing configured library returns a safe not-found error;
  * mismatched backend key or scheme returns a safe conflict/invalid-input
    error;
  * no raw URI, library root, host path, backend URL, credentials, etag,
    fingerprint, or raw backend error appears in public/Admin errors.
* Keep the legacy URI/root matching fallback for rows whose authority is absent.
* Update app/server tests so a new local failure with authority refreshes the
  correct backend when multiple local libraries are configured.
* Keep the existing ambiguous legacy local-target rejection test.

## Acceptance Criteria

* [ ] New `VfsCacheFailure` rows can round-trip optional `library_id` and
      `backend_key` through SQLite and PostgreSQL contract tests.
* [ ] Migrated SQLite and PostgreSQL stores contain the new failure-authority
      columns.
* [ ] Server-created cached WebDAV backends record cache failures with
      authority.
* [ ] Test-configured cached local backends can record local failures with
      authority.
* [ ] `refresh_latest_vfs_cache_repair()` uses authority to select the intended
      backend when multiple local libraries exist.
* [ ] Legacy failures without authority still use the existing fallback and
      remain safe on ambiguous matches.
* [ ] Authority mismatch/missing-library cases reject without backend calls and
      do not expose raw targets.
* [ ] Admin HTTP repair action behavior remains redaction-safe and still rejects
      non-admin principals through existing route guards.

## Out Of Scope

* Full VFS cache object/listing partitioning by backend authority.
* Enabling production local VFS caching by default.
* Purge/delete/invalidate/bulk repair actions.
* Durable jobs, repair queues, cancellation, progress polling, or repair
  history.
* Admin Web UI changes beyond generated contract updates if they become
  necessary.
* Public Client API changes.
* Storage health reset or manual circuit-breaker reset.

## Technical Approach

Use the recommended Option A from
[`research/target-authority-options.md`](research/target-authority-options.md):

1. Add a small authority value in `nako-core` for VFS cache failures, or add
   optional `library_id` / `backend_key` fields directly to
   `NewVfsCacheFailure` and `VfsCacheFailure`.
2. Extend `VfsCacheOptions` in `nako-vfs` with optional failure authority.
   `CachedStorageBackend::new` remains authority-less by default.
3. When `nako-server` builds a cached backend for a configured library, use
   `storage_backend_key(config.id, scheme)` and `config.id` to populate the
   authority.
4. Add SQLite/PostgreSQL migration `0004_vfs_cache_failure_authority` and update
   baselines/registration.
5. Update inserts, upserts, selects, and row mappers for the new fields.
6. In `backend_for_vfs_cache_failure`, resolve in this order:
   * authority present -> configured library id and backend-key validation;
   * authority absent -> current URI/root fallback.
7. Keep all repair responses mapped through existing redaction-safe DTOs and
   error mapping.

## Research References

* [`research/target-authority-options.md`](research/target-authority-options.md)
  - compares persisted failure authority, path inference, and full cache
  partitioning.
* Previous executable refresh task:
  `.trellis/tasks/archive/2026-06/06-04-vfs-cache-repair-executable-refresh-action/`.

## Technical Notes

Relevant architecture and ADRs:

* `docs/architecture/STORAGE_VFS.md`
* `docs/architecture/CONTROL_PLANE.md`
* `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`
* `docs/adr/0053-application-control-plane-boundary.md`

Relevant specs:

* `.trellis/spec/nako-vfs/backend/index.md`
* `.trellis/spec/nako-vfs/backend/database-guidelines.md`
* `.trellis/spec/nako-vfs/backend/error-handling.md`
* `.trellis/spec/nako-vfs/backend/quality-guidelines.md`
* `.trellis/spec/nako-server/backend/index.md`
* `.trellis/spec/nako-server/backend/database-guidelines.md`
* `.trellis/spec/nako-db/backend/index.md`
* `.trellis/spec/nako-db/backend/database-guidelines.md`

Expected code areas:

* `crates/nako-core/src/vfs_cache.rs`
* `crates/nako-core/src/repository/vfs.rs`
* `crates/nako-vfs/src/cache.rs`
* `crates/nako-vfs/src/lib.rs`
* `crates/nako-db/migrations/`
* `crates/nako-db/src/sqlite/vfs_cache.rs`
* `crates/nako-db/src/sqlite/codec.rs`
* `crates/nako-db/src/postgres/vfs_staging.rs`
* `crates/nako-db/src/postgres.rs`
* `crates/nako-db/src/contract_tests.rs`
* `crates/nako-server/src/app/storage.rs`
* `crates/nako-server/src/app/tests/storage.rs`
* `crates/nako-server/src/http/tests/system.rs`

## Verification Plan

* `cargo fmt --all -- --check`
* `cargo check -p nako-core -p nako-vfs -p nako-db -p nako-server --tests`
* `cargo nextest run -p nako-db vfs_cache --no-fail-fast`
* `cargo nextest run -p nako-vfs cache --no-fail-fast`
* `cargo nextest run -p nako-server vfs_cache_refresh_action --no-fail-fast`
* `git diff --check`
* `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-06-04-local-vfs-cache-repair-target-authority`
