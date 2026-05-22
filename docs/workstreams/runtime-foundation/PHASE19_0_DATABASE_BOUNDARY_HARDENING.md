# Phase 19.0: Database Boundary Hardening

## Summary

M19 hardens `nako-db` as the SQLite implementation behind Nako's repository
traits. The goal is not to introduce an ORM. Nako continues to use `sqlx` plus
domain repository traits because the current write paths need explicit
transactions, stable domain mappers, and focused SQL modules more than
ActiveRecord-style models.

## Decisions

- Do not add SeaORM.
- Keep `SqliteStore` as the concrete SQLite adapter.
- Keep `nako-core` repository traits as the boundary visible to application
  services.
- Keep SQL and SQLite transaction details inside `nako-db`.
- Remove oversized root-module responsibilities instead of preserving MVP file
  layout for compatibility.

## Repository Module Boundary

`crates/nako-db/src/lib.rs` now owns only the store type, module wiring, and a
few shared internal lookup helpers. Shared row mapping and SQL value encoding
live in `crates/nako-db/src/codec.rs`; root-level repository tests live in
`crates/nako-db/src/tests.rs`.

The previous mixed `jobs.rs` repository file was split by bounded context:

- `jobs.rs`
- `event_outbox.rs`
- `automation.rs`
- `webhooks.rs`
- `addons.rs`

This keeps repository review aligned with domain ownership and makes later
changes smaller.

## Transaction Boundaries

Scan indexing no longer writes discovered media through three separate service
calls. `ScanRepository::record_scanned_media_source` records the media item,
media source, and source state in one SQLite transaction. The library indexer
uses this repository-level operation directly.

Metadata refresh no longer updates the media item and raw provider response as
two independent service writes. `MetadataRepository::apply_metadata_refresh`
applies the merged item metadata and provider raw response in one SQLite
transaction and rejects mismatched raw-response item IDs before writing.

Existing transactional repository operations remain in place for catalog
external IDs, media probe streams, VFS listing entries, and staging budget
reservation.

## Runtime And Migration Evidence

The SQLite runtime and migration boundary was already hardened in M15 and stays
part of the M19 database baseline:

- on-disk stores use WAL
- foreign keys are enabled
- busy timeout is configured
- on-disk stores use a bounded pool
- in-memory stores remain single-connection
- migrations run through `sqlx::Migrator`
- migration tests cover semicolons inside SQL string literals and rollback

## Validation

Focused validation during M19:

```powershell
cargo test -p nako-db scan -- --nocapture
cargo test -p nako-db metadata -- --nocapture
cargo test -p nako-db -- --nocapture
cargo test -p nako-metadata strategy_ -- --nocapture
cargo check -p nako-library --tests
cargo check -p nako-metadata --tests
```

Full close-out validation:

```powershell
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace
git diff --check
```
