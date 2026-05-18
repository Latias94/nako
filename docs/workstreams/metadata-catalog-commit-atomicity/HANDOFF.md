# Metadata Catalog Commit Atomicity Handoff

Status: Completed
Last updated: 2026-05-18

## Current State

The workstream is open. MCC-020 and MCC-030 are complete.

Catalog hydration now commits Catalog Item Graph replacement and Search
Projection records through `CatalogRepository::commit_item_projection`. The
SQLite adapter persists both in one transaction. A failure-path test proves the
graph write rolls back when search projection cannot be committed.

Metadata refresh persistence now commits Canonical Metadata, Provider Raw
Response, Provider Subject, accepted Provider Mapping, and Library Item State
confirmation through `MetadataRepository::commit_metadata_refresh`. The SQLite
adapter owns the transaction and reads current Library Item State rows inside
that transaction before confirming them. The old shallow
`apply_metadata_refresh` interface was removed.

## Closeout

- Task ID: MCC-040
- Owner: codex
- Status: COMPLETE
- Validation: Fresh closeout gate recorded in `EVIDENCE_AND_GATES.md`.
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- The first slice is intentionally smaller than the full metadata refresh unit
  of work.
- Search adapter replacement remains out of scope.
- NFO merge policy work belongs in a separate lane.
- `CatalogItemProjectionCommit` intentionally uses `search.item_id` as the
  single item identity instead of duplicating the same ID at the commit root.
- MCC-030 deliberately stops at the metadata refresh persistence commit unit.
  Catalog hydration remains a separate workflow step, but its graph/search write
  is already atomic after MCC-020.
- The metadata refresh adapter, not the workflow caller, selects which Library
  Item State rows to confirm. This avoids passing a stale library ID list into
  the commit.

## Blockers

- Existing unrelated worktree changes touch `taru-core`, `taru-db`, and server
  files. Do not revert them. Read context before editing shared files.

## Next Recommended Action

If a larger metadata-refresh-plus-prepared-catalog or event-driven projection
design is still desired, open a new architecture or execution lane. Keep this
lane closed.
