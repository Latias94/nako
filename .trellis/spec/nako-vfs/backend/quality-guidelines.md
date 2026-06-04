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
- Keep local writes transactional or planned when the module supports it.
- Treat remote latency, range readability, rate limits, and writable capability
  as product behavior, not incidental adapter details.

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
- Cache stale/failure/repair diagnostic tests for cache changes.
- Repair action preview tests when cache classifications or operator guidance
  change.
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
