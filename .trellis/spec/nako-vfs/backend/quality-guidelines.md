# Quality Guidelines

Storage/VFS changes must protect path authority, remote-storage behavior, and
cache diagnostics.

## Required Patterns

- Use `StorageUri` for storage identity. Do not pass raw paths across API or
  cross-crate boundaries when a URI is the domain concept.
- Attach accurate `StorageCapabilities` to metadata so playback, scan, and
  staging can make policy decisions.
- Keep stale fallback explicit through cache status and repair diagnostics.
- Keep VFS cache repair operator guidance structured and redaction-safe:
  expose stable action enums for routing, keep display prose separate, and do
  not make clients parse free-form operator text.
- Keep read-only VFS cache remediation planning aggregate-only: group unresolved
  repair pressure by safe action/classification, expose bounded opaque
  `target_ref` samples, and keep the top-level boundary non-mutating even when
  a nested refresh group points to the existing selected-target refresh route.
- Keep durable VFS cache repair enqueue separate from repair execution:
  enqueue may create redaction-safe jobs from opaque unresolved targets, but it
  must not refresh, purge, delete, invalidate, mutate backend configuration, or
  write library files. Backend-touching repair work belongs in a future
  executor boundary.
- Keep local writes transactional or planned when the module supports it.
- Treat remote latency, range readability, rate limits, and writable capability
  as product behavior, not incidental adapter details.
- Implement explicit byte-range reads without forcing callers to load the whole
  object when the backend can seek. Full-object consumers that do not need local
  path hints should prefer `stream_range(uri, None)` over `read_range(uri,
  None)`.
- Keep byte-range validation centralized on `ByteRange`. Backends should use
  `ByteRange::validate_syntax` before constructing protocol range headers when
  object length is unknown, and `ByteRange::validate_for_len` /
  `ByteRange::resolved_len` when object length is known. Do not duplicate
  offset, zero-length, overflow, or open-ended length logic in individual
  backend adapters.

## Forbidden Patterns

- Do not bypass `LocalFsBackend` path authority with direct filesystem access
  from server/app code.
- Do not expose credentials, signed URLs, or raw local paths in object metadata
  or cache repair summaries.
- Do not hide storage health/circuit breaker behavior inside a feature caller.
  Use the storage health/control-plane boundary.
- Do not assume every backend is seekable, range-readable, or writable.

## Tests Required

- URI parsing and path normalization tests for new URI/path behavior.
- Backend capability tests for new backend adapters.
- Range and stream tests when changing `read_range` / `stream_range` behavior.
  Include explicit tests for bounded ranges, open-ended ranges, invalid syntax,
  and backend-specific range header/request construction.
- Cache stale/failure/repair diagnostic tests for cache changes.
- Repair action preview tests when cache classifications or operator guidance
  change.
- Remediation plan tests when unresolved repair pressure is grouped for Admin
  APIs; assert aggregate counts, bounded opaque samples, redaction, Admin guard,
  and that plan reads do not refresh, purge, delete, enqueue jobs, mutate
  backend configuration, or write library files.
- Durable repair enqueue tests must assert the persisted job input is
  redaction-safe, non-refresh targets reject without backend calls, queued and
  running duplicates are idempotent across paginated job results, and terminal
  jobs can be re-enqueued later.
- App/server integration tests when playback, scan, or Admin routes consume the
  new VFS behavior.

## Gate Selection

- Focused VFS:
  `cargo nextest run -p nako-vfs <filter> --no-fail-fast`
- Cross-crate storage/server:
  `cargo check -p nako-core -p nako-vfs -p nako-server --tests`

## Review Checklist

- Does the code preserve storage authority?
- Are capabilities accurate and tested?
- Is remote behavior bounded and observable?
- Are diagnostics redaction-safe?
- Do range-reading backends reuse `ByteRange` validation instead of carrying
  private range math?
