# Storage VFS OpenDAL Adapter Decision Spike Evidence

## Result

Passed.

This task records an architecture decision only. It does not add an OpenDAL
dependency, update `Cargo.lock`, change runtime behavior, alter storage schema,
or change Admin/Public API shape.

## Decision

Apache OpenDAL is accepted as a future optional implementation adapter
foundation behind Nako's existing `StorageBackend` boundary.

It is not accepted as a replacement for:

- `StorageUri`;
- Source Locator redaction;
- `StorageCapabilities`;
- `ByteRange`;
- `StorageBackend`;
- VFS cache repair authority;
- storage health/circuit-breaker state;
- source fingerprint semantics;
- deterministic staging.

## Evidence Gathered

- `cargo info opendal`
  - Passed.
  - Reported `opendal` version `0.57.0`, license `Apache-2.0`, Rust version
    `1.85`, and default features including Tokio executor support plus retry,
    timeout, logging, and concurrent-limit layers.
- Official Apache OpenDAL docs checked:
  - https://opendal.apache.org/
  - https://opendal.apache.org/docs/rust/opendal/
  - https://opendal.apache.org/docs/rust/opendal/struct.Operator.html
  - https://docs.rs/crate/opendal/latest/source/CHANGELOG.md
- Existing Nako storage boundary inspected:
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/adr/0002-internal-vfs-before-os-mounting.md`
  - `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`
  - `crates/nako-vfs/src/lib.rs`
  - `crates/nako-vfs/src/local.rs`
  - `crates/nako-vfs/src/webdav.rs`
  - `crates/nako-vfs/Cargo.toml`

## Validation

- `python ./.trellis/scripts/task.py validate .trellis/tasks/archive/2026-06/06-07-06-07-storage-vfs-opendal-adapter-decision-spike`
  - Passed after archive: `implement.jsonl` 9 entries, `check.jsonl` 9 entries.
- `git diff --check`
  - Passed. Git reported LF/CRLF working-copy warnings for Markdown files only.

## Boundaries Preserved

- No dependency was added.
- No Rust source file was changed.
- No storage behavior was changed.
- No WebDAV/local adapter replacement was attempted.
- No cache repair, storage health, source fingerprint, staging, scheduler, or
  API behavior was moved into OpenDAL.

## Follow-On

Open a separate proof-adapter task before any production backend rollout. The
proof should use a low-risk OpenDAL service, preserve Nako directory-list and
range-read semantics, map errors to Nako-safe storage failure classes, and keep
OpenDAL optional behind the VFS adapter boundary.
