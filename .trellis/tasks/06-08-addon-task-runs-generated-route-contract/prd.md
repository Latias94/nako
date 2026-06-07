# Addon task-runs generated route contract

## Goal

Move the existing Admin Addon Task Run routes from explicit route-contract
exclusions into generated Admin contracts and expose a bounded Admin Web
operator workflow for observing and retrying Addon Task Runs.

This continues the overnight Jellyfin comparison campaign by closing one
implemented-but-unreachable Admin surface. Jellyfin exposes scheduled task list,
detail, start/stop, and trigger update routes; Nako's current domain is
different: an **Addon Task** is declared by an **Addon Sidecar**, while Nako owns
the task run lifecycle, progress model, retry boundary, and audit-safe summary.

## Requirements

- Add generated Admin route keys for:
  - `GET|POST /admin/v1/addons/{addon_id}/task-runs`
  - `GET /admin/v1/addons/{addon_id}/task-runs/{job_id}`
  - `POST /admin/v1/addons/{addon_id}/task-runs/{job_id}/retry`
- Remove those three route suffixes from `ADMIN_ROUTE_EXCLUSION_SUFFIXES`.
- Regenerate both Admin TypeScript contract copies:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Extend `AdminApiClient` and `AdminDataSource` with list/detail/retry methods
  for Addon Task Runs.
- Extend the Addons route summary with a bounded task-run panel for the selected
  addon.
- Render only redaction-safe run facts: job ID, declaration name/id, status,
  resource class, attempt counters, retryability, scope IDs, safe error code,
  and timestamps.
- Add a retry action only for retryable failed runs and require explicit
  confirmation before calling the mutation.
- Keep mock fallback deterministic. Mock fallback may show runs, but it must not
  fabricate successful retry mutations when live mutation is unavailable.
- Preserve existing Addons route status filter behavior and URL-owned state.

## Acceptance Criteria

- [ ] Admin route inventory tests prove Addon Task Run routes are generated, not
      excluded.
- [ ] Generated Admin contracts contain the new route keys and DTO types.
- [ ] Admin Web client methods build generated paths with encoded `addon_id` and
      `job_id`, include list query params, and POST retry payloads.
- [ ] Data source maps live Addon Task Run responses into route-local summary
      rows and falls back safely when read endpoints fail.
- [ ] Addons page renders a task-run panel for the selected addon.
- [ ] Retry button is disabled for mock/unavailable mutation and non-retryable
      runs.
- [ ] Confirmation step precedes retry mutation.
- [ ] Tests prove rendered Addons route text does not contain raw input,
      progress/result payload, token, backend URL, local path, or raw sidecar
      command material.

## Definition of Done

- Focused Rust gates:
  - `cargo check -p nako-api --tests`
  - `cargo check -p nako-server --tests`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
- Frontend gates:
  - `npm run check --prefix apps/admin-web`
  - `npm run test --prefix apps/admin-web`
- Formatting/whitespace:
  - `cargo fmt --all -- --check`
  - `git diff --check`
- Trellis task validation passes.
- Changes are committed with a Conventional Commit message after verification.

## Technical Approach

Use the existing generated Admin route inventory pattern in
`crates/nako-api/src/admin_contract.rs`. Do not hand-edit generated TypeScript
contracts.

Use the existing Addons route data path:

```text
AdminApiClient -> AdminDataSource -> AddonsRouteSummary -> AddonsPage
```

Keep the page broad enough for operator visibility but not a full task
scheduler. Addon Task Run creation, task triggers, cancellation, and generic
durable-job scheduling are out of scope.

## Decision (ADR-lite)

**Context**: Jellyfin's scheduled task controller exposes generic scheduled
task listing and start/stop/update-trigger commands. Nako already has a more
specific Addon Task Run route family implemented server-side, but Admin Web
route constants intentionally excluded it.

**Decision**: Generate and consume only the existing Nako Addon Task Run routes
for list/detail/retry. Do not add a generic scheduled-task abstraction or
Jellyfin-compatible task API.

**Consequences**: Operators can observe and retry Addon Sidecar-backed work
without broadening Nako's control-plane contract. Generic scheduler operations
remain a future ADR 0053-aligned task if release evidence demands them.

## Out of Scope

- Addon Task Run creation UI.
- Addon Task cancellation UI.
- Generic scheduled task manager.
- Trigger editing or recurring schedules.
- Raw task input/progress/result rendering.
- Addon token issue/rotate/revoke UI changes.
- Invitation route generation or artwork maintenance route generation.

## Research References

- [`research/jellyfin-scheduled-tasks-comparison.md`](research/jellyfin-scheduled-tasks-comparison.md)

## Technical Notes

- Nako server routes already exist in `crates/nako-server/src/http/addons.rs`.
- DTOs already exist in `crates/nako-api/src/extension.rs`.
- Current exclusions are in `crates/nako-api/src/admin_contract.rs`.
- Addons Admin Web route lives in `apps/admin-web/src/features/addons/AddonsPage.tsx`.
- Redaction boundary: do not render raw token, input, progress/result JSON,
  backend URL, filesystem path, sidecar hosted URL, shell command, device node,
  credential, or proxy secret.
