# Core Architecture Deepening Evidence And Gates

Status: Completed
Last updated: 2026-05-18

## Evidence Anchors

- `docs/workstreams/core-architecture-deepening/`
- `crates/taru-core/src/repository/`
- `crates/taru-db/src/metadata.rs`
- `crates/taru-db/src/scan.rs`
- `crates/taru-nfo/src/import.rs`
- `crates/taru-library/src/index.rs`
- `crates/taru-search/`
- `crates/taru-server/src/app/nfo.rs`
- `crates/taru-server/src/app/jobs.rs`
- `crates/taru-server/src/app/playback/`
- `crates/taru-streaming/`
- `crates/taru-transcode/`
- `crates/taru-api/src/admin.rs`

## Task Gates

CAD-010:

- `git diff --check`

CAD-020:

- `cargo check -p taru-core --tests`
- `cargo check -p taru-db --tests`
- `cargo check -p taru-nfo --tests`
- `cargo nextest run -p taru-nfo --no-fail-fast`
- `cargo nextest run -p taru-db nfo --no-fail-fast`
- `git diff --check`

CAD-030:

- `cargo check -p taru-library --tests`
- `cargo check -p taru-db --tests`
- `cargo nextest run -p taru-library --no-fail-fast`
- `cargo nextest run -p taru-db scan --no-fail-fast`
- `git diff --check`

CAD-040:

- `cargo check -p taru-server --tests`
- `cargo nextest run -p taru-server app --no-fail-fast`
- Focused crate gates from CAD-020 and CAD-030 for touched crates.
- `git diff --check`

CAD-050:

- `cargo check -p taru-streaming --tests`
- `cargo check -p taru-transcode --tests`
- `cargo check -p taru-server --tests`
- `cargo nextest run -p taru-server playback --no-fail-fast`
- `git diff --check`

CAD-060:

- `cargo check -p taru-transcode --tests`
- `cargo check -p taru-api --tests`
- `cargo check -p taru-server --tests`
- `cargo nextest run -p taru-transcode --no-fail-fast`
- Focused admin playback diagnostics tests.
- `git diff --check`

CAD-070:

- Docs consistency review against addon workstreams.
- Focused crate checks for touched addon/server modules.
- `git diff --check`

CAD-080:

- `cargo fmt --all -- --check`
- Focused `cargo check` and `cargo nextest run` for every touched crate.
- `git diff --check`

CAD-090:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- `git diff --check`

## Evidence Log

- 2026-05-18: Workstream opened. CAD-010 docs created. Implementation evidence
  begins with CAD-020.
- 2026-05-18: CAD-020 implemented NFO import atomic commit.
  - `cargo check -p taru-core --tests`: passed.
  - `cargo check -p taru-db --tests`: passed.
  - `cargo check -p taru-catalog --tests`: passed; added because CAD-020 now
    uses catalog projection planning.
  - `cargo check -p taru-nfo --tests`: passed.
  - `cargo nextest run -p taru-nfo --no-fail-fast`: passed, 20 tests.
  - `cargo nextest run -p taru-db nfo --no-fail-fast`: passed, 2 tests.
  - `cargo fmt --all -- --check`: passed.
  - `git diff --check`: passed; Git reported existing CRLF-normalization
    warnings only.
  - Behavior proven: `commit_nfo_import` rolls back item, field lock, library
    item state, catalog graph, and search projection on projection failure; NFO
    import production flow no longer performs caller-side ordered durable
    writes; existing non-NFO field locks remain authoritative across repeated
    NFO imports.
