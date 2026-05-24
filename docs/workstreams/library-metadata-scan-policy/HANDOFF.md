# Library Metadata Scan Policy - Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

The NFO scan-time metadata acquisition lane is closed. Real smoke before
implementation proved:

- `H:\Super\Videos` scan discovered 5 videos, probed 5, and had 0 ingestion
  failures.
- Manual `import-nfo` then discovered 3 NFO files and imported 3 items.
- Local playback decision and Range stream passed with direct play.
- NAS single-directory SMB scan discovered 1 video, probed 1, and direct-played
  over HTTP Range.

The shipped change closes the user-visible gap: scan now applies configured NFO
metadata when the Media Library's Metadata Profile enables scan-time local NFO
acquisition.

## Completed Task

- Task ID: LMSP-050
- Owner: codex
- Scope: `docs/workstreams/library-metadata-scan-policy`
- Goal: Close the NFO scan-time metadata slice and split follow-ons.
- Validation: focused gates plus real local/NAS smoke pass.

## Dirty Worktree Notes

At lane open, `git status` already showed unrelated addon task runtime changes:

- `crates/nako-core/src/addon_task.rs`
- `crates/nako-db/migrations/0037_addon_task_runs.sql`
- `crates/nako-db/migrations/postgres/0009_addon_task_runs.sql`
- `crates/nako-db/src/postgres/addon_tasks.rs`
- `crates/nako-db/src/sqlite/addon_tasks.rs`
- `crates/nako-server/src/app/addons/task_runtime.rs`
- `crates/nako-server/src/http/tests/addons.rs`

Treat these as user/other-session changes. Do not restore, delete, format,
stage, or commit them unless explicitly requested.

## Follow-Ons

- Provider refresh as a planned scan acquisition step.
- Addon Bulk Metadata Scrape using Addon Task lifecycle and grants.
- Embedded metadata and additional local sidecar readers.
- Image/artwork discovery routed through Managed Artwork boundaries.
- Full NAS root scan only after better progress/cancellation visibility for
  large SMB trees.

## Next Recommended Action

Start a new narrow workstream for the next metadata acquisition step rather
than reopening this NFO-only lane.
