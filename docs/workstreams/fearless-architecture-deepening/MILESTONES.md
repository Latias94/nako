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

Status: in progress.

Exit criteria:

- Addon Principal/grant resolution, Side Effect intake, apply routing, and
  domain-specific apply behavior have clearer Modules with deeper Interfaces.
  Completed structurally by FAD-020.
- Behavior-preserving refactor evidence exists before semantic changes.
  Completed for FAD-020 with focused `taru-server` Addon Side Effect and addon
  HTTP tests.
- Addon Canonical Metadata writes are committed through a transactional seam
  that proves metadata/catalog/search/apply-outcome consistency.
  Pending FAD-030.
- SQLite always-on and PostgreSQL opt-in evidence covers any new persistence
  commit semantics.
  Pending FAD-030 because FAD-020 changed server Module structure only and did
  not add a persistence seam.

Primary gates:

- `cargo check -p taru-core -p taru-db -p taru-server --tests`
- focused `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
- focused DB contract nextest for the new commit seam
- `git diff --check`

## M2 — Library Ingestion Workflow Depth

Status: planned.

Exit criteria:

- Library ingestion callers no longer coordinate scan/source/evidence/search
  persistence through a broad trait alias.
- The M62 scan commit behavior remains covered by backend-neutral contracts.
- Local Inference, VFS scanning, and commit behavior stay separate Modules.

Primary gates:

- `cargo check -p taru-library -p taru-db --tests`
- `cargo nextest run -p taru-db scan_commit --no-fail-fast`
- focused `taru-library` nextest
- PostgreSQL opt-in scan contract when available
- `git diff --check`

## M3 — Playback And Transcode Readiness

Status: planned.

Exit criteria:

- Playback request/cache identity composes Playback Source Selection identity,
  Transcode Profile identity, source fingerprint/version, and runtime capability
  inputs intentionally.
- Multi-profile HLS reuse is not widened until the identity Interface is stable.
- Hardware diagnostics distinguish static FFmpeg encoder discovery from device
  initialization and optional smoke-probe results.

Primary gates:

- `cargo check -p taru-streaming -p taru-transcode -p taru-server --tests`
- focused playback/profile identity nextest
- focused hardware diagnostics nextest
- `git diff --check`

## M4 — Search Semantics And Test Locality

Status: planned.

Exit criteria:

- Search semantics have a small evaluation harness before AI/vector search is
  introduced.
- Search Projection versioning or equivalent discipline is documented and
  tested where needed.
- Touched giant test families are split only when doing so improves locality and
  reviewability without weakening coverage.

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