- 2026-05-18: CAD-030 implemented Library Scan Source Commit.
  - `cargo check -p taru-core --tests`: passed.
  - `cargo check -p taru-db --tests`: passed.
  - `cargo check -p taru-library --tests`: passed.
  - `cargo check -p taru-catalog --tests`: passed as a touched dependency
    check from CAD-020.
  - `cargo check -p taru-nfo --tests`: passed as a touched dependency check
    after the core commit DTO additions.
  - `cargo nextest run -p taru-library --no-fail-fast`: passed, 15 tests.
  - `cargo nextest run -p taru-db scan --no-fail-fast`: passed, 4 tests.
  - `cargo fmt --all -- --check`: passed.
  - `git diff --check`: passed; Git reported existing CRLF-normalization
    warnings only.
  - Behavior proven: `commit_library_scan_source` atomically writes Media
    Item, Media Source, Source State, Library Item State, Local Inference
    Evidence, Search Projection, and scan failure resolution; rollback test
    proves item/source/state/evidence/state changes do not survive a search
    projection failure and the prior scan failure remains open.
- 2026-05-18: CAD-040 narrowed NFO import and Library indexing service seams to
  named workflow ports.
  - `cargo check -p taru-server --tests`: passed.
  - `cargo nextest run -p taru-server app --no-fail-fast`: passed, 76 tests.
  - `cargo nextest run -p taru-nfo --no-fail-fast`: passed, 20 tests.
  - `cargo nextest run -p taru-library --no-fail-fast`: passed, 15 tests.
  - `cargo nextest run -p taru-db nfo --no-fail-fast`: passed, 2 tests.
  - `cargo nextest run -p taru-db scan --no-fail-fast`: passed, 4 tests.
  - `cargo fmt --all -- --check`: passed.
  - `git diff --check`: passed; Git reported existing CRLF-normalization
    warnings only.
  - Behavior proven: `NfoImportRepository` and `LibraryIndexRepository` expose
    focused service ports around the new durable commit units without changing
    server app behavior, NFO import/export behavior, library indexing behavior,
    or SQLite rollback guarantees from CAD-020/CAD-030.
- 2026-05-18: CAD-050 replaced weak playback/transcode request identity with
  profile-shaped identity.
  - `cargo check -p taru-streaming --tests`: passed.
  - `cargo check -p taru-transcode --tests`: passed.
  - `cargo check -p taru-api --tests`: passed; added because DTO sample tests
    were updated away from obsolete request-key constants.
  - `cargo check -p taru-server --tests`: passed.
  - `cargo nextest run -p taru-streaming --no-fail-fast`: passed, 9 tests.
  - `cargo nextest run -p taru-transcode --no-fail-fast`: passed, 22 tests.
  - `cargo nextest run -p taru-api --no-fail-fast`: passed, 21 tests.
  - `cargo nextest run -p taru-server playback --no-fail-fast`: passed, 36
    tests.
  - `cargo fmt --all -- --check`: passed.
  - `git diff --check`: passed; Git reported existing CRLF-normalization
    warnings only.
  - Behavior proven: playback profile identity normalizes client capability
    order and case; HLS transcode profile identity changes when selected
    hardware acceleration changes; HLS CPU fallback output is not reused by a
    later NVENC profile; existing Remux/HLS finished-output reuse, duplicate
    detection, cancellation, and HTTP playback routes continue to pass.
- 2026-05-18: CAD-060 deepened hardware acceleration diagnostics with safe
  capability evidence and smoke-probe/operator-check hooks.
  - `cargo check -p taru-transcode --tests`: passed.
  - `cargo check -p taru-api --tests`: passed.
  - `cargo check -p taru-server --tests`: passed.
  - `cargo nextest run -p taru-transcode --no-fail-fast`: passed, 23 tests.
  - `cargo nextest run -p taru-api admin_playback_runtime_diagnostics --no-fail-fast`:
    passed, 1 test.
  - `cargo nextest run -p taru-api --no-fail-fast`: passed, 21 tests.
  - `cargo nextest run -p taru-server admin_v1_playback_runtime_reports_safe_diagnostics --no-fail-fast`:
    passed, 1 test.
  - `cargo fmt --all -- --check`: passed.
  - `git diff --check`: passed; Git reported existing CRLF-normalization
    warnings only.
  - Behavior proven: admin playback runtime diagnostics now distinguish
    configured policy, selected fallback, FFmpeg encoder evidence, static
    detector evidence, and smoke-probe status; hardware smoke detail remains
    internal while admin diagnostics expose only safe operator guidance and a
    boolean detail flag.
