# Directory Structure

`nako-vfs` implements storage backend access. It should expose storage facts and
safe streams to callers without letting app code depend on raw filesystem or
remote-provider internals.

## Current Layout

```text
crates/nako-vfs/src/
├── lib.rs                 # StorageUri, StorageBackend trait, metadata types
├── cache.rs               # CachedStorageBackend and cache policy
├── local.rs               # LocalFsBackend root
├── local/
│   ├── path_authority.rs  # local path authorization and resolution
│   ├── lifecycle.rs       # local lifecycle helpers
│   ├── apply_plan.rs      # planned local file operations
│   └── write_transaction.rs
└── webdav.rs              # WebDAV backend and secret resolver
```

## Module Rules

- Keep common storage contracts in `lib.rs`: `StorageUri`,
  `StorageCapabilities`, object metadata, cache repair diagnostics, and the
  backend trait.
- Keep cache wrapper behavior in `cache.rs`; do not duplicate stale fallback
  logic in individual callers.
- Keep local path resolution and write safety under `local/`.
- Keep remote WebDAV credential resolution in `webdav.rs` through
  `WebDavSecretResolver`.
- Keep VFS cache persistence records in `nako-core` and DB adapters in
  `nako-db`; `nako-vfs` should not implement repository storage itself.

## Forbidden Placement

- Do not expose raw local filesystem paths as public API identities. Use
  `StorageUri` and redacted diagnostics.
- Do not let addon, metadata, or playback code write directly to library paths.
  Use Nako-owned storage/write APIs.
- Do not treat remote storage as local path access. Use capabilities such as
  `RANGE_READABLE`, `REMOTE_LATENCY`, `RATE_LIMITED`, and `WRITABLE`.

## Examples

- `lib.rs`: `StorageUri::parse`, `StorageCapabilities`, object metadata, and
  cache repair diagnostic types.
- `cache.rs`: stale fallback and cache refresh behavior.
- `local/path_authority.rs`: local path authority.
- `webdav.rs`: remote backend adapter and secret resolver.
