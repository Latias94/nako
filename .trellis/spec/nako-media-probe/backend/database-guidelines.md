# Database Guidelines

`nako-media-probe` does not persist anything. It produces probe facts for
callers that decide whether and where to store them.

## Required Patterns

- Return `MediaProbeResult` to the caller; do not write repositories,
  migrations, or database adapters here.
- Keep probe retries, durable failures, and persistence in `nako-library` or a
  durable job layer.
- Keep source identity as `StorageUri` input context, not as a database row.
- Keep parsing deterministic so persisted callers can compare probe results.

## Forbidden Patterns

- Do not depend on `nako-db`, SQL crates, server repositories, or migration
  helpers.
- Do not persist probe facts, failure records, or ffprobe stderr from this
  crate.
- Do not query media item/catalog state while probing.
- Do not couple ffprobe execution to a specific storage backend.

## Review Checklist

- Can the adapter be tested with only JSON bytes or a local path hint?
- Does a new behavior belong in library ingestion instead?
- Are persisted failure classes handled by callers rather than this crate?
- Does the dependency graph stay limited to core, VFS identity, serde, tokio,
  and async trait support?

## Evidence

- `crates/nako-media-probe/Cargo.toml`
- `crates/nako-media-probe/src/lib.rs`
