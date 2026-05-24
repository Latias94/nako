# Cross-Repo Fearless Boundary Alignment - Handoff

Status: Active
Last updated: 2026-05-24

## Current State

The workstream is active. CRFBA-030 has landed in `crates/nako-library`, and
CRFBA-020 now owns the active addon registration workflow-port slice in
`crates/nako-server`. That server lane has now also narrowed
`crates/nako-server/src/app/acquisition_intake.rs` behind a workflow store for
candidate record/list/discovery/acceptance and
`crates/nako-server/src/app/job_runtime.rs` behind a durable job lease store
for claim/heartbeat/succeed/fail/cancel. `crates/nako-server/src/app/metadata.rs`
has now also narrowed direct metadata refresh, maintenance, raw-response, and
attempt queries behind a dedicated metadata workflow store.
`crates/nako-server/src/app/nfo.rs` now routes NFO job creation,
library/item/source lookups, sidecar-apply audit state transitions, outbox
writes, and durable job runtime lease handoff behind a dedicated NFO workflow
store while keeping `NfoService` as the import/export domain repository
boundary. `crates/nako-server/src/app/artwork.rs` now routes artwork candidate
lookup, media lookup, library-item-state validation, and acceptance commit
through a dedicated `ArtworkAcceptanceWorkflowStore` while leaving the rest of
the managed-artwork operations on the existing store. It now also routes
publish/select/unpublish through a dedicated `ArtworkSelectionWorkflowStore`
while keeping gallery and ingest processing on the existing broad store.
CRFBA-050 and CRFBA-060 have now landed in `../nako-official-addons` with
focused module splits and passing package tests.

Initial review found:

- Nako core crate boundaries are broadly aligned with existing ADRs.
- `NakoDatabase` remains the most obvious overly wide server facade.
- Library ingestion still mixes Source observation, Local Inference planning,
  search projection, and persistence commit concerns.
- Metadata refresh has good ports, but Candidate/Acceptance should become the
  common authority before Addon, NFO, provider, artwork, and AI-like outputs
  grow more write paths.
- The official metadata addon has a sound sidecar boundary, but its scrape
  runtime and provider adapters now carry multiple change axes.
- Jellyfin is useful as a boundary reference; Plex is useful only as a
  product-level benchmark because it is proprietary.

## Active Task

- Task ID: CRFBA-040
- Owner: codex
- Files: `crates/nako-server/src/app/artwork.rs`, `docs/workstreams/cross-repo-fearless-boundary-alignment`
- Validation: focused artwork acceptance and selection nextest, `cargo fmt
  --all -- --check`, and path-scoped `git diff --check` pass.
- Status: IN_PROGRESS
- Review: pending after the next workflow-port slice.
- Evidence: `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`.

## Dirty Worktree Notes

Observed at planning time in `F:/SourceCodes/Rust/nako`:

- `docs/workstreams/README.md` is modified.
- `docs/workstreams/official-addon-e2e-alpha2/HANDOFF.md` is modified.
- `docs/workstreams/official-addon-e2e-alpha2/TODO.md` is modified.
- `docs/workstreams/addon-outbound-task-dispatch-credentials/` is untracked.

Observed at planning time in `F:/SourceCodes/Rust/nako-official-addons`:

- Multiple `crates/nako-metadata-scraper` source files are modified.
- Several official addon workstream evidence files are modified.
- `crates/nako-metadata-scraper/tests/` is untracked.
- Several provider follow-on workstream directories are untracked.

These are treated as user or other-session changes. Do not restore, delete,
format, stage, or commit them unless the user explicitly asks.

## Decisions Since Last Update

- The new lane is cross-repo and coordination-first.
- The authoritative coordination docs live in the Nako main repo.
- `CRFBA-010` is accepted by user request on 2026-05-24.
- `CRFBA-011` recorded dirty worktree constraints; no unrelated dirty files
  existed under `crates/nako-library` before the first implementation slice.
- First implementation slice is `CRFBA-030`, limited to the Library ingestion /
  Local Inference commit path.
- The lane will not update `docs/workstreams/README.md` while that file is
  already modified by another change.
- `CRFBA-030` now has a dedicated `ingestion/source_commit.rs` seam and the
  focused local tests pass.
- `CRFBA-030` is now treated as done with concerns for this lane.
- `CRFBA-020` now has a dedicated addon registration store port in
  `crates/nako-server/src/app/addons.rs`.
- `CRFBA-020` now also has a dedicated acquisition intake workflow store port
  in `crates/nako-server/src/app/acquisition_intake.rs`; the focused intake
  nextest gate and `cargo check -p nako-server --bin nako-server` both passed.
- `CRFBA-020` now also narrows `crates/nako-server/src/app/job_runtime.rs`
  behind a dedicated durable job lease store; the focused runtime nextest gate
  passed with four job-runtime tests and `cargo fmt --all -- --check` stayed
  green.
- `CRFBA-020` now also narrows `crates/nako-server/src/app/metadata.rs`
  behind a dedicated metadata workflow store for direct metadata reads/writes;
  the focused metadata nextest gate passed with 27 metadata-related tests and
  `cargo fmt --all -- --check` stayed green.
- `CRFBA-020` now also narrows `crates/nako-server/src/app/nfo.rs` behind a
  dedicated NFO workflow store for app-level job, library/source/item,
  sidecar-apply audit, outbox, and durable job lease handoff operations. The
  actual `NfoService` repository dependency stays intact for import/export
  domain behavior.
- `CRFBA-040` now narrows `crates/nako-server/src/app/artwork.rs` behind a
  dedicated artwork acceptance workflow store for candidate lookup,
  media/item-state validation, and acceptance commit while leaving the rest of
  the managed-artwork operations on the existing broad store.
- `CRFBA-040` now also narrows `crates/nako-server/src/app/artwork.rs` behind
  a dedicated artwork selection workflow store for publish, select, and
  unpublish operations while keeping gallery and ingest processing on the
  existing broad store.
- `CRFBA-050` split `MetadataScrapeRuntime` into `query`, `orchestration`,
  `response`, `runtime`, `writeback`, and `bulk` modules with unchanged public
  payloads.
- `CRFBA-060` split TMDB into provider-local `client`, `search`, `parser`,
  `mapper`, `enrichment`, and `test_support` modules with focused provider
  tests.

## Blockers

- None on the official addon runtime/provider slices; CRFBA-050 and CRFBA-060
  are complete and verified.
- Broader `cargo nextest run -p nako-library local_inference --no-fail-fast`
  still trips over an unrelated database migration conflict from the addon
  credentials work (`duplicate column name: outbound_task_dispatch_secret_env`).
- The `nako-server` package test binary still hits unrelated unsafe test code
  in `src/http/tests/addons.rs`, so `cargo check` is the current clean
  verification gate for the addon registration port slice.
- `DurableJobRuntime` is now behind a smaller job lease store and can be used
  as the next server-side seam for metadata maintenance or other job-based
  workflows.
- `MetadataAppService` now uses a dedicated workflow store for direct metadata
  queries and writes, which makes the next seam candidate easier to isolate
  without changing executor behavior.
- `NfoAppService` now uses a dedicated workflow store for direct app-level
  persistence and only keeps a concrete repository handle for `NfoService`
  domain execution.

## Next Recommended Action

Continue CRFBA-040 by reviewing whether the remaining managed-artwork
operations should share the same acceptance authority slice or be split into
follow-ons. Keep CRFBA-030 only as a follow-up review note unless the migration
blocker is independently resolved.
