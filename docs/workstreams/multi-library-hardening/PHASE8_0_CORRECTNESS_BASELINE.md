# Phase 8.0 Multi-Library Correctness Baseline

## Summary

M8 fixes correctness issues that appear once more than one configured library
exists. Library identity is now part of media-source identity, CLI operations
can target one library or all libraries explicitly, and remote staging budget
checks are serialized with manifest writes.

## Decisions

### Media source identity

`media_sources.locator` is not globally unique. Local libraries intentionally
emit library-relative locators such as `local:///Movie.mkv`, so two local
roots can produce the same locator for different physical files.

The durable identity rule is:

- `media_sources.id` remains the primary key;
- `(library_id, locator)` is the natural uniqueness constraint;
- repository APIs that look up a source by locator must also accept
  `library_id`;
- scan/index code must reuse existing sources only inside the target library.

This keeps local locators stable and readable without embedding host paths or
secrets into source URIs.

### CLI multi-library semantics

CLI commands that operate on library-scoped data must not silently imply
whole-server behavior.

The command-line contract is:

- `scan --library-id <id>` scans one library;
- `scan` without an ID is accepted only when exactly one library is configured;
- `scan-all` scans every configured library sequentially and returns a JSON
  array of per-library scan outputs;
- `list --library-id <id>` lists one library;
- `list` without an ID is accepted only when exactly one library is configured.

The CLI resolver returns an error for empty configs and for ambiguous
multi-library commands. It must not panic.

### Staging disk budget

Staging budget enforcement depends on current manifest state plus the incoming
object size. The check is only correct if the following sequence is atomic with
respect to other staging tasks:

1. compute deterministic target path and additional bytes;
2. read current manifest usage;
3. stage the object;
4. record the ready manifest.

M8 enforces this with a shared async mutex inside
`ManifestRecordingStorageBackend`. Stage concurrency still has its own
semaphore, but budget check, staging, and manifest recording are serialized so
two concurrent tasks cannot both pass the same budget snapshot.

## Implemented Artifacts

- `crates/taru-db/migrations/0001_initial.sql` creates
  `media_sources_library_locator_idx` on `(library_id, locator)`.
- `crates/taru-db/migrations/0015_media_source_library_locator.sql` updates
  existing development databases from the old global locator index.
- `MediaRepository::get_media_source_by_locator` requires `library_id`.
- `LibraryIndexService` queries existing sources with
  `(request.library.id, locator)`.
- `taru-server` CLI supports `scan --library-id`, `scan-all`, and
  `list --library-id`.
- `default_library_from_config` replaces the panic-style config helper.
- `ManifestRecordingStorageBackend` serializes budget check, stage, and
  manifest recording with a shared budget lock.

## Validation Focus

Required tests should prove:

- two local libraries with identical relative media paths create distinct
  source IDs and item IDs;
- source state, probe results, and search hits remain isolated by library;
- concurrent remote staging cannot exceed `[staging].max_bytes`;
- CLI and app code compile against explicit library selection APIs.

## Follow-Ups

- Provider runtime remains a separate metadata workstream: configured provider
  arrays, secret resolution, request timeout/retry/rate limiting, and persisted
  provider attempts for real Douban/Bangumi integrations.
- `taru-db`, `taru-server::app`, and `taru-server::http` still need broader
  file decomposition by domain.
