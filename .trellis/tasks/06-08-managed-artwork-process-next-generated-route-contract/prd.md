# Managed artwork process-next generated route contract

## Goal

Move `POST /admin/v1/artwork/ingests/process-next` out of the explicit Admin
route exclusion list and into the generated Admin route/TypeScript contract
surface. This is the last remaining Admin route inventory exclusion; generating
it makes the inventory parity gate strict without exceptions while preserving
the existing Admin-only boundary.

## Requirements

- Add generated route key `managedArtworkIngestProcessNext` for
  `artwork/ingests/process-next`.
- Remove the final `artwork/ingests/process-next` exclusion from
  `admin_contract_route_exclusions()`.
- Refactor the Admin contract exclusion helper so the current exclusion list is
  empty without keeping stale suffix constants.
- Generate TypeScript DTO for:
  - `ProcessManagedArtworkIngestResponse`
- Reuse already generated `ManagedArtworkIngestSummary`,
  `ManagedArtworkArtifactSummary`, and `JobResponse` shapes where applicable.
- Regenerate:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Add Admin Web client method `processNextManagedArtworkIngest()`:
  - uses the generated route key
  - sends `POST` with an empty JSON body
- Add focused client tests for generated route usage, empty body, response
  typing for both processed and empty responses, and redaction fixture terms.
- Keep route Admin-only and out of Public Client inventories/SDKs.

## Acceptance Criteria

- [x] `cargo nextest run -p nako-api admin_contract --no-fail-fast` passes.
- [x] `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast` passes.
- [x] `cargo nextest run -p nako-server admin_process_next_managed_artwork_ingest --no-fail-fast` passes.
- [x] `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` passes.
- [x] `npm run check --prefix apps/admin-web` passes.
- [x] Generated TypeScript artifacts are produced from the Rust generator, not hand-edited.
- [x] Admin route inventory has no explicit exclusions.

## Definition of Done

- Code, generated artifacts, specs, task evidence, and focused tests are updated
  together.
- `cargo fmt --all -- --check`, `git diff --check`, and Trellis task
  validation pass.
- Commit with a Conventional Commit message, then archive the task in a
  separate chore commit.

## Technical Approach

Update `crates/nako-api/src/admin_contract.rs` route inventory and TypeScript
contract body. Add the generated route key, emit
`ProcessManagedArtworkIngestResponse`, and remove the now-empty exclusion
constant/adapter path. Regenerate both TypeScript contract copies. Add a typed
Admin Web client method next to other Managed Artwork client methods.

Existing server behavior stays unchanged: the route delegates to
`app.artwork().process_next()`, returns `ProcessManagedArtworkIngestResponse`,
and focused server tests already cover storing the internal artifact without
public artwork, empty queues, unsupported media type failure, invalid image
failure, and redaction of raw provider URL/token/storage material.

## Decision (ADR-lite)

Context: Jellyfin exposes elevated Scheduled Task start/stop controls through a
controller that delegates to task manager/worker infrastructure. Nako already
has `process-next` as an Admin-only manual worker command with safe response
DTOs and route tests.

Decision: Generate the `process-next` route as a low-level Admin client command
without adding UI controls. This removes the last route inventory exception and
keeps process execution behind the existing Admin route guard and app-service
boundary.

Consequences: Admin route parity becomes simpler: every implemented Admin route
is generated, and the explicit exclusion list is empty. Any future page that
invokes this method still needs a dedicated live-only workflow task.

## Out of Scope

- No Admin Web page button, workflow, queue dashboard, or confirmation modal in
  this slice.
- No change to ingest processing, provider fetch, artifact storage,
  publication, worker scheduling, or runtime supervision behavior.
- No Public Client API exposure.

## Research References

- [`research/jellyfin-scheduled-task-start-comparison.md`](research/jellyfin-scheduled-task-start-comparison.md)
  - Jellyfin comparison for elevated scheduled task start/stop versus Nako
    generated manual process-next command.

## Technical Notes

- Relevant Nako files:
  - `crates/nako-api/src/admin/managed_artwork.rs`
  - `crates/nako-api/src/admin_contract.rs`
  - `crates/nako-server/src/http/admin.rs`
  - `crates/nako-server/src/http/tests/addons.rs`
  - `apps/admin-web/src/adminApi/client.ts`
  - `apps/admin-web/src/adminApi/client.test.ts`
