# nako-vfs Backend Development Guidelines

These specs describe the storage/VFS adapter boundary in `crates/nako-vfs`.
VFS code owns storage backend access, cache/stale fallback behavior, local path
authority, WebDAV access, and redaction-safe storage diagnostics.

## Pre-Development Checklist

- Read [Directory Structure](./directory-structure.md) before adding storage
  backend, cache, or local path modules.
- Read [Database Guidelines](./database-guidelines.md) before touching VFS cache
  persistence records or repository contracts.
- Read [Error Handling](./error-handling.md) before adding storage failure
  classification or operator diagnostics.
- Read [Quality Guidelines](./quality-guidelines.md) before changing URI,
  backend capability, cache, or write transaction behavior.
- Read [Logging Guidelines](./logging-guidelines.md) before adding storage logs.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | StorageUri, backend traits, cache, local/WebDAV modules | Filled from code and ADR 0016 |
| [Database Guidelines](./database-guidelines.md) | VFS cache persistence handoff to `nako-core`/`nako-db` | Filled as adapter boundary |
| [Error Handling](./error-handling.md) | Storage failure classes and cache repair diagnostics | Filled from code |
| [Quality Guidelines](./quality-guidelines.md) | URI validation, capabilities, cache fallback, write safety | Filled from code |
| [Logging Guidelines](./logging-guidelines.md) | Redaction-safe storage diagnostics | Filled from code |

## Authority / Evidence

- ADR 0002: internal VFS before OS mounting.
- ADR 0016: remote storage and VFS cache boundary.
- `docs/architecture/STORAGE_VFS.md`
- `crates/nako-vfs/src/lib.rs`
- `crates/nako-vfs/src/cache.rs`
- `crates/nako-vfs/src/local/*.rs`
- `crates/nako-vfs/src/webdav.rs`
