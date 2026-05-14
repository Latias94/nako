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

Library pipeline tests:

- recursive VFS scanning
- idempotent indexing
- bounded probe concurrency
- probe failure isolation

Server tests:

- config parsing and defaults
- application-level scan job success
- health route
- library route
- scan job enqueue route
- paginated sources and items routes
- missing job and missing probe error mapping

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

- provider matching fixtures
- field lock merge behavior
- raw provider cache behavior
- NFO import/export round trips

Streaming and transcode:

- direct play decision fixtures
- FFmpeg command planning without running FFmpeg
- remux session lifecycle transitions
- remux runner success, failure, cancellation, timeout, and concurrency guard
- transcode session cancellation cleanup
- hardware capability detection with mocked probes

Remote storage:

- directory cache invalidation
- range-read retry behavior
- provider rate-limit behavior

Addons and automation:

- addon manifest validation
- timeout and retry policy
- authentication failure mapping
- secret-reference handling without plaintext leakage
