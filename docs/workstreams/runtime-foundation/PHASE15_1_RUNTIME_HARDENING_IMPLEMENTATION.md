# Phase 15.1: Runtime Hardening Implementation

## Summary

M15 hardens the runtime foundation without preserving MVP shortcuts that had
become shared risks. The implementation covers SQLite runtime behavior,
migration execution, secret redaction, and HLS hardware acceleration selection.

## SQLite Runtime

`nako-db` now has focused runtime and migration modules:

- `crates/nako-db/src/runtime.rs`
- `crates/nako-db/src/migrations.rs`

On-disk SQLite uses explicit runtime defaults:

- `create_if_missing(true)`
- `foreign_keys(true)`
- WAL journal mode
- normal synchronous mode
- 10 second busy timeout
- bounded pool with 8 max connections

In-memory SQLite remains a single-connection runtime so tests keep one shared
in-memory database instead of creating one database per connection.

This remains a single-process SQLite policy. Multi-process coordination is not
part of M15.

## Migration Execution

Migration execution now uses SQLx `Migrator` with embedded migration SQL. Nako
no longer splits migration files with `split(';')` and no longer maintains its
own `nako_schema_migrations` table.

SQLx owns:

- `_sqlx_migrations`
- migration checksums
- dirty migration detection
- transaction-wrapped migration application

Tests cover semicolons inside string literals and rollback behavior after a
failed migration.

## Secret Redaction

`nako-core::SecretString` is the shared resolved-secret wrapper. It redacts
`Debug`, `Display`, and serialization, and requires explicit
`expose_secret()` calls at integration boundaries.

The following resolved-secret paths use `SecretString`:

- TMDB read access token
- Bangumi access token
- Douban API key
- Douban custom header values
- metadata runtime proxy
- literal provider header values in server config

Diagnostics continue to expose booleans such as `proxy_configured` rather than
secret values. Tests cover provider config `Debug`, server config `Debug`, and
existing diagnostics responses.

## Hardware Capability Selection

HLS service construction now runs `select_hardware_acceleration` and stores the
selected acceleration. HLS command planning and runtime resource budgets use the
selected acceleration instead of the requested accelerator.

The current server detector is CPU-only. This is intentional for M15: the
runtime path now has the correct selection boundary, fallback behavior, and
failure semantics. A later phase can replace the CPU-only report with a real
FFmpeg capability probe without changing HLS orchestration.

Tests cover:

- available GPU detector selection in the server HLS service.
- GPU requested with CPU fallback uses CPU HLS planning.
- GPU requested with `fail` fallback rejects startup when unavailable.
- Existing HLS route and app-service tests continue to pass.

## Validation

Focused validation used during implementation:

```powershell
cargo test -p nako-db
cargo test -p nako-core -p nako-metadata -p nako-server redact -- --nocapture
cargo test -p nako-server hls_ -- --nocapture
cargo check -p nako-server --tests
```

Full workspace validation remains the close-out gate for M15.
