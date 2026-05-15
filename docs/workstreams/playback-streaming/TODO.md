# Playback Streaming TODO

## M7.0 Design Baseline

- [x] Add ADR 0017 for playback streaming and remote hardening boundaries.
- [x] Create playback-streaming workstream.
- [x] Split M7 milestones.
- [x] Move M6 deferred playback hardening tasks into M7 ownership.
- [x] Update roadmap, goal map, ADR index, and workstream index.

## Remote Direct Body Streaming

- [ ] Replace remote direct-play `Vec<u8>` response bodies with an async body
      stream abstraction.
- [ ] Preserve local file streaming behavior.
- [ ] Preserve HTTP Range, HEAD, content length, and content range behavior.
- [ ] Add timeout and cancellation handling for remote body streams.
- [ ] Acquire `playback.remote.stream` budget before opening remote bodies.
- [ ] Add tests proving selected remote ranges are streamed rather than fully
      buffered.

## Staging Manifest and Cleanup

- [ ] Define staging manifest record model.
- [ ] Add SQLite migration and repository.
- [ ] Track source URI, staging purpose, local path, size, etag/fingerprint,
      state, last access, and expiration.
- [ ] Add disk budget configuration.
- [ ] Enforce disk budget before staging remote inputs.
- [ ] Add startup cleanup.
- [ ] Add bounded background cleanup worker if needed.
- [ ] Ensure cleanup does not delete active staged inputs.
- [ ] Add tests for stale cleanup, budget exhaustion, reuse, and validation
      mismatch.

## Playback Error Mapping

- [ ] Define typed playback/storage error categories.
- [ ] Map remote not found, unauthorized, timeout, transient backend failure,
      stale cache fallback, unsupported range, staging budget exhaustion,
      staging validation mismatch, and FFmpeg failure.
- [ ] Update HTTP API docs.
- [ ] Add app and route tests for representative failures.
- [ ] Verify errors never expose credentials or raw backend internals.

## Remote Playback Resource Budgets

- [ ] Add `playback.remote.stream` config and defaults.
- [ ] Add `playback.remote.stage` config and defaults.
- [ ] Decide whether cleanup needs a separate resource class.
- [ ] Add budget acquisition to direct streaming and staging paths.
- [ ] Add concurrency-limit tests.

## Multi-Library and Multi-Remote Config

- [ ] Design explicit library configuration model.
- [ ] Support multiple named library backends at startup.
- [ ] Keep current single-library config working during migration.
- [ ] Support mixed local and WebDAV libraries.
- [ ] Ensure source URI roots resolve to the correct backend.
- [ ] Keep credentials as secret references only.
- [ ] Add config parsing and app-level tests.

## Stabilization

- [ ] Update local setup docs.
- [ ] Update HTTP API docs.
- [ ] Update test strategy docs.
- [ ] Document M7 known limitations.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo check --workspace`.
- [ ] Run `cargo nextest run --workspace`.
- [ ] Run `git diff --check`.
