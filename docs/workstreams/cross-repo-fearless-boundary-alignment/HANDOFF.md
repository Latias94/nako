# Cross-Repo Fearless Boundary Alignment - Handoff

Status: Completed
Last updated: 2026-05-29

## Current State

The workstream is closed as of 2026-05-29. CRFBA-030 has landed in `crates/nako-library`, and
CRFBA-020 now owns the active server workflow-port slice in
`crates/nako-server`. That server lane has now also narrowed
`crates/nako-server/src/app/acquisition_intake.rs` behind a workflow store for
candidate record/list/discovery/acceptance and
`crates/nako-server/src/app/job_runtime.rs` behind a durable job lease store
for claim/heartbeat/succeed/fail/cancel. `crates/nako-server/src/app/metadata.rs`
now narrows direct metadata refresh, maintenance, raw-response, and attempt
queries behind a dedicated metadata workflow store plus a smaller execution
store for catalog hydration, refresh snapshot/commit, and attempt recording.
`crates/nako-server/src/app/jobs.rs` now narrows library scan enqueue, library
lookup, outbox writes, scan ingestion, probe execution, and failure
bookkeeping behind a dedicated library-scan workflow store plus an execution
store. `crates/nako-server/src/app/playback/mod.rs` now narrows transcode
session access, cancellation, and playback execution through a dedicated
runtime store. `crates/nako-server/src/app/playback/input.rs` now keeps
staging manifest lookup and lease acquisition on `Arc<dyn
StagingManifestRepository>`, and `PlaybackAppService::new` now receives the
runtime/staging ports from composition instead of the raw database facade.
`crates/nako-server/src/app/nfo.rs` now routes NFO job
creation, library/item/source lookups, sidecar-apply audit state transitions,
outbox writes, and durable job runtime lease handoff behind a dedicated NFO
workflow store while keeping `NfoService` as the import/export domain
repository boundary. CRFBA-050 and CRFBA-060 have now landed in
`../nako-official-addons` with focused module splits and passing package tests.
CRFBA-070 now aligns protected-write host-client responsibilities across
`crates/nako-addon-protocol`, `crates/nako-addon-client`, and the official
metadata scraper `nako_runtime` facade: protocol owns wire payload shapes,
client owns runtime HTTP behavior, and the official addon delegates to the
public crates instead of keeping a private duplicate implementation.
CRFBA-090 now hardens the touched contract surface: Addon Runtime route paths
have a public protocol inventory, the Nako Runtime client and server route
registration use those constants, Public Client OpenAPI tests exclude the
runtime inventory, and `nako-api` tests prove access-check and side-effect
wire-shape parity against `nako-addon-protocol`.
CRFBA-080 then moved product-level transcode session ownership into Playback
Runtime while keeping `nako-transcode` as an immutable execution API. CRFBA-100
closed the lane after fresh focused verification in Nako and
`../nako-official-addons`.

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

- Task ID: none; CRFBA-100 closed the lane.
- Owner: codex
- Files: `docs/workstreams/cross-repo-fearless-boundary-alignment/*` plus the
  official addon fixture compatibility change in
  `../nako-official-addons/crates/nako-metadata-scraper/src/providers/fixture.rs`.
- Validation: focused Nako playback/transcode gates, `cargo check -p
  nako-server`, `cargo fmt --all -- --check`, official addon package tests,
  official addon focused metadata/writeback/artwork/ranking filter, and
  path-scoped `git diff --check`.
- Status: DONE_WITH_CONCERNS
- Review: no blocking CRFBA-080 implementation finding. Stale CRFBA-020/040
  status was corrected during closeout.
