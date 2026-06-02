# Logging Guidelines

VFS logs and diagnostics must be storage-safe.

## Rules

- Do not log raw local paths, WebDAV credentials, signed URLs, or request
  headers.
- Prefer storage URI scheme, operation kind, failure class, retryability, and
  safe operator action over raw error text.
- Cache repair messages exposed to operators must be redaction-safe.
- Keep high-volume listing/read logs behind deliberate tracing decisions; large
  libraries and remote storage can produce noisy logs.

## Evidence

- `crates/nako-vfs/src/lib.rs`
- `crates/nako-vfs/src/cache.rs`
- `docs/architecture/STORAGE_VFS.md`
