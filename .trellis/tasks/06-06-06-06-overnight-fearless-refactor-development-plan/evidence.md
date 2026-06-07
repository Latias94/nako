# Evidence

## 2026-06-07 M1 / Storage Planning

- Created and committed the continuous development plan:
  `1ce7cd0f chore(task): plan overnight fearless refactor development`.
- Recorded completed M1 evidence gates in roadmap/goal docs:
  `f8eac5fe docs(roadmap): record completed M1 evidence gates`.
- Recorded OpenDAL adapter-spike decision instead of adding a production
  dependency:
  `c035a813 docs(storage): record OpenDAL adapter spike decision`.

## 2026-06-07 VFS Byte Range Refactor

Commits:

- `07743077 refactor(vfs): centralize byte range validation`
- `4d1ec9d1 fix(vfs): validate open-ended WebDAV range length`
- `0f58cc0e fix(vfs): reject invalid WebDAV range syntax`

What changed:

- Moved shared byte-range boundary validation onto `ByteRange`.
- Removed duplicated local/WebDAV range validation helpers.
- Fixed WebDAV open-ended range reads so `bytes=1-` expects `object_len - 1`
  bytes rather than the whole object length.
- Added syntax validation before WebDAV constructs a `Range` header, so
  zero-length and overflowing ranges reject before sending invalid remote
  requests when object metadata lacks length.

Validation:

- `cargo nextest run -p nako-vfs byte_range --no-fail-fast`
  - Result: passed, 9 tests run after syntax-validation slice.
- `cargo nextest run -p nako-vfs webdav_backend_reads_open_ended_byte_ranges_with_resolved_length --no-fail-fast`
  - Result: passed, 1 test run.
- `cargo nextest run -p nako-vfs --no-fail-fast`
  - Result after open-ended fix: 58 tests passed.
  - Result after syntax-validation fix: 59 tests passed.
- `cargo check -p nako-vfs --tests`
  - Result: passed after both VFS range fixes.

## 2026-06-07 VFS Cache Repair Durable Enqueue

What changed:

- Added `JobKind::VfsCacheRepair` and the persisted
  `storage.vfs.cache_repair` resource class.
- Added `VfsCacheRepairJobInput` as the durable repair input contract, derived
  from existing `VfsCacheFailure` facts using source scheme, operation,
  failure timestamp/count, URI digest, and stored failure authority.
- Added internal `StorageDiagnosticsAppService::enqueue_vfs_cache_repair_target`
  for opaque unresolved repair targets that recommend `refresh_cache`.
- Kept enqueue non-mutating: no backend stat/list refresh, no purge/delete,
  no invalidation, no backend configuration mutation, and no library file write.
- Made queued/running enqueue idempotent by validated input equality, including
  duplicates beyond the first paginated durable job page; terminal jobs do not
  block future enqueue.
- Updated `docs/architecture/STORAGE_VFS.md`,
  `docs/architecture/CONTROL_PLANE.md`, and Trellis code-specs with the shipped
  contract and follow-on boundaries.

Boundaries:

- No Admin/Public API route or DTO was added.
- No durable repair executor, scheduler loop, retry/requeue route, schema
  migration, cache purge/delete/invalidation, backend configuration mutation,
  or automated repair worker was added.
- Raw `StorageUri`, local paths, backend URLs, credentials, raw backend errors,
  etags, fingerprints, and cache payloads remain outside durable job input.

Validation:

- `cargo check -p nako-server --tests`
  - Result: passed.
- `cargo nextest run -p nako-server vfs_cache_repair_target_enqueue --no-fail-fast`
  - Result: passed, 4 tests run.
- `cargo nextest run -p nako-core vfs_cache_repair --no-fail-fast`
  - Result: passed, 5 tests run.
- `cargo nextest run -p nako-server runtime_job_resource_class_mapping_maps_known_jobs_to_budget_classes --no-fail-fast`
  - Result: passed, 1 test run.
