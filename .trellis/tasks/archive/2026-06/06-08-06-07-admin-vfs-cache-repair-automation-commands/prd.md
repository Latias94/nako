# Admin VFS Cache Repair Automation Commands

## Context

The overnight architecture campaign compares Jellyfin's operator-visible
scheduled maintenance model with Nako's durable-job control plane. Nako now has
an internal VFS cache repair automation dry-run planner and an explicit internal
automation enqueue command. The next safe step is an Admin control surface that
lets operators inspect the automation policy result and explicitly enqueue
eligible durable repair jobs.

This slice must remain an explicit Admin command surface. It must not introduce
a recurring scheduler, startup worker, or raw background task.

## Goal

Expose VFS cache repair automation dry-run and explicit enqueue through the
Admin API using redaction-safe DTOs and generated Admin contract routes.

## Requirements

- Add Admin DTOs for:
  - automation policy request;
  - automation policy dry-run response;
  - automation enqueue response;
  - automation boundary, block reasons, targets, and job facts.
- Add Admin routes:
  - `POST /admin/v1/storage/vfs-cache/repair/automation/plan`
  - `POST /admin/v1/storage/vfs-cache/repair/automation/jobs`
- Both routes must inherit the existing Admin route guard.
- The plan route must not enqueue jobs or mutate storage.
- The jobs route must enqueue only eligible targets from the app-layer policy
  command and must not execute jobs.
- Responses must not expose raw `StorageUri`, paths, backend URLs,
  credentials, etags, fingerprints, URI digests, raw backend errors, durable
  input JSON, or cache payloads.
- Route inventory and generated Admin Web contract must be updated from
  `nako-api` sources, not hand-edited.
- Public Client route inventory and SDK/OpenAPI must remain free of Admin
  automation routes.

## Validation

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_vfs_cache_repair_automation --no-fail-fast`
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
- `cargo check -p nako-api -p nako-server --tests`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-08-06-07-admin-vfs-cache-repair-automation-commands`

## Non-Goals

- No recurring scheduler or background runtime task.
- No direct repair job execution from the automation jobs route.
- No cache purge/delete/invalidation, backend configuration mutation, or
  library file write.
- No Admin Web UI wiring beyond generated contract refresh.
- No Jellyfin source copying; reference usage remains architectural comparison
  only.
