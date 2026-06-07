# Evidence

## Implementation Summary

- Added `opendal-proof` as an explicit non-default `nako-vfs` feature.
- Added optional OpenDAL workspace dependency with default features disabled and
  only `services-memory` enabled.
- Added `OpenDalStorageBackend` behind `#[cfg(feature = "opendal-proof")]`.
- Kept OpenDAL contained behind `StorageBackend`; no server/API/catalog/playback,
  cache repair, storage health, staging, or Admin behavior changed.
- Implemented a memory-backed proof adapter for tests only.
- Preserved Nako storage semantics:
  - `StorageUri` remains the adapter input and metadata identity.
  - Proof URI mapping rejects credentials, naked authority, query/fragment,
    traversal, dot segments, and backslashes.
  - `stat` maps OpenDAL metadata into Nako metadata, kind, capabilities, etag,
    and fingerprint fields.
  - `list` filters OpenDAL prefix results to direct children.
  - `read_range` and `stream_range` validate through `ByteRange`.
  - `stream_range` uses OpenDAL reader byte streams instead of reading the whole
    object into memory first.
  - OpenDAL errors map to safe Nako error kinds/messages.
- Recorded follow-on implementation contracts in
  `.trellis/spec/nako-vfs/backend/quality-guidelines.md`.

## Verification

All commands were run from `F:\SourceCodes\Rust\nako` on 2026-06-07.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo check -p nako-vfs --tests` | Passed |
| `cargo check -p nako-vfs --features opendal-proof --tests` | Passed |
| `cargo nextest run -p nako-vfs opendal --features opendal-proof --no-fail-fast` | Passed: 3 tests run, 3 passed |
| `cargo tree -p nako-vfs --no-default-features \| Select-String 'opendal\|opendal-core\|reqwest v0.13'` | Passed: no output |
| `cargo tree -p nako-vfs --features opendal-proof -i opendal` | Passed: shows `opendal v0.57.0 -> nako-vfs` |
| `python ./.trellis/scripts/task.py validate .trellis/tasks/archive/2026-06/06-07-storage-vfs-opendal-proof-adapter` | Passed |
| `git diff --check` | Passed |

## Spec Update Decision

This task introduced an optional storage integration boundary, so it triggered
Trellis code-spec depth. The reusable contract was added to the VFS quality
guidelines as `Scenario: Optional OpenDAL Adapter Boundary`.
