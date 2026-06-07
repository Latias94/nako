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
  write library files. Backend-touching refresh repair work must go through the
  storage app durable executor boundary, revalidate current unresolved failure
  authority before backend calls, and keep summaries/errors redaction-safe.
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

## Scenario: Optional OpenDAL Adapter Boundary

### 1. Scope / Trigger

- Trigger: adding or extending an OpenDAL-backed storage adapter in `nako-vfs`.
- Scope: adapter module placement, feature wiring, URI/path mapping, metadata
  mapping, list semantics, range reads, stream reads, and error mapping.

### 2. Signatures

- Feature flag: `opendal-proof` or a future explicit backend-specific feature.
- Public export: an OpenDAL adapter type may be exported only behind that
  explicit feature.
- Backend contract: adapter implementations must satisfy `StorageBackend`
  without exposing `opendal::Operator` outside `nako-vfs`.

### 3. Contracts

- `StorageUri` remains the only external object identity.
- OpenDAL paths are derived from Nako path-only URI forms such as
  `scheme:///Movies/Demo.mkv`; authority, credentials, query, fragment,
  traversal, `.` segments, and backslash separators must be rejected before
  calling OpenDAL.
- OpenDAL metadata must be translated into `ObjectMetadata`,
  `ObjectKind`, `StorageCapabilities`, safe etag/fingerprint fields, and
  `cache: None` unless the VFS cache wrapper supplies cache state.
- `list` must narrow prefix-listing results to direct directory children.
- `read_range` and `stream_range` must validate ranges through `ByteRange`
  and map OpenDAL range errors into safe Nako storage errors.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Wrong scheme | `NakoError::InvalidInput` |
| URI has authority, credentials, query, or fragment | `NakoError::InvalidInput` |
| URI path has traversal or backslashes | storage security violation |
| OpenDAL `NotFound` | `NakoError::NotFound` |
| OpenDAL `PermissionDenied` | `StorageErrorKind::Unauthorized` |
| OpenDAL `RateLimited` | `StorageErrorKind::RateLimited` |
| OpenDAL `RangeNotSatisfied` or short read | `StorageErrorKind::StagingValidationMismatch` |

### 5. Good/Base/Bad Cases

- Good: `opendal:///Movies/Demo.mkv` maps to `Movies/Demo.mkv`, validates a
  bounded `ByteRange`, and streams through OpenDAL without reading the whole
  object first.
- Base: `opendal:///Movies/` lists only direct child files and child
  directories under `Movies/`.
- Bad: `opendal://host/Movies/Demo.mkv` or
  `opendal://user:password@host/Movies/Demo.mkv` is accepted as an object path.

### 6. Tests Required

- Feature-gated compile tests for the OpenDAL adapter and default compile tests
  proving OpenDAL is not required without the feature.
- URI rejection tests for credentials, naked authority, query/fragment, path
  traversal, dot segments, and backslashes.
- Metadata/capability tests for file and directory entries.
- Listing tests that include nested files and assert only direct children are
  returned.
- Range and stream tests for bounded ranges, full streams, invalid ranges, and
  short-read or provider range failures when the service can trigger them.

### 7. Wrong vs Correct

#### Wrong

```rust
let path = uri.path_part().trim_start_matches('/');
operator.read(path).await?;
```

#### Correct

```rust
let raw_path = uri.path_part();
if !raw_path.starts_with('/') || uri.as_str().contains('@') {
    return Err(NakoError::InvalidInput {
        message: "OpenDAL uri must not contain authority".to_owned(),
    });
}
let range = range.validate_for_len(uri, object_len)?;
let end = range.offset + range.resolved_len(uri, object_len)?;
operator.read_with(path).range(range.offset..end).await?;
```

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
- Durable repair executor tests must assert explicit-job lease claiming,
  current unresolved failure revalidation, selected-target refresh authority,
  stale-input rejection without backend calls, redaction-safe summary JSON, and
  redacted storage failure persistence.
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
