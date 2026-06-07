# VFS Cache Repair Target Collection Refactor

## Context

The overnight architecture campaign compares Nako's Storage/VFS and control
plane evolution with Jellyfin's explicit scheduled maintenance model. Nako
already has durable jobs and resource budgets, so VFS cache repair automation
should deepen existing Nako-native seams instead of adding a broad in-process
task runner.

The previous slice added a non-mutating VFS cache repair automation policy
planner. That planner and the existing remediation planner both page through
unresolved VFS cache failures and classify them through the same target
authority. This duplicated scan loop is small, but it is exactly the seam that
future automatic enqueue policy will depend on. Keeping multiple loops invites
dry-run/execution drift.

## Goal

Create one internal StorageDiagnosticsAppService target collection boundary for
unresolved VFS cache repair targets, then reuse it from both remediation and
automation planners.

## Requirements

- Preserve current behavior and public/internal report shapes.
- Keep the collection boundary internal to `nako-server::app::storage`.
- Reuse the existing latest-target authority and redaction-safe diagnostic
  target reports.
- Do not enqueue jobs, execute repairs, refresh cache, purge/delete/invalidate
  cache entries, mutate backend configuration, write library files, or change
  durable job input/summary/error contracts.
- Avoid exposing raw `StorageUri`, paths, backend URLs, credentials, etags,
  fingerprints, URI digests, raw backend errors, or durable input JSON.
- Keep pagination semantics equivalent to the existing planner loops.

## Validation

- `cargo fmt --all -- --check`
- Focused `cargo nextest run -p nako-server` tests covering VFS cache repair
  remediation and automation planners.
- `cargo check -p nako-server --tests`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-07-06-07-vfs-cache-repair-target-collection-refactor`

## Non-Goals

- No Admin route changes.
- No scheduler/background-loop changes.
- No automatic enqueue or execution policy.
- No cache cleanup, purge, delete, or invalidation behavior.
- No Jellyfin source copying; reference usage remains architectural comparison
  only.
