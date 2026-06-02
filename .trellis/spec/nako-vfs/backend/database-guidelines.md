# Database Guidelines

`nako-vfs` does not own database adapters, but it does produce facts that may be
persisted by `nako-core` repository contracts and `nako-db` adapters.

## Rules

- Keep VFS persistence records in `nako-core` (`vfs_cache.rs`,
  `storage_health.rs`, staging records).
- Keep SQLite/Postgres persistence in `nako-db` (`sqlite/vfs_cache.rs`,
  `sqlite/vfs_health.rs`, `postgres/vfs_health.rs`, `postgres/vfs_staging.rs`).
- `nako-vfs` may call cache/storage abstractions passed into it, but should not
  import `nako-db` or `sqlx`.
- Persist redaction-safe cache and health facts, not raw provider errors that
  contain credentials or host-local details.

## Review Checklist

- Is the fact a VFS runtime result or a durable storage health/cache record?
- Does the durable record live in `nako-core`?
- Are DB adapters updated in `nako-db` if the durable shape changed?
- Is the diagnostic safe for Admin/Public exposure?
