# Test Strategy

## Principles

- Prefer crate-level tests for domain and infrastructure boundaries.
- Use `cargo nextest run --workspace` as the default full test command.
- Keep tests deterministic and filesystem-isolated with temporary directories.
- Test idempotency for repeated jobs and scans.
- Test partial-failure behavior for bounded async pipelines.
- Add HTTP handler tests for every stable response envelope.

## Current Coverage

Core and model tests:

- typed ID round trips
- storage URI parsing
- media naming parser behavior

Infrastructure tests:

- local VFS traversal protection
- SQLite library, media item, media source, probe, stream, and job persistence
- job lifecycle state transitions
- metadata profile, catalog graph, artwork task, search projection, and
  transcode session persistence

Library pipeline tests:

- recursive VFS scanning
- idempotent indexing
- bounded probe concurrency
- probe failure isolation
- durable scan state and source tombstone updates

Metadata, NFO, catalog, and search tests:

- field lock merge behavior
- TMDB provider behavior with mocked responses
- provider priority and fallback execution
- raw provider cache behavior
- NFO import/export round trips
- NFO local-authority behavior
- catalog graph hydration for people, credits, tags, genres, studios, and images
- SQLite search projection rebuilds after scan, metadata refresh, and NFO import

Server tests:

- config parsing and defaults
- application-level scan job success
- health route
- library route
- scan job enqueue route
- paginated sources and items routes
- missing job and missing probe error mapping
- item detail, credits, images, people, tags, genres, and search browse routes
- metadata refresh and NFO import/export job routes
- direct play range, HEAD preflight, invalid range, and unsupported multi-range
  behavior
- remux application service lifecycle, runner failure, duplicate conflict,
  staged-output reuse, restart reuse, and HTTP route behavior
- persisted playback session lookup
- HLS playlist and segment routes, path traversal rejection, missing segment
  handling, active-session conflict, runner failure, duplicate conflict, and
  restart reuse
- hardware acceleration policy, fallback behavior, CPU/GPU budget selection,
  and FFmpeg argument planning without requiring real GPU hardware
- webhook endpoint and delivery-attempt persistence
- webhook signing, failed retry state, mocked local webhook receiver delivery,
  and HTTP configuration/inspection routes

## Gates

Run before committing meaningful code changes:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
```

For narrow Rust edits, it is acceptable to run a smaller package-specific check
first, then run the full gates before finalizing the task.

## Future Coverage

Metadata and NFO:

- TMDB series, season, and episode fixtures
- Douban provider MVP fixtures
- Bangumi provider MVP fixtures
- item-level metadata profile override behavior

Streaming and transcode:

- adaptive bitrate HLS ladder planning and route behavior
- transcode cancellation cleanup exposed through public app/API paths
- remote-source staging/cache behavior for FFmpeg
- real FFmpeg smoke tests outside the deterministic unit suite
- real VAAPI/NVENC/Quick Sync smoke tests outside CI

Remote storage:

- directory cache invalidation
- range-read retry behavior
- provider rate-limit behavior
- byte-range cache eviction and resumability

Addons and automation:

- addon manifest validation
- authentication failure mapping
- secret-reference handling without plaintext leakage
