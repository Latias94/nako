# Fearless Architecture Deepening — Milestones

Status: Active
Last updated: 2026-05-20

## M0 — Scope And Evidence Freeze

Status: completed.

Exit criteria:

- Workstream docs exist and agree.
- Architecture review findings are recorded.
- First executable task is selected.
- Non-goals prevent the lane from becoming provider/plugin/AI feature breadth.

Primary evidence:

- `DESIGN.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## M1 — Addon Side Effect Depth

Status: completed.

Exit criteria:

- Addon Principal/grant resolution, Side Effect intake, apply routing, and
  domain-specific apply behavior have clearer Modules with deeper Interfaces.
  Completed structurally by FAD-020.
- Behavior-preserving refactor evidence exists before semantic changes.
  Completed for FAD-020 with focused `taru-server` Addon Side Effect and addon
  HTTP tests.
- Addon Canonical Metadata writes are committed through a transactional seam
  that proves metadata/catalog/search/apply-outcome consistency.
  Completed by FAD-030.
- SQLite always-on and PostgreSQL opt-in evidence covers any new persistence
  commit semantics.
  SQLite evidence completed by FAD-030. PostgreSQL opt-in contract pair exists
  and was not run because `TARU_TEST_POSTGRES_URL` was unset.

Primary gates:

- `cargo check -p taru-core -p taru-db -p taru-server --tests`
- focused `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
- focused DB contract nextest for the new commit seam
- `git diff --check`

## M2 — Library Ingestion Workflow Depth

Status: completed.

Exit criteria:

- Library ingestion callers no longer coordinate scan/source/evidence/search
  persistence through a broad trait alias.
  Completed by FAD-040 with `LibraryIngestionWorkflow`.
- The M62 scan commit behavior remains covered by backend-neutral contracts.
  Preserved and re-run for SQLite scan commit contracts.
- Local Inference, VFS scanning, and commit behavior stay separate Modules.
  Preserved by keeping `LibraryScanner`, `LocalInferenceEngine`, and the
  `commit_library_scan_source` persistence seam separate while moving their
  orchestration behind the workflow Adapter.

Primary gates:

- `cargo check -p taru-library -p taru-db --tests`
- `cargo nextest run -p taru-db scan_commit --no-fail-fast`
- focused `taru-library` nextest
- PostgreSQL opt-in scan contract when available
- `git diff --check`

Evidence:

- `LibraryIndexService` now depends on `LibraryIngestionWorkflow` instead of
  the deleted broad `LibraryIndexRepository` alias.
- A fake workflow test proves index orchestration can be exercised without
  implementing low-level repository traits.
- SQLite scan commit contracts passed. PostgreSQL opt-in scan contract was
  skipped because `TARU_TEST_POSTGRES_URL` was unset.

## M3 — Playback And Transcode Readiness

Status: completed.

Exit criteria:

- Playback request/cache identity composes Playback Source Selection identity,
  Transcode Profile identity, source fingerprint/version, and runtime capability
  inputs intentionally.
  Completed by FAD-050 with `PlaybackProfileIdentity`,
  `TranscodeProfileIdentity`, `TranscodeSourceIdentity`, and
  `TranscodeRequestIdentity`.
- Multi-profile HLS reuse is not widened until the identity Interface is stable.
  Preserved: no adaptive bitrate or multi-profile HLS behavior was added.
- Hardware diagnostics distinguish static FFmpeg encoder discovery from device
  initialization and optional smoke-probe results.
  Completed by FAD-060 with separate encoder discovery, device initialization,
  and smoke-probe diagnostics records plus safe Admin API summaries.

Primary gates:

- `cargo check -p taru-streaming -p taru-transcode -p taru-server --tests`
- focused playback/profile identity nextest
- focused hardware diagnostics nextest
- `git diff --check`

Evidence:

- FAD-050 request/cache identity tests passed for streaming/transcode/server
  playback.
- FAD-060 hardware diagnostics tests passed for `taru-transcode`, Admin API
  DTO/contract serialization, the Admin playback runtime HTTP route, and
  admin-web TypeScript checking.

## M4 — Search Semantics And Test Locality

Status: active.

Exit criteria:

- Search semantics have a small evaluation harness before AI/vector search is
  introduced. Completed by FAD-070 with the shared `taru-search` evaluator and
  focused title/alias/provider-title/CJK fixtures.
- Search Projection versioning or equivalent discipline is documented and
  tested where needed. Completed for FAD-070 with current-version helpers on
  `SearchDocument` and `SearchEvaluationDocument`.
- Touched giant test families are split only when doing so improves locality and
  reviewability without weakening coverage. Pending FAD-080.

Primary gates:

- `cargo check -p taru-search -p taru-catalog -p taru-db --tests`
- focused search nextest
- focused nextest for refactored test families
- `cargo check --workspace --tests`
- `git diff --check`

## M5 — Closeout Or Split

Status: planned.

Exit criteria:

- All planned tasks are complete, or independent tails are split into named
  workstreams with clear gates.
- Documentation and ADRs reflect shipped Interfaces.
- Final workspace verification passes.
- The next implementation goal is recommended from evidence, not from a vague
  backlog note.

Primary gates:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- PostgreSQL opt-in contracts for touched persistence seams when available
- `git diff --check`