- Evidence: `TODO.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and
  `JOURNAL/2026-05-29-CRFBA-100.md`.

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
- `CRFBA-020` now also narrows `crates/nako-server/src/app/jobs.rs` behind a
  dedicated library-scan workflow store and execution store so library scan
  enqueue, library lookup, outbox, scan ingestion, probe execution, and
  failure bookkeeping no longer reach straight through the broad database
  facade.
- `CRFBA-020` now also narrows `crates/nako-server/src/app/playback/mod.rs`
  behind a dedicated playback runtime store so playback decisions,
  remux/HLS execution, transcode-session access, cancellation, and
  finished-event writes no longer call the broad database facade directly.
- `CRFBA-020` now also narrows `crates/nako-server/src/app/playback/input.rs`
  behind `Arc<dyn StagingManifestRepository>` for staging manifest lookup and
  lease acquisition, while `PlaybackAppService::new` now receives runtime and
  staging ports from composition instead of the raw database facade.
- `crates/nako-library/src/probe.rs` now depends on `LibraryProbeWorkflow`,
  which keeps probe execution on a narrower source/probe/failure port instead
  of the full repository set.
- `crates/nako-metadata/src/lib.rs` now re-exports focused metadata strategy
  ports so `MetadataExecutionStore` can own refresh snapshot/commit and
  attempt-record operations without widening the public metadata surface.
- `CRFBA-040` now narrows `crates/nako-server/src/app/artwork.rs` behind a
  dedicated artwork acceptance workflow store for candidate lookup,
  media/item-state validation, and acceptance commit while leaving the rest of
  the managed-artwork operations on the existing broad store.
- `CRFBA-040` now also narrows `crates/nako-server/src/app/artwork.rs` behind
  a dedicated artwork selection workflow store for publish, select, and
  unpublish operations while keeping gallery and ingest processing on the
  existing broad store.
- `CRFBA-040` now also narrows `crates/nako-server/src/app/artwork.rs` behind
  a dedicated artwork ingest workflow store for claim, requeue, commit, and
  fail operations while leaving the remaining lifecycle and cleanup reads on
  the narrower lifecycle store.
- `CRFBA-050` split `MetadataScrapeRuntime` into `query`, `orchestration`,
  `response`, `runtime`, `writeback`, and `bulk` modules with unchanged public
  payloads.
- `CRFBA-060` split TMDB into provider-local `client`, `search`, `parser`,
  `mapper`, `enrichment`, and `test_support` modules with focused provider
  tests.
- `CRFBA-070` moved protected-write access-check and side-effect client behavior
  into the public `nako-addon-client` crate while keeping wire payload shapes in
  `nako-addon-protocol`.
- `CRFBA-070` kept the permissive protocol crate free of reqwest-heavy runtime
  behavior; the runtime HTTP client lives in `nako-addon-client`.
- `CRFBA-070` changed the official metadata scraper `nako_runtime` module into
  a thin facade over the public client/protocol crates. The official addon uses
  local path dependencies for the current cross-repo proof; release/versioning
  is a separate follow-on if distribution requires it.
- `CRFBA-070` review found and fixed a safe-error-mapping gap in the reusable
  reqwest transport: public client transport errors now strip request URLs and
  cap error text before surfacing `AddonClientError::Http`.
- `CRFBA-090` chose route-inventory plus wire-shape parity tests instead of a
  generated Addon Runtime SDK for this slice. This matches the current public
  crate distribution model and avoids widening the task into release tooling.
- `CRFBA-080` moved product-level transcode session ownership out of
  `nako-transcode`: the public execution surface is now
  `TranscodeExecutionRequest` plus typed engine outcomes, while
  `Playback Runtime` owns persisted session state, cancellation, failure
  taxonomy, finished events, and HLS metrics persistence.
- `CRFBA-080` also made FFmpeg HLS path assertions platform-stable so the
  `nako-transcode` package gate passes on the Windows development host.
- `CRFBA-100` closed the lane. The remaining work is split into follow-ons:
  adaptive bitrate ladders, optimized versions, additional hardware backends,
  broader Candidate/Acceptance paths beyond managed artwork, public addon
  SDK/release artifacts, and provider splits beyond the current TMDB proof.
- `CRFBA-100` also updated the official addon fixture provider to default new
  optional `AddonMetadataPatch` fields so it stays compatible with the local
  public protocol crate.

## Residual Risks And Follow-ons

- Broader historical package/workspace gates still have unrelated blockers in
  older slices; CRFBA closeout used focused gates and `cargo check -p
  nako-server` instead of treating unrelated failures as lane blockers.
- The current official addon checkout only has four `nako-metadata-scraper`
  tests. Historical `tmdb` and `nako_runtime` filters now produce "no tests to
  run", so closeout uses the full current package gate plus the
  metadata/writeback/artwork/ranking filter.
- Adaptive bitrate ladders, optimized versions, additional hardware backends,
  broader Candidate/Acceptance paths beyond managed artwork, public addon
  SDK/release artifacts, and provider splits beyond TMDB remain separate
  follow-on lanes.

## Next Recommended Action

Commit the CRFBA-100 closeout docs in Nako and the official addon fixture
compatibility fix as separate repository commits if the user wants the lane
recorded in git. Future work should open a new workstream rather than adding
more scope to this closed lane.
