# Evidence

## Implementation Summary

- `nako-library::ingestion` now returns each committed source observation's
  `SourceFingerprintEscalationDecision`.
- `LibraryIndexSummary` aggregates redaction-safe
  `ScanSourceFingerprintHashTrigger` values in-process only; the field is
  skipped from summary serialization.
- `nako-server::app::jobs` consumes committed trigger facts after successful
  index and delegates durable enqueue to `SourceFingerprintHashAppService`.
- `SourceFingerprintHashAppService` owns scan-originated policy mapping and
  idempotency for queued/running same library/source/mode jobs.
- Scan-originated jobs use existing `JobKind::SourceFingerprintHash`,
  `disk.scan.source_fingerprint_hash`, safe `SourceFingerprintHashJobInput`,
  and the existing disk-scan scheduler path.
- No Public API, DB schema, source-hash-specific runtime loop, duplicate
  relationship mutation, or Media Source merge behavior was added.

## Verification

- `cargo check -p nako-library --tests` passed.
- `cargo check -p nako-server --bin nako-server --tests` passed with existing
  warning noise.
- `cargo nextest run -p nako-library index_service_returns_redacted_source_hash_trigger_facts --no-fail-fast`
  passed.
- `cargo nextest run -p nako-server scan_originated_source_fingerprint_hash --no-fail-fast`
  passed.
- `cargo nextest run -p nako-server scan_library_enqueues_scan_originated_source_hash_after_weak_match --no-fail-fast`
  passed.
- `cargo nextest run -p nako-server source_fingerprint_hash --no-fail-fast`
  passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed. Git reported existing LF-to-CRLF working-copy
  conversion warnings only.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-scan-originated-source-hash-triggering`
  passed with 14 `implement.jsonl` and 14 `check.jsonl` entries.

## Notes

- A broad parallel `cargo check -p nako-library -p nako-server --tests` hit a
  Windows rustc/resource crash before returning a Rust diagnostic. Re-running
  focused checks with `CARGO_BUILD_JOBS=1` succeeded.
- Trellis workflow recommends dispatching Codex `trellis-implement` and
  `trellis-check` sub-agents, but the available multi-agent tool exposed no
  concrete Trellis subagent type in this session and its contract requires
  explicit user request for generic sub-agent use. The same Trellis phases were
  executed in the main session with spec context loaded and recorded here.
