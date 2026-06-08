# Managed artwork candidate accept generated route contract

## Goal

Move `POST /admin/v1/artwork/candidates/{candidate_id}/accept` out of the explicit Admin route exclusion list and into the generated Admin route/TypeScript contract surface. Candidate acceptance is an explicit operator selection that queues Managed Artwork ingestion; it should be typed for Admin Web while preserving redaction and Admin-only boundaries.

## Requirements

- Add generated route key `managedArtworkCandidateAccept` for `artwork/candidates/{candidate_id}/accept`.
- Remove only `artwork/candidates/{candidate_id}/accept` from `admin_contract_route_exclusions()`.
- Generate TypeScript DTOs for:
  - `AcceptManagedArtworkCandidateResponse`
  - `JobResponse`
- Reuse already generated `ManagedArtworkIngestSummary`, `AdminJobStatus`, `AdminJobPriority`, and `AdminJobDiagnostics` shapes where applicable.
- Regenerate:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Add Admin Web client method `acceptManagedArtworkCandidate(candidateId)`:
  - uses the generated route key
  - encodes `candidate_id`
  - sends `POST` with an empty JSON body
- Add focused client tests for generated route usage, encoded path param, empty body, response typing, and redaction fixture terms.
- Keep route Admin-only and out of Public Client inventories/SDKs.

## Acceptance Criteria

- [x] `cargo nextest run -p nako-api admin_contract --no-fail-fast` passes.
- [x] `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast` passes.
- [x] `cargo nextest run -p nako-server admin_accept_artwork_candidate --no-fail-fast` passes.
- [x] `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` passes.
- [x] `npm run check --prefix apps/admin-web` passes.
- [x] Generated TypeScript artifacts are produced from the Rust generator, not hand-edited.

## Definition of Done

- Code, generated artifacts, specs, task evidence, and focused tests are updated together.
- `cargo fmt --all -- --check`, `git diff --check`, and Trellis task validation pass.
- Commit with a Conventional Commit message, then archive the task in a separate chore commit.

## Technical Approach

Update `crates/nako-api/src/admin_contract.rs` route inventory and TypeScript contract body, including the `JobResponse` command response shape used by `AcceptManagedArtworkCandidateResponse`. Regenerate both TypeScript contract copies. Add a typed Admin Web client method next to other Managed Artwork client methods. Existing server tests already cover candidate acceptance redaction, idempotent replay, queued ingest creation, and no immediate public selected artwork.

## Decision (ADR-lite)

Context: Jellyfin exposes remote image download as an elevated explicit item-scoped user action. Nako's candidate accept route is similar, but Nako separates selection from ingestion by queuing a Managed Artwork ingest job.

Decision: Generate the accept route now without adding a confirm query, because the target is an opaque candidate ID selected by an Admin user and the server queues an ingest job rather than deleting or directly writing public artwork.

Consequences: Admin Web gets a typed selection command while remaining unable to submit raw URLs, paths, tokens, or artifact storage details.

## Out of Scope

- No Admin Web page button or confirmation modal in this slice.
- No generated route for `artwork/ingests/process-next` or `artwork/ingests/{ingest_id}/requeue`.
- No change to candidate acceptance, ingest queue, provider fetch, or publication behavior.
- No Public Client API exposure.

## Research References

- [`research/jellyfin-remote-image-selection-comparison.md`](research/jellyfin-remote-image-selection-comparison.md) - Jellyfin comparison for explicit remote image download/selection.

## Technical Notes

- Relevant Nako files:
  - `crates/nako-api/src/admin/managed_artwork.rs`
  - `crates/nako-api/src/admin/operations.rs`
  - `crates/nako-api/src/admin_contract.rs`
  - `crates/nako-server/src/http/admin.rs`
  - `crates/nako-server/src/http/tests/addons.rs`
  - `apps/admin-web/src/adminApi/client.ts`
  - `apps/admin-web/src/adminApi/client.test.ts`
