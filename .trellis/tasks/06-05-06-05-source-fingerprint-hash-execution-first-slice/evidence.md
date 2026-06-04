# Evidence

## 2026-06-05

- Task opened after `source-fingerprint-hash-execution` remained the next
  storage follow-on and the previous escalation policy seam was reconciled as
  shipped.
- Added `nako-library::source_hash` with explicit partial/full modes:
  partial uses `StorageBackend::read_range` on a configured prefix and returns
  redaction-safe `BackendFingerprint` evidence; full uses
  `StorageBackend::stream_range(uri, None)` and returns redaction-safe
  `ContentHash` evidence.
- Updated `LocalFsBackend` so explicit `read_range` reads only the requested
  slice and `stream_range` streams local files in bounded chunks.
- Updated library/VFS specs and storage/library architecture maps to mark the
  execution kernel shipped while leaving scan/operator scheduling, queues,
  diagnostics, persistence, API, and automatic source merge behavior as
  follow-ons.
- Verification:
  - `cargo fmt --all -- --check`
  - `cargo check -p nako-library -p nako-vfs --tests`
  - `cargo nextest run -p nako-library source_hash --no-fail-fast`
  - `cargo nextest run -p nako-vfs local_backend_reads_byte_ranges local_backend_streams_byte_ranges --no-fail-fast`
  - `git diff --check` (only CRLF conversion warnings)
  - `python .trellis\scripts\task.py validate .trellis\tasks\06-05-06-05-source-fingerprint-hash-execution-first-slice`
