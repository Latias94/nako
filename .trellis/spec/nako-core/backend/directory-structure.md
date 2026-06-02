# Directory Structure

`nako-core` owns domain contracts only. Add code here when multiple crates need
the same Nako concept, ID, repository trait, validation record, or persisted
enum.

## Current Layout

```text
crates/nako-core/src/
├── lib.rs                 # public re-export surface
├── id.rs                  # strong IDs used across crates
├── error.rs               # shared NakoError and Result
├── media/                 # media-library, item, source, probe, metadata records
├── repository/            # async repository traits and list filters
├── job.rs                 # durable job records and scheduler-visible policy
├── storage_health.rs      # storage health and circuit-breaker domain records
├── vfs_cache.rs           # VFS cache persistence records
└── *_policy.rs / *.rs     # feature domain records without adapters
```

## Module Rules

- Put provider-neutral domain records in a top-level module or a focused
  submodule such as `media/`.
- Put repository traits under `repository/` and keep their signatures in terms
  of core records, strong IDs, `PageRequest`, and `Result<T>`.
- Re-export public records from `lib.rs` so downstream crates do not depend on
  private module paths.
- Keep persisted enum strings in the enum module with paired `as_str` and
  `parse` or score conversion helpers. See `JobKind`, `JobStatus`, and
  `JobPriority` in `job.rs`.
- Keep domain terminology aligned with `CONTEXT.md`: use Media Source, Media
  Item, Provider Mapping, Playback Runtime, Addon, Generated Artifact, and
  Acceptance Workflow terms instead of provider-centric names.

## Forbidden Placement

- Do not import `sqlx`, Axum, tower, reqwest provider adapters, FFmpeg command
  builders, filesystem backends, or `tokio::spawn` into `nako-core`.
- Do not put server app services here. Application orchestration belongs in
  `nako-server/src/app/*`.
- Do not put database migrations or row mappers here. They belong in `nako-db`.
- Do not put storage backend implementation here. Storage adapters belong in
  `nako-vfs`.

## Examples

- `repository/jobs.rs`: repository traits split from durable job records.
- `job.rs`: persisted enum parse helpers plus durable job request/record types.
- `media/metadata.rs`: provider-neutral metadata records.
- `storage_health.rs`: storage health domain state without VFS implementation.
