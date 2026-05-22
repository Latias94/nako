# Core Architecture Deepening Handoff

Status: Completed
Last updated: 2026-05-18

## Current State

The workstream is closed. CAD-010 through CAD-090 are complete with fresh
focused and workspace evidence.

Completed:

- CAD-010 opened the workstream documentation and linked it from the workstream
  index.
- CAD-020 replaced the NFO import production write ordering with
  `NfoImportPersistenceCommit` and `MetadataRepository::commit_nfo_import`.
  The SQLite adapter now commits Media Item, Metadata Field Lock, Library Item
  State, Catalog Item Graph, and Search Projection changes in one transaction.
  `nako-nfo` now plans an import commit, including hierarchy confirmation
  state, and removed the production calls to `upsert_media_item`,
  `upsert_field_lock`, `HierarchyConfirmationService::confirm_hierarchy`, and
  `hydrate_item_catalog`.
- CAD-030 replaced discovered source caller-side write ordering with
  `LibraryScanSourcePersistenceCommit` and
  `ScanRepository::commit_library_scan_source`. The SQLite adapter now commits
  Media Item, Media Source, Source State, Library Item State, Local Inference
  Evidence, Search Projection, and scan failure resolution in one transaction.
  `nako-library` now plans provisional series/season parents without early
  writes and no longer calls separate source/evidence/search/failure-resolution
  writes for discovered media sources.
- CAD-040 narrowed the NFO import and Library indexing app-service seams with
  named workflow ports. `NfoImportRepository` groups only the repository
  contracts needed by NFO import planning and `commit_nfo_import`.
  `LibraryIndexRepository` groups only the repository contracts needed by
  Library indexing and `commit_library_scan_source`. The change did not add
  pass-through methods or retain replaced caller-side write ordering.
- CAD-050 introduced Playback Profile and Transcode Profile identity. HLS and
  Remux persisted request keys now come from explicit profile facts rather than
  `hls:single` or `remux:<container>` constants, and output staging paths now
  include a profile storage slug so different profiles do not share one output
  directory. HLS identity includes the selected hardware acceleration, and tests
  prove a CPU fallback HLS output is not reused by a later NVENC profile.
- CAD-060 deepened hardware acceleration diagnostics. `nako-transcode` now
  records safe `HardwareCapabilityEvidence` values for CPU, FFmpeg
  encoder-listed, encoder-missing, probe-error, and static-detector reports.
  Hardware capabilities also include `HardwareSmokeProbe` state with
  operator-run smoke-check guidance for VAAPI, NVENC, and Quick Sync.
  `FfmpegHardwareAccelerationDetector` accepts fakeable smoke probes for tests,
  while production diagnostics report operator-run checks. The admin playback
  runtime API maps the new evidence and smoke status to explicit DTOs and only
  exposes `has_detail` for smoke failures rather than raw detail.
- CAD-070 audited Addon Sidecar protected-write follow-ons against the new
  first-party commit boundaries. Existing Addon runtime behavior still only has
  bounded `metadata_write`; artwork, subtitle, NFO, and Library File Write
  runtime behavior remains in dedicated follow-on workstreams. Those
  workstreams now explicitly require NFO-derived metadata apply to reuse
  `commit_nfo_import`, discoverable source/state/search changes to reuse
  `commit_library_scan_source` or a new first-party commit unit, and artwork
  multi-row persistence to use or introduce a Nako-owned artwork/catalog commit
  boundary.
- CAD-080 removed stale test anchors and checked for old/new production path
  duplication. DB transcode-session tests and the client mock session no longer
  use obsolete `hls:single` or `remux:<container>` request-key examples. The
  fakeable hardware smoke-probe hook is now covered by a `nako-transcode` test.
  Scans did not find remaining production use of the replaced NFO import or
  discovered-source caller-side write ordering.

Current task:

- None. The workstream is complete.

## Next Step

- Continue artwork, subtitle, NFO export, and Library File Write product
  breadth in `addon-managed-artwork-artifacts` or
  `addon-library-file-write-policy` when those features become priority.
- Do not reopen this lane for provider breadth, adaptive bitrate, addon token
  lifecycle, or client UI work; those are outside its closed scope.

CAD-020 evidence:

