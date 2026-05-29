# Storage/VFS Resilience And Source Identity — Handoff

Status: Active
Last updated: 2026-05-29

## Current State

SVRS-020 and SVRS-030 are complete. The scan path now derives
**Source Fingerprint** values through a layered evidence policy instead of
trusting one optional backend string directly. The policy lives in `nako-core`,
and `VfsLibraryScanner` carries evidence kind and confidence into discovered
media sources.

The implementation proves:

- size + ETag can create a redaction-safe fingerprint without full-file reads;
- locator-only evidence does not create a fingerprint;
- content hashes are the only evidence strong enough to preserve source identity
  automatically;
- equal fingerprints across different locators do not merge **Media Sources**.
- strong content-hash moves/renames preserve the existing **Media Source**,
  curated item metadata, and non-provisional item state;
- weak duplicate evidence and simultaneous strong duplicate files become
  suggested **Source Duplicate Relationship** records instead of automatic
  merges;
- scan source commits persist duplicate relationships in the same SQLite or
  PostgreSQL transaction as the source, item state, inference evidence, search
  projection, and ingestion-failure resolution.

The branch/worktree prepared for this lane is:

- `fearless/non-web-architecture-deepening`
- `F:\SourceCodes\Rust\nako-worktrees\nako-non-web-architecture-deepening`

The main worktree had unrelated `web/` edits when this lane was opened. Treat
them as user/other-agent work and do not revert or format them from this lane.

## Active Task

- Task ID: SVRS-040
- Owner: codex
- Files:
  - `crates/nako-vfs`
  - `crates/nako-library`
  - `crates/nako-server`
- Validation:
  - `cargo nextest run -p nako-vfs --no-fail-fast`
  - `cargo nextest run -p nako-server storage --no-fail-fast`
- Status: READY

## Decisions So Far

- Keep this lane non-Web and non-HLS.
- Start with source identity evidence policy, not a new backend.
- Do not require full-file hashes during normal scan.
- Preserve **Media Source** identity only when evidence is strong.
- Use **Source Duplicate Relationship** for duplicate evidence; do not merge
  sources automatically.
- Keep diagnostics redaction-safe and avoid exposing **Source Locators**, local
  paths, raw ETags, credentials, or fingerprint values.
- SVRS-020 did not add persistence schema or repository contract changes.
- `MediaSource.fingerprint` and `SourceState.fingerprint` now receive hashed
  policy fingerprints such as `source:v1:size_etag:sha256:...`, not raw ETags
  or raw backend fingerprint strings.
- Strong relocation is eligible only when there is exactly one same-fingerprint
  prior source and that old locator is absent from the current scan.
- `LibraryScanSourcePersistenceCommit` now carries
  `source_duplicate_relationships`; adapters must keep them atomic with the
  rest of scan source persistence.

## Blockers

- None currently known.

## Next Action

Execute SVRS-040: classify timeout, unavailable, permission, rate-limit,
stale-cache, and partial-read failures consistently across VFS-backed
scan/probe/stage paths. Keep diagnostics redaction-safe and avoid hiding
long-running storage work in ad hoc tasks.