- 2026-05-18: CAD-070 aligned Addon Sidecar protected-write follow-ons with the
  new first-party commit boundaries.
  - Audit command: `rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator|commit_nfo_import|commit_library_scan_source|NfoImportPersistenceCommit|LibraryScanSourcePersistenceCommit" crates docs`.
  - `git diff --check`: passed; Git reported existing CRLF-normalization
    warnings only.
  - Behavior proven: no current Addon runtime path writes NFO files, subtitle
    files, Managed Artwork, Library File Write state, Media Source scan state,
    or NFO import field locks. `addon-library-file-write-policy` now requires
    future NFO-derived metadata apply to reuse `commit_nfo_import` and future
    discoverable source/state/search changes to reuse
    `commit_library_scan_source`, `LibraryIndexRepository`, or a new
    first-party commit unit. `addon-managed-artwork-artifacts` now requires
    artwork multi-row persistence to use or introduce a Taru-owned
    artwork/catalog commit boundary and routes sidecar-file export back to the
    Library File Write lane.
- 2026-05-18: CAD-080 deletion sweep removed stale test anchors and hardened
  the CAD-060 hook evidence.
  - Obsolete request-key scan:
    `rg -n "hls:single|remux:" crates docs --glob '!docs/workstreams/core-architecture-deepening/TODO.md' --glob '!docs/workstreams/core-architecture-deepening/HANDOFF.md'`.
  - Production write-path scan:
    `rg -n "record_scanned_media_source|upsert_source_state\\(|upsert_local_inference_evidence\\(|commit_library_scan_source\\(" crates/taru-library crates/taru-server crates/taru-db/src/scan.rs crates/taru-core/src/repository/scan.rs`.
  - NFO write-path scan:
    `rg -n "upsert_field_lock\\(|confirm_hierarchy\\(|hydrate_item_catalog\\(|commit_nfo_import\\(" crates/taru-nfo/src/import.rs crates/taru-nfo/src/lib.rs crates/taru-server/src/app/nfo.rs crates/taru-db/src/metadata.rs crates/taru-core/src/repository/metadata.rs`.
  - `cargo check -p taru-transcode --tests`: passed.
  - `cargo check -p taru-db --tests`: passed.
  - `cargo check -p taru-client --tests`: passed.
  - `cargo nextest run -p taru-transcode --no-fail-fast`: passed, 24 tests.
  - `cargo nextest run -p taru-db transcode --no-fail-fast`: passed, 3 tests.
  - `cargo nextest run -p taru-client playback_decision_query_and_session_cancel_paths_are_stable --no-fail-fast`:
    passed, 1 test.
  - `cargo fmt --all -- --check`: passed.
  - `git diff --check`: passed; Git reported existing CRLF-normalization
    warnings only.
  - Behavior proven: obsolete production request-key constants are no longer
    present outside historical workstream notes; DB/client tests use opaque
    test request keys rather than old production identity strings; no replaced
    NFO import or discovered-source production write path is reachable through
    `taru-nfo` import or `taru-library` source indexing; fake smoke-probe
    capability is covered by CI.
- 2026-05-18: CAD-090 closeout completed with workspace gates.
  - `cargo check --workspace --tests`: passed.
  - `cargo nextest run --workspace --no-fail-fast`: passed, 375 tests.
  - `cargo fmt --all -- --check`: passed.
  - `git diff --check`: passed; Git reported existing CRLF-normalization
    warnings only.
  - Behavior proven: the full workspace compiles under test configuration and
    all Rust tests pass after the NFO import, Library scan source,
    workflow-port, playback/transcode identity, hardware diagnostics, addon
    alignment, and deletion-sweep changes.
