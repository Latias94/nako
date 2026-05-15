# Playback Streaming TODO

## M7.0 Design Baseline

- [x] Add ADR 0017 for playback streaming and remote hardening boundaries.
- [x] Create playback-streaming workstream.
- [x] Split M7 milestones.
- [x] Move M6 deferred playback hardening tasks into M7 ownership.
- [x] Update roadmap, goal map, ADR index, and workstream index.

## Remote Direct Body Streaming

- [x] Replace remote direct-play `Vec<u8>` response bodies with an async body
      stream abstraction.
- [x] Preserve local file streaming behavior.
- [x] Preserve HTTP Range, HEAD, content length, and content range behavior.
- [x] Use backend request timeout and HTTP body drop/cancellation behavior for
      remote body streams.
- [x] Acquire `playback.remote.stream` budget before opening remote bodies.
- [x] Add tests proving selected remote ranges are streamed rather than fully
      buffered.
- [x] Split direct-play app planning and HTTP response helpers out of the
      largest server files.

## Staging Manifest and Cleanup

- [x] Define staging manifest record model.
- [x] Add SQLite migration and repository.
- [x] Track source URI, staging purpose, local path, size, etag/fingerprint,
      state, last access, and expiration.
- [x] Record remote probe input staging in the manifest.
- [x] Record remux/HLS remote FFmpeg input staging in the manifest.
- [x] Add disk budget configuration.
- [x] Enforce disk budget before staging remote inputs.
- [x] Add budget exhaustion test.
- [x] Add startup cleanup.
- [ ] Add bounded background cleanup worker if needed.
- [x] Ensure cleanup does not delete active staged inputs.
- [x] Add stale cleanup test.
- [ ] Add tests for reuse and validation mismatch.

## Playback Error Mapping

- [x] Add initial stable HTTP codes for staging budget, staging validation,
      storage timeout/auth/rate-limit, and FFmpeg provider failures.
- [ ] Define typed playback/storage error categories.
- [ ] Map remote not found, unauthorized, timeout, transient backend failure,
      stale cache fallback, unsupported range, staging budget exhaustion,
      staging validation mismatch, and FFmpeg failure.
- [ ] Update HTTP API docs.
- [ ] Add app and route tests for representative failures.
- [ ] Verify errors never expose credentials or raw backend internals.

## Remote Playback Resource Budgets

- [x] Add `playback.remote.stream` config and defaults.
- [x] Add `playback.remote.stage` config and defaults.
- [ ] Decide whether cleanup needs a separate resource class.
- [x] Add budget acquisition to direct streaming and staging paths.
- [x] Add concurrency-limit tests.

## Multi-Library and Multi-Remote Config

- [ ] Design explicit library configuration model.
- [ ] Support multiple named library backends at startup.
- [ ] Keep current single-library config working during migration.
- [ ] Support mixed local and WebDAV libraries.
- [ ] Ensure source URI roots resolve to the correct backend.
- [ ] Keep credentials as secret references only.
- [ ] Add config parsing and app-level tests.

## NFO Storage Boundary

- [x] Route NFO import through the configured library VFS backend.
- [x] Route NFO export through the configured library VFS backend.
- [x] Gate NFO export on `StorageCapabilities::WRITABLE`.
- [x] Add WebDAV import and read-only export tests.

## Stabilization

- [ ] Update local setup docs.
- [ ] Update HTTP API docs.
- [ ] Update test strategy docs.
- [ ] Document M7 known limitations.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo check --workspace`.
- [ ] Run `cargo nextest run --workspace`.
- [ ] Run `git diff --check`.