- `cargo check -p nako-core --tests`: passed.
- `cargo check -p nako-db --tests`: passed.
- `cargo check -p nako-catalog --tests`: passed.
- `cargo check -p nako-nfo --tests`: passed.
- `cargo nextest run -p nako-nfo --no-fail-fast`: passed, 20 tests.
- `cargo nextest run -p nako-db nfo --no-fail-fast`: passed, 2 tests.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed with CRLF-normalization warnings only.

CAD-030 evidence:

- `cargo check -p nako-core --tests`: passed.
- `cargo check -p nako-db --tests`: passed.
- `cargo check -p nako-library --tests`: passed.
- `cargo check -p nako-catalog --tests`: passed.
- `cargo check -p nako-nfo --tests`: passed.
- `cargo nextest run -p nako-library --no-fail-fast`: passed, 15 tests.
- `cargo nextest run -p nako-db scan --no-fail-fast`: passed, 4 tests.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed with CRLF-normalization warnings only.

CAD-040 evidence:

- `cargo check -p nako-server --tests`: passed.
- `cargo nextest run -p nako-server app --no-fail-fast`: passed, 76 tests.
- `cargo nextest run -p nako-nfo --no-fail-fast`: passed, 20 tests.
- `cargo nextest run -p nako-library --no-fail-fast`: passed, 15 tests.
- `cargo nextest run -p nako-db nfo --no-fail-fast`: passed, 2 tests.
- `cargo nextest run -p nako-db scan --no-fail-fast`: passed, 4 tests.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed with CRLF-normalization warnings only.

CAD-050 evidence:

- `cargo check -p nako-streaming --tests`: passed.
- `cargo check -p nako-transcode --tests`: passed.
- `cargo check -p nako-api --tests`: passed.
- `cargo check -p nako-server --tests`: passed.
- `cargo nextest run -p nako-streaming --no-fail-fast`: passed, 9 tests.
- `cargo nextest run -p nako-transcode --no-fail-fast`: passed, 22 tests.
- `cargo nextest run -p nako-api --no-fail-fast`: passed, 21 tests.
- `cargo nextest run -p nako-server playback --no-fail-fast`: passed, 36
  tests.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed with CRLF-normalization warnings only.

CAD-060 evidence:

- `cargo check -p nako-transcode --tests`: passed.
- `cargo check -p nako-api --tests`: passed.
- `cargo check -p nako-server --tests`: passed.
- `cargo nextest run -p nako-transcode --no-fail-fast`: passed, 23 tests.
- `cargo nextest run -p nako-api admin_playback_runtime_diagnostics --no-fail-fast`:
  passed, 1 test.
- `cargo nextest run -p nako-api --no-fail-fast`: passed, 21 tests.
- `cargo nextest run -p nako-server admin_v1_playback_runtime_reports_safe_diagnostics --no-fail-fast`:
  passed, 1 test.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed with CRLF-normalization warnings only.

CAD-070 evidence:

- Audit command
  `rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator|commit_nfo_import|commit_library_scan_source|NfoImportPersistenceCommit|LibraryScanSourcePersistenceCommit" crates docs`:
  passed and identified the existing Addon write surface plus relevant
  follow-on docs.
- `git diff --check`: passed with CRLF-normalization warnings only.

CAD-080 evidence:

- `cargo check -p nako-transcode --tests`: passed.
- `cargo check -p nako-db --tests`: passed.
- `cargo check -p nako-client --tests`: passed.
- `cargo nextest run -p nako-transcode --no-fail-fast`: passed, 24 tests.
- `cargo nextest run -p nako-db transcode --no-fail-fast`: passed, 3 tests.
- `cargo nextest run -p nako-client playback_decision_query_and_session_cancel_paths_are_stable --no-fail-fast`:
  passed, 1 test.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed with CRLF-normalization warnings only.

CAD-090 evidence:

- `cargo check --workspace --tests`: passed.
- `cargo nextest run --workspace --no-fail-fast`: passed, 375 tests.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed with CRLF-normalization warnings only.

## Parallelization

CAD-020 and CAD-050 have mostly disjoint write scopes and can be parallelized
after CAD-010 if workers are explicitly assigned. CAD-020 and CAD-030 should not
run in parallel because both will likely touch repository commit interfaces and
SQLite transaction helpers.

## Safety Notes

- Do not copy source, comments, migrations, tests, schemas, or assets from
  `repo-ref/jellyfin`.
- Do not reintroduce old and new production write paths in parallel after this
  lane closes.
- Do not revert unrelated user changes.
- Prefer focused nextest suites first; run workspace gates during closeout.
