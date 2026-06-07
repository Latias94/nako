# VFS Cache Repair Automation Enqueue Command

## Context

Jellyfin exposes maintenance work as explicit scheduled tasks. Nako should not
copy that in-process task-runner shape directly because Nako already owns
durable jobs, resource classes, redaction-safe job input, retry seams, and
disk-scan scheduler execution.

Nako now has:

- an internal dry-run VFS cache repair automation policy planner;
- one shared unresolved target collection boundary;
- an existing target-scoped durable enqueue authority;
- existing scheduler execution for queued VFS cache repair jobs.

The next safe slice is an explicit internal automation enqueue command. It can
turn the planner's eligible targets into durable queued jobs by reusing the
target enqueue authority, but it must not add a background scheduler loop or an
Admin route yet.

## Goal

Add an internal `StorageDiagnosticsAppService` command that applies an explicit
VFS cache repair automation policy by enqueueing eligible dry-run targets
through the existing target enqueue authority and returning a redaction-safe
summary.

## Requirements

- The command must accept `VfsCacheRepairAutomationPolicy` and optional
  `JobPriority`.
- Disabled policy must not enqueue jobs and must return a policy report showing
  blocked targets.
- Enabled policy may enqueue only targets that the dry-run planner marks
  eligible.
- Reuse `enqueue_vfs_cache_repair_target` for durable job creation and
  idempotency.
- The command report must distinguish newly enqueued jobs from already queued
  jobs without exposing raw target material or job input JSON.
- Preserve current dry-run planner behavior and report shape.
- Preserve all redaction boundaries: no raw `StorageUri`, paths, backend URLs,
  credentials, etags, fingerprints, URI digests, raw backend errors, or durable
  input JSON in new report surfaces.
- Do not refresh cache, execute jobs, purge/delete/invalidate cache entries,
  mutate backend configuration, write library files, or create a scheduler loop.

## Validation

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-server vfs_cache_repair_automation --no-fail-fast`
- `cargo nextest run -p nako-server vfs_cache_repair --no-fail-fast`
- `cargo check -p nako-server --tests`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-07-06-07-vfs-cache-repair-automation-enqueue-command`

## Non-Goals

- No Admin/Public route changes.
- No startup scheduler, recurring scheduler, or raw `tokio::spawn`.
- No direct durable job execution.
- No cache purge/delete/invalidation or backend configuration workflow.
- No Jellyfin source copying; reference use remains architecture comparison
  only.
