# Managed artwork ingest requeue generated route contract

## Goal

Move `POST /admin/v1/artwork/ingests/{ingest_id}/requeue` out of the explicit
Admin route exclusion list and into the generated Admin route/TypeScript
contract surface. Requeue is an explicit operator retry command for a failed
Managed Artwork ingest; it should be typed for Admin Web while preserving
redaction and keeping worker execution internals out of the generated client.

## Requirements

- Add generated route key `managedArtworkIngestRequeue` for
  `artwork/ingests/{ingest_id}/requeue`.
- Remove only `artwork/ingests/{ingest_id}/requeue` from
  `admin_contract_route_exclusions()`.
- Keep `artwork/ingests/process-next` as the only remaining Managed Artwork
  route exclusion in this slice.
- Generate TypeScript DTOs for:
  - `RequeueManagedArtworkIngestResponse`
  - `ManagedArtworkIngestJobSummary`
- Reuse already generated `ManagedArtworkIngestSummary`, `AdminJobStatus`, and
  related safe Admin job enum shapes where applicable.
- Regenerate:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Add Admin Web client method `requeueManagedArtworkIngest(ingestId)`:
  - uses the generated route key
  - encodes `ingest_id`
  - sends `POST` with an empty JSON body
- Add focused client tests for generated route usage, encoded path param, empty
  body, response typing, replay-safe response shape, and redaction fixture
  terms.
- Keep route Admin-only and out of Public Client inventories/SDKs.

## Acceptance Criteria

- [x] `cargo nextest run -p nako-api admin_contract --no-fail-fast` passes.
- [x] `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast` passes.
- [x] `cargo nextest run -p nako-server admin_managed_artwork_ingest_requeue --no-fail-fast` passes.
- [x] `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` passes.
- [x] `npm run check --prefix apps/admin-web` passes.
- [x] Generated TypeScript artifacts are produced from the Rust generator, not hand-edited.

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
`RequeueManagedArtworkIngestResponse` and `ManagedArtworkIngestJobSummary`, and
remove only the requeue suffix from exclusions. Regenerate both TypeScript
contract copies. Add a typed Admin Web client method next to other Managed
Artwork client methods.

Existing server behavior stays unchanged: the route delegates to
`app.artwork().requeue_ingest(ingest_id)`, returns
`RequeueManagedArtworkIngestResponse`, and focused server tests already cover
failed-ingest retry, idempotent replay, stored-ingest conflict, and redaction of
raw provider URL/token/job payload material.

## Decision (ADR-lite)

Context: Jellyfin exposes elevated Scheduled Task start/stop controls, but its
controller talks to a task manager/worker rather than exposing arbitrary worker
internals. Nako's Managed Artwork `requeue` is the analogous operator retry
command for one known failed ingest, while `process-next` is a low-level worker
execution hook.

Decision: Generate the `requeue` route now and keep `process-next` excluded.
The Admin Web client can retry a selected failed ingest by ID, but it still
cannot execute arbitrary ingest worker steps.

Consequences: Admin Web gets a typed retry command with safe ingest/job
summaries. The remaining exclusion list becomes a single internal worker route,
which makes future architecture review more explicit.

## Out of Scope

- No generated route for `artwork/ingests/process-next`.
- No Admin Web page button, workflow, or confirmation modal in this slice.
- No change to ingest requeue, retry, worker execution, provider fetch, or
  publication behavior.
- No Public Client API exposure.

## Research References

- [`research/jellyfin-scheduled-task-command-comparison.md`](research/jellyfin-scheduled-task-command-comparison.md)
  - Jellyfin comparison for elevated scheduled task start/stop versus Nako
    generated retry command and internal process-next worker hook.

## Technical Notes

- Relevant Nako files:
  - `crates/nako-api/src/admin/managed_artwork.rs`
  - `crates/nako-api/src/admin_contract.rs`
  - `crates/nako-server/src/http/admin.rs`
  - `crates/nako-server/src/http/tests/addons.rs`
  - `apps/admin-web/src/adminApi/client.ts`
  - `apps/admin-web/src/adminApi/client.test.ts`
