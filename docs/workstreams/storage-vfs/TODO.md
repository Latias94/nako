# Storage and VFS TODO

## M6.0 Design Baseline

- [x] Add ADR 0016 for remote storage and VFS cache boundaries.
- [x] Create storage-vfs workstream.
- [x] Audit current VFS, scan/probe, and playback local-path dependencies.
- [x] Split M6 milestones and choose the first backend.
- [x] Update roadmap, goal map, ADR index, and workstream index.

## WebDAV Read-Only Backend

- [x] Define WebDAV backend configuration and secret references.
- [x] Add WebDAV URI scheme and locator rules.
- [x] Implement `stat` for files and directories.
- [x] Implement `list` with pagination/depth policy.
- [x] Implement `open_range` without local path hints.
- [x] Add mocked WebDAV server tests.
- [x] Add timeout, retry, and rate-limit tests.
- [x] Verify source locators never contain plaintext credentials.

## Directory and Stat Cache

- [x] Define cache record model.
- [x] Add SQLite migration and repository.
- [x] Add TTL and refresh policy.
- [x] Add transient failure state.
- [x] Ensure cache failures do not directly tombstone catalog sources.

## Remote Probe Staging

- [x] Define staging service API.
- [x] Add deterministic staging paths.
- [x] Validate staged file reuse by size/etag/fingerprint.
- [ ] Add disk budget and cleanup policy.
- [x] Route remote probe inputs through staging.

## Remote Playback

- [x] Stream remote range-readable sources without local path hints.
- [x] Define remux staging policy for remote inputs.
- [x] Define HLS staging policy for remote inputs.
- [ ] Extend playback decisions with remote storage constraints beyond the
      initial VFS range/local-path policy.
- [ ] Add failure mapping for remote storage timeouts and stale cache.

## Stabilization

- [ ] Add first-class WebDAV preview configuration to server setup.
- [ ] Update local setup docs for WebDAV preview.
- [ ] Update HTTP API docs and known limitations.
- [ ] Run workspace validation gates.
- [ ] Document remaining M6 gaps and next goal.
