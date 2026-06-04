# Local VFS cache repair target authority options

Date: 2026-06-04

## Problem

The executable VFS cache repair refresh action currently resolves the latest
failure target by parsing `VfsCacheFailure.uri` and matching it against
configured library roots. That is safe for many WebDAV cases, but local VFS
URIs are library-root-relative (`local:///...`). With multiple local libraries,
every local failure matches the broad `local:///` root, so the action rejects as
ambiguous rather than selecting a backend.

The current behavior is safer than guessing, and
`test(storage): cover ambiguous vfs cache repair target` now locks it down.
This task should make the backend authority explicit for new failures.

## Existing flow

* `CachedStorageBackend` records cache failures through
  `NewVfsCacheFailure { uri, scheme, operation, failed_at_ms, error }`.
* `LibraryStorageBackend` wraps the cached backend and knows the `library_id`
  and `backend_key`, but the failure is already recorded inside the inner cache
  wrapper.
* The Admin repair action reads `get_latest_vfs_cache_failure()`, verifies the
  diagnostic recommends `RefreshCache`, then calls
  `backend_for_vfs_cache_failure()`.
* `backend_for_vfs_cache_failure()` currently falls back to URI/root matching.

## Options

### Option A: Persist optional authority on VFS cache failures

Add optional `library_id` and `backend_key` fields to
`NewVfsCacheFailure` / `VfsCacheFailure`, inject them into
`CachedStorageBackend` options when the server constructs a per-library cached
backend, and make repair resolution prefer `library_id/backend_key` over URI
prefix matching.

Pros:

* Fixes the actual Admin repair target ambiguity.
* Keeps URI/path/backend details internal and redaction-safe.
* Preserves legacy rows through nullable fields and fallback matching.
* Avoids duplicate failure recording in `LibraryStorageBackend`.
* Avoids full VFS cache object/listing key migration in this first slice.

Cons:

* Does not fully partition cached object/listing rows by backend authority.
* Existing legacy rows without authority can still be ambiguous.

### Option B: Infer local backend by configured filesystem root

Parse the local URI and compare it to configured `LocalLibraryConfig.root`.

Rejected:

* Local VFS URIs are relative to the backend root; they do not carry the host
  filesystem root.
* Matching host paths in the repair layer risks leaking or depending on raw
  local paths.

### Option C: Fully partition VFS cache objects/listings/failures by backend

Introduce an authority key into all VFS cache tables and repository lookups so
cache objects, listings, listing entries, and failures are all backend-scoped.

Pros:

* Architecturally complete for future local cache enablement.
* Prevents cache row collisions for same relative local URIs across libraries.

Cons:

* Requires table rebuilds and broad repository contract churn.
* Much larger than the current repair-action target bug.
* Local caching is not enabled in production backend construction today.

## Recommendation

Use Option A for this task.

This solves the current executable repair action issue with a bounded schema and
repository update. Record Option C as a follow-on if Nako later enables local VFS
cache by default or needs backend-scoped cache object/listing rows.

## MVP contracts

* New cache failures produced from a server-configured per-library cached
  backend include `library_id` and `backend_key`.
* Admin repair target resolution first uses persisted authority.
* If authority is missing, the existing URI/root matching remains as a legacy
  fallback.
* If authority points to a missing library or mismatched backend key, return a
  redaction-safe error.
* Admin responses still do not expose URI, library root, host path, backend
  URL, credentials, etag, fingerprint, or raw storage errors.
* Full cache object/listing partitioning remains out of scope.
