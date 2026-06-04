# VFS cache repair action preview first slice

## Goal

Make the existing Admin VFS cache repair preview easier for operators and UI
clients to route by adding a stable structured recommended action alongside the
current redaction-safe operator action text.

## Background

`/admin/v1/storage/staging` already exposes `summary.vfs_cache.repair` with a
classification, operation, failure class, retryable flag, safe message, and
operator action text. The next proposed storage/VFS lane is cache repair
operator actions, but adding real mutation or cache invalidation semantics is
too broad for this slice.

This task keeps the endpoint read-only. It turns the current free-form action
text into a typed preview signal that future Admin UI and repair endpoints can
use without parsing prose.

## Requirements

* Add a VFS-owned structured cache repair action enum derived from the same
  repair classification used today.
* Expose the action through the Admin DTO as
  `recommended_action` in `AdminVfsCacheRepairDiagnostic`.
* Preserve the existing `operator_action` text for display compatibility.
* Keep all repair diagnostics redaction-safe: no cache URI, source locator,
  local path, etag, fingerprint, credential, token, or raw backend error.
* Keep `/admin/v1/storage/staging` read-only. Do not add a refresh/delete/retry
  route, durable job, queue, or schema migration.
* Refresh the generated Admin TypeScript contracts from `nako-api` if the Admin
  DTO shape changes; do not hand-edit generated output or change Admin Web
  business logic.
* Update storage/VFS architecture/spec notes so future tasks know that this
  slice is action preview only, not executable remediation.

## Proposed Action Vocabulary

* `none`: cache is healthy; no operator action is recommended.
* `refresh_cache`: stale fallback or retryable refresh failure can be refreshed
  after backend recovery.
* `fix_backend_configuration`: permission/security failures need operator
  configuration work before refresh.
* `inspect_failure`: unknown failures need inspection before a safe automated
  repair can be chosen.

## Acceptance Criteria

* VFS diagnostic tests cover the recommended action for healthy, stale fallback,
  retryable, permission/security, and unknown classifications.
* `nako-api` serialization test proves `recommended_action` is snake_case and
  does not leak sensitive strings.
* Server storage diagnostics test proves `/admin/v1/storage/staging` maps the
  structured action and keeps existing redaction assertions.
* Focused build/test gates pass and evidence is recorded.

## Verification Plan

* `cargo fmt --all -- --check`
* `cargo check -p nako-vfs -p nako-api -p nako-server --tests`
* `cargo nextest run -p nako-vfs cache --no-fail-fast`
* `cargo nextest run -p nako-api admin_vfs_cache_summary_serializes_redacted_repair_preview --no-fail-fast`
* `cargo nextest run -p nako-api admin_contract --no-fail-fast`
* `cargo nextest run -p nako-server admin_storage_staging_lists_records_and_summarizes_pressure --no-fail-fast`
* `git diff --check`
* `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-06-04-vfs-cache-repair-action-preview-first-slice`

## Out Of Scope

* Cache refresh, invalidation, delete, retry, or repair mutation endpoints.
* Durable jobs or operator queues.
* Database schema or repository contract changes.
* Admin Web or web business logic changes beyond generated contract refresh.
* Source fingerprint hash execution or playback artifact I/O pressure policy.
