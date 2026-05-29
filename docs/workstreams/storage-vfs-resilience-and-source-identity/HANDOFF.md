# Storage/VFS Resilience And Source Identity — Handoff

Status: Active
Last updated: 2026-05-29

## Current State

SVRS-020 is complete. The scan path now derives **Source Fingerprint** values
through a layered evidence policy instead of trusting one optional backend
string directly. The policy lives in `nako-core`, and `VfsLibraryScanner` uses
it when converting VFS metadata into discovered media sources.

The implementation proves:

- size + ETag can create a redaction-safe fingerprint without full-file reads;
- locator-only evidence does not create a fingerprint;
- content hashes are the only evidence strong enough to preserve source identity
  automatically;
- equal fingerprints across different locators do not merge **Media Sources**.

The branch/worktree prepared for this lane is:

- `fearless/non-web-architecture-deepening`
- `F:\SourceCodes\Rust\nako-worktrees\nako-non-web-architecture-deepening`

The main worktree had unrelated `web/` edits when this lane was opened. Treat
them as user/other-agent work and do not revert or format them from this lane.

## Active Task

- Task ID: SVRS-030
- Owner: codex
- Files:
  - `crates/nako-library`
  - `crates/nako-db`
  - `crates/nako-server` if scan job integration needs it
- Validation:
  - `cargo nextest run -p nako-library rename_reconciliation --no-fail-fast`
  - `cargo nextest run -p nako-db scan source_duplicate --no-fail-fast`
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

## Blockers

- None currently known.

## Next Action

Execute SVRS-030: use the SVRS-020 evidence policy to reconcile moves and
renames. Strong evidence may preserve source identity; weak evidence should
create reviewable duplicate/reconciliation state without automatic merge.
