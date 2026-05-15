# Phase 6.2: Directory and Stat Cache

## Status

Completed.

## Objective

Add a persistent VFS directory/stat cache for remote storage metadata and make
remote transient listing failures non-destructive for catalog tombstones.

## Implemented

- Added VFS cache domain records in `taru-core`:
  - cached object metadata keyed by URI and scheme;
  - cached listing entries;
  - operation failure records for `stat` and `list`;
  - freshness timestamps in milliseconds.
- Added SQLite migration `0013_vfs_cache.sql` and `VfsCacheRepository`
  implementation in `taru-db`.
- Added `CachedStorageBackend` in `taru-vfs`:
  - caches remote stat/list results through the repository boundary;
  - returns fresh cache hits without calling the inner backend;
  - records transient storage failures;
  - serves stale cache entries on transient storage errors when configured.
- Extended `ObjectMetadata` and `ObjectListing` with cache status.
- Extended `VfsLibraryScanner` summaries with stale-cache detection.
- Updated `LibraryIndexService` so a stale-cache scan can update discovered
  sources but does not tombstone sources that were absent from the degraded
  scan.

## Validation

- `cargo test -p taru-vfs`
- `cargo test -p taru-db sqlite_store_round_trips_vfs_cache_records_and_failures`
- `cargo test -p taru-library index_service_does_not_tombstone_when_scan_uses_stale_vfs_cache`

## Boundary Notes

- VFS cache state remains separate from catalog `SourceState`.
- The cache stores URI, object kind, size, modified time, etag, fingerprint,
  capability bits, fetched time, freshness deadline, and failure state.
- Catalog tombstones still work for complete fresh scans.
- Stale cache fallback is restricted to storage errors, not invalid input or
  not-found responses.
- Cache timestamps are implementation-local millisecond values; user-facing
  freshness policy can be refined when HTTP/API exposure is added.

## Remaining Gaps

- No HTTP/API exposure for cache status yet.
- No cache eviction policy yet.
- No background refresh scheduler yet.
- No remote byte or staging cache yet; that belongs to M6.3 and M6.4.

## Next Step

Proceed to M6.3 remote probe staging so `ffprobe` can inspect remote sources
through deterministic local staging instead of requiring backend-local paths.
