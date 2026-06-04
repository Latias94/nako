# Lane A Watch Folder Stable Intake Runtime

Date: 2026-06-04
Status: implemented

## Scope

- Added a typed, redaction-safe watch-folder intake plan in
  `crates/nako-library/src/intake.rs`.
- Wired `WatchFolderRuntimeAppService::tick_library` to build that plan from
  discovery counts before deciding whether to enqueue the existing durable
  library scan job.
- Extended focused watch-folder runtime tests for first/second/third tick
  behavior and suppression-safe diagnostics.

## Invariants

- Scan and probe execution still flow only through
  `LibraryScanAppService::enqueue_library_scan`.
- The runtime enqueues a scan only when the intake plan sees
  `newly_ready_candidates > 0`.
- A repeated already-ready candidate does not enqueue another scan.
- Suppressed candidates contribute count-only diagnostics and do not advance
  intake or enqueue scans.
- The new intake plan contains only counts and enum decisions; it does not
  carry raw paths, Source Locators, fingerprints, etags, backend URLs, or
  suppression tokens.

## Validation

- `cargo fmt -p nako-library -p nako-server`: passed.
- `cargo nextest run -p nako-library intake --no-fail-fast`: passed, 5 tests.
- `cargo nextest run -p nako-server watch_folder --no-fail-fast`: passed, 10
  tests.
- `cargo nextest run -p nako-server library --no-fail-fast`: passed, 62 tests.
- `cargo check -p nako-library -p nako-server --tests`: passed with existing
  dead-code warnings in `nako-server` playback runtime-session helpers.

## Follow-Ups

- A later Admin diagnostics slice can expose the new count-only plan summary if
  a generated contract owner is available.
- The existing dead-code warnings are outside Lane A and should be handled by
  the playback/artwork lanes or cleanup tasks.
