# Storage/VFS Resilience And Source Identity — Handoff

Status: Active
Last updated: 2026-05-29

## Current State

SVRS-020, SVRS-030, and SVRS-040 are complete. The scan path now derives
**Source Fingerprint** values through a layered evidence policy instead of
trusting one optional backend string directly. The policy lives in `nako-core`,
and `VfsLibraryScanner` carries evidence kind and confidence into discovered
media sources. Storage failures now have a shared redaction-safe taxonomy and
bounded process-local backoff for VFS-backed read/probe/stage calls.

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
- timeout, unavailable, permission, rate-limit, stale-cache, partial-read,
  budget, security, and unknown storage failures are classified in `nako-core`;
- WebDAV short range bodies are treated as partial reads instead of successful
  reads;
- stale-cache fallback and scan/probe/stage failure records store safe messages
  instead of raw backend paths, URLs, ETags, or credentials;
- library storage health applies bounded process-local backoff only for
  retryable classes and only short-circuits read/probe/stage calls, so
  promotion apply and cleanup compensation still hit the real backend.

The branch/worktree prepared for this lane is:

- `fearless/non-web-architecture-deepening`
- `F:\SourceCodes\Rust\nako-worktrees\nako-non-web-architecture-deepening`

The main worktree had unrelated `web/` edits when this lane was opened. Treat
them as user/other-agent work and do not revert or format them from this lane.

## Active Task

- Task ID: SVRS-050
- Owner: codex
- Files:
  - `crates/nako-api`
  - `crates/nako-server`
  - `docs`
- Validation:
  - `cargo nextest run -p nako-server system storage --no-fail-fast`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast` if Admin DTOs change
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
- `StorageFailureClass::Unknown` is intentionally not retryable; unknown means
  classification is insufficient, not that the backend is transiently down.
- Backoff must remain library-scoped and process-local until a durable
  supervisor/circuit-breaker design is opened.
- Backoff should not suppress storage apply, cleanup, restore, or other
  compensation paths that need to observe real backend behavior.

## Blockers

- None currently known.

## Next Action

Execute SVRS-050: expose redaction-safe Admin diagnostics for source identity
reconciliation, stale VFS cache, storage health, and partial staging cleanup
pressure. Keep diagnostics free of **Source Locators**, local paths, raw ETags,
credentials, and fingerprint values.
