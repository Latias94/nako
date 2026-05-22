# Phase 14.0: Scheduling And Lifecycle

## Goal

Extend manual metadata maintenance into a lifecycle boundary with dry-run
planning, scheduled policy enqueueing, background raw cache cleanup, and
provider backoff.

## Design

Maintenance policies live under `metadata.maintenance.policies`. Each enabled
policy maps to the same request shape as `POST /metadata/maintenance/jobs`:

- `library_id` or `item_ids`;
- provider override;
- profile override;
- kind filter;
- language;
- refresh mode;
- force.

Policies enqueue normal `metadata_maintenance` jobs after `initial_delay_ms` and
then every `interval_ms`. The scheduler is process-local and intentionally does
not persist run history yet.

`POST /metadata/maintenance/plan` uses the same request body and returns the
items that would be processed with their effective providers, language, and
refresh mode. Planning does not persist a job, call providers, write raw cache,
or hydrate catalog/search state.

Raw cache lifecycle is controlled by:

- `metadata.raw_cache_retention_ms`;
- `metadata.maintenance.raw_cache_cleanup_on_startup`;
- `metadata.maintenance.raw_cache_cleanup_interval_ms`.

Provider backoff is controlled by `circuit_breaker_failures` and
`circuit_breaker_backoff_ms`. When the runtime opens the circuit, provider calls
fail fast until the open-until timestamp expires. Diagnostics expose both the
configured backoff and current process-local open-until state.

## Current Limits

- Scheduler state is process-local.
- Missed schedules are not replayed after restart.
- Raw cleanup is single-process.
- Provider backoff is in-memory.

These limits are acceptable until Nako supports multi-instance server
deployments.

## Validation

- Runtime tests cover circuit open, open-until state, and fail-fast behavior.
- App tests cover startup raw cache cleanup and policy-to-request mapping.
- HTTP tests cover dry-run plan and queued job routes.
- Workspace tests cover that planning does not break existing refresh and
  diagnostics paths.
