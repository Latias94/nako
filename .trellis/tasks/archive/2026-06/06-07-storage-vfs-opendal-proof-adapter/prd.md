# Storage VFS OpenDAL Proof Adapter

## Goal

Implement a low-risk OpenDAL-backed proof adapter inside `nako-vfs` to prove
ADR 0055's semantic mapping before any production storage backend rollout.

## Requirements

- Add an explicit, non-default `opendal-proof` feature to `nako-vfs`.
- Add OpenDAL with default features disabled and only the minimum proof-service
  support needed for tests.
- Implement `OpenDalStorageBackend` behind the feature.
- Keep it behind `StorageBackend`; do not expose OpenDAL to server, API,
  catalog, scan, playback, cache repair, or Admin layers.
- Support a memory-backed proof adapter for tests.
- Preserve Nako semantics:
  - `StorageUri` remains the external identity.
  - path mapping rejects credentials and traversal.
  - `stat` maps object kind, size, etag/fingerprint facts, and capabilities.
  - `list` returns direct directory children, not raw OpenDAL prefix results.
  - `read_range` and `stream_range` use centralized `ByteRange` validation.
  - error messages exposed through Nako types do not include raw provider
    authority details.
- Keep write/link/apply/cache/health/staging policy outside OpenDAL.

## Acceptance Criteria

- [x] `cargo check -p nako-vfs --features opendal-proof --tests` passes.
- [x] `cargo nextest run -p nako-vfs opendal --features opendal-proof --no-fail-fast` passes.
- [x] Default `cargo check -p nako-vfs --tests` still passes without OpenDAL.
- [x] `cargo fmt --all -- --check` passes or the touched package is formatted.
- [x] `git diff --check` passes.
- [x] Trellis task validation passes.

## Definition Of Done

- Code, tests, docs/task evidence, and feature wiring are committed together.
- No default dependency pull-in of OpenDAL.
- No product configuration, runtime scheduler, DB, API, or Admin behavior
  changes are included.

## Technical Approach

- Add `crates/nako-vfs/src/opendal.rs` gated by `#[cfg(feature =
  "opendal-proof")]`.
- Use OpenDAL's memory service as the only proof target.
- Export `OpenDalStorageBackend` only behind the feature.
- Write focused module tests for stat/list/range/stream/path rejection.
- Keep production S3/WebDAV replacement as a follow-on.

## Out Of Scope

- Production S3/WebDAV/OpenDAL backend configuration.
- Replacing `LocalFsBackend` or `WebDavBackend`.
- Cache repair or storage health integration.
- Admin/Public API exposure.
- Durable job or scheduler changes.
- Schema migrations.

## Technical Notes

- Decision baseline: `docs/adr/0055-opendal-storage-adapter-foundation.md`.
- Architecture route: `docs/architecture/STORAGE_VFS.md`.
- VFS specs:
  - `.trellis/spec/nako-vfs/backend/index.md`
  - `.trellis/spec/nako-vfs/backend/directory-structure.md`
  - `.trellis/spec/nako-vfs/backend/quality-guidelines.md`
  - `.trellis/spec/nako-vfs/backend/error-handling.md`
  - `.trellis/spec/nako-vfs/backend/logging-guidelines.md`
