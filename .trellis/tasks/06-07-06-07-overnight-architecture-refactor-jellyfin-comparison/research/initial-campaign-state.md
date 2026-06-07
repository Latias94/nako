# Initial Campaign State

## Repository State

- Branch: `main`
- Working directory: clean at campaign start.
- Local `repo-ref/jellyfin` is available for architecture and workflow
  comparison.

## Recent Nako State

- VFS cache repair durable job contract, enqueue, executor, Admin manual
  enqueue/execute/retry, disk-scan scheduler integration, internal retry seam,
  and Admin Jobs diagnostics projection are shipped.
- Architecture maps now distinguish shipped repair-job diagnostics from future
  automated repair policy, cache mutation semantics, and realtime/incident
  diagnostics.

## Comparison Hypotheses

- Jellyfin likely has mature operational patterns around library scanning,
  scheduled tasks, media path/file-system abstractions, cache cleanup, and Admin
  diagnostics that can reveal missing Nako product workflows.
- Nako should not mirror Jellyfin's implementation shape mechanically. The
  useful output is Nako-native seams, tests, and policies that preserve Nako's
  Source Locator, Source Fingerprint, VFS cache repair authority, and
  redaction-safe Admin boundary.

## First Read-Only Targets

- Nako:
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `crates/nako-server/src/app/storage.rs`
  - `crates/nako-server/src/app/jobs.rs`
  - `crates/nako-library/src/source_hash.rs`
  - `crates/nako-vfs/src/lib.rs`
- Jellyfin:
  - storage/file-system abstraction names;
  - scheduled task and job orchestration names;
  - library scan and item refresh workflows;
  - cache cleanup / image / transcode artifact lifecycle names.
