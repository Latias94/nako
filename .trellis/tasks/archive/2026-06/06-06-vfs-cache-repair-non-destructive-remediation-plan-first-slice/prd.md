# VFS Cache Repair Non-Destructive Remediation Plan First Slice

## Goal

Add a read-only Admin remediation plan for unresolved VFS cache repair pressure
so operators can see grouped, safe next actions before Nako grows durable repair
queues, purge/delete actions, or broader cache mutation behavior.

## Requirements

- Add a versioned Admin read-only route:
  `GET /admin/v1/storage/vfs-cache/repair/remediation-plan`.
- Require the existing Admin route guard.
- Reuse current VFS cache repair diagnostics and target inventory semantics.
- Summarize unresolved repair targets by redaction-safe action groups:
  - `refresh_cache` executable targets,
  - `operator_action_required` plan-only targets,
  - `no_action` / non-actionable targets.
- Return safe aggregate fields only:
  - total unresolved targets,
  - per-action counts,
  - per-classification counts,
  - sample opaque `target_ref` values with operation/scheme/classification,
  - route key/path for existing selected-target refresh where executable,
  - boundary flags proving the plan is non-destructive.
- Keep `target_ref` opaque and process-local; do not expose raw URI, Source
  Locator, backend URL, local path, etag, fingerprint, credentials, provider
  error bodies, or raw backend authority.
- Keep the first slice read-only. It must not refresh, purge, delete,
  invalidate cache, enqueue durable jobs, mutate backend configuration, write
  library files, or start repair workers.
- Register the new route in Admin route inventory / generated Admin Web
  TypeScript contracts.
- Preserve existing latest-failure action-plan, target inventory, preview, and
  selected-target refresh behavior.

## Acceptance Criteria

- [ ] Admin can request a remediation plan and receive aggregate unresolved VFS
      cache repair pressure without mutation.
- [ ] A plan with refreshable targets includes executable route metadata for
      selected-target refresh and sample opaque target refs.
- [ ] A plan with operator-action targets remains plan-only and does not expose
      executable refresh metadata for those targets.
- [ ] Counts are grouped by recommended action and classification.
- [ ] Non-admin callers are rejected.
- [ ] Responses and errors do not contain raw URI/path/backend URL/etag/
      fingerprint/credential/raw backend error values.
- [ ] Existing target preview and target refresh focused tests continue to pass.
- [ ] Admin contract route inventory and generated Admin Web contract output
      cover the new route and DTOs.

## Definition Of Done

- `cargo check -p nako-api -p nako-server --tests` passes.
- Focused `cargo nextest` gates for VFS cache repair app/HTTP behavior pass.
- Focused `cargo nextest` gate for `nako-api` Admin contract generation passes.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- Trellis task validation passes.
- If this slice creates durable guidance, update the relevant Trellis spec.
- Commit only task-scoped changes with a Conventional Commit message.

## Technical Approach

- Add Admin DTOs in `nako-api::admin::storage` for remediation plan response,
  grouped counts, sample targets, and non-destructive boundary facts.
- Add a route key in `nako-api::admin_contract`:
  `storageVfsCacheRepairRemediationPlan`.
- Add a `StorageAppService::vfs_cache_repair_remediation_plan` method that
  pages through unresolved repair targets using the existing target inventory
  mechanics, classifies each target with existing repair plan logic, and
  returns only aggregate/sample facts.
- Add `GET /admin/v1/storage/vfs-cache/repair/remediation-plan` in
  `nako-server::http::admin`.
- Keep sampling bounded to avoid using the remediation plan as a raw target dump.
- Reuse existing selected-target refresh route metadata for executable
  `refresh_cache` samples.

## Decision (ADR-lite)

**Context**: VFS cache repair currently supports latest-failure planning,
target inventory, target previews, and selected-target refresh. The architecture
map still lists broader non-destructive remediation planning as a follow-on, but
durable repair queues and destructive cache changes remain intentionally
deferred.

**Decision**: Ship a read-only remediation plan first. It gives operators a
safe aggregate view and routes them to already-existing selected-target refresh
when applicable, without adding new mutation semantics.

**Consequences**: Future durable repair queue or broader remediation work can
reuse the grouped plan as an operator-facing contract. This slice still avoids
purge/delete/invalidation, retry queues, backend configuration mutation, and
hidden background work.

## Out Of Scope

- No cache purge, delete, invalidation, cleanup, or row mutation beyond existing
  read-only target lookup.
- No durable repair queue, retry queue, scheduler loop, runtime worker, or
  background task.
- No backend configuration mutation.
- No library file write.
- No Public Client API.
- No Admin Web page implementation beyond generated contract refresh.
- No database schema migration.

## Technical Notes

- Parent architecture map:
  `docs/architecture/STORAGE_VFS.md`.
- Predecessor task:
  `.trellis/tasks/archive/2026-06/06-05-vfs-cache-repair-executable-refresh-action/`.
- Relevant specs:
  `.trellis/spec/nako-server/backend/http-api-patterns.md`,
  `.trellis/spec/nako-server/backend/error-handling.md`,
  `.trellis/spec/nako-server/backend/quality-guidelines.md`,
  `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`,
  `.trellis/spec/nako-api/backend/quality-guidelines.md`,
  `.trellis/spec/nako-vfs/backend/index.md`.
