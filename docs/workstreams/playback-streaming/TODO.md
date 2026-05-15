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
- [x] Decide bounded background cleanup worker is post-M7 unless startup
      cleanup proves insufficient.
- [x] Ensure cleanup does not delete active staged inputs.
- [x] Add stale cleanup test.
- [x] Defer reuse and validation mismatch tests to post-M7 hardening.

## Playback Error Mapping

- [x] Add initial stable HTTP codes for staging budget, staging validation,
      storage timeout/auth/rate-limit, and FFmpeg provider failures.
- [x] Defer typed playback/storage error categories to M8 hardening.
- [x] Map remote not found, unauthorized, timeout, transient backend failure,
      stale cache fallback, unsupported range, staging budget exhaustion,
      staging validation mismatch, and FFmpeg failure.
- [x] Update HTTP API docs.
- [x] Add app and route tests for representative failures.
- [x] Verify mapped errors use stable public messages.

## Remote Playback Resource Budgets

- [x] Add `playback.remote.stream` config and defaults.
- [x] Add `playback.remote.stage` config and defaults.
- [x] Decide cleanup does not need a separate resource class until a concurrent
      background worker exists.
- [x] Add budget acquisition to direct streaming and staging paths.
- [x] Add concurrency-limit tests.

## Multi-Library and Multi-Remote Config

- [x] Design explicit library configuration model.
- [x] Support multiple named library backends at startup.
- [x] Replace current single-library config with a single `[[libraries]]`
      entry.
- [x] Support mixed local and WebDAV libraries.
- [x] Ensure persisted sources resolve to the correct configured backend.
- [x] Keep credentials as secret references only.
- [x] Add config parsing and app-level tests.
- [x] Add source-level library identity to avoid paged source lookup in app
      services.
- [x] Use `source.library_id` as the disambiguating identity for multiple
      local `local:///` libraries.

## NFO Storage Boundary

- [x] Route NFO import through the configured library VFS backend.
- [x] Route NFO export through the configured library VFS backend.
- [x] Gate NFO export on `StorageCapabilities::WRITABLE`.
- [x] Add WebDAV import and read-only export tests.

## Stabilization

- [x] Update local setup docs.
- [x] Update HTTP API docs.
- [x] Update test strategy docs.
- [x] Document M7 known limitations.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo check --workspace`.
- [x] Run `cargo check --workspace --tests`.
- [x] Run `cargo nextest run --workspace`.
- [x] Run `git diff --check`.
