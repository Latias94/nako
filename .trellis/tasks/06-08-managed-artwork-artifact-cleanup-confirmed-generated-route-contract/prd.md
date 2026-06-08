# Managed artwork artifact cleanup confirmed generated route contract

## Goal

Promote `POST /admin/v1/artwork/artifacts/cleanup` from an explicit Admin route exclusion to a generated Admin contract only after adding an explicit `confirm=true` safety boundary. The route deletes unselected Managed Artwork artifacts and best-effort artifact files, so it should not be generated for Admin Web while it can execute from an unconfirmed POST.

## Requirements

- Add HTTP query parsing that requires `confirm=true` before `cleanup_admin_artwork_artifacts` delegates to `cleanup_unselected_artifacts`.
- Keep cleanup bounded by pagination: `limit` and `offset` remain supported.
- Do not expose or rely on `cleanup_candidates_only` for cleanup. That filter belongs to read-only lifecycle diagnostics; cleanup always targets repository-owned cleanup candidates.
- Add a generated Admin route key for `POST /admin/v1/artwork/artifacts/cleanup`, tentatively `managedArtworkArtifactCleanup`.
- Remove only `artwork/artifacts/cleanup` from `admin_contract_route_exclusions()`.
- Generate TypeScript DTOs for:
  - `AdminManagedArtworkArtifactCleanupQuery`
  - `AdminManagedArtworkArtifactCleanupResponse`
  - `AdminManagedArtworkArtifactCleanupItem`
- Regenerate:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Add an Admin Web client method, tentatively `cleanupManagedArtworkArtifacts(query)`, that uses the generated route, serializes `confirm`, `limit`, and `offset` as query params, and sends an empty JSON body.
- Update server route tests so missing confirmation returns invalid input and does not delete DB records or files.
- Keep response/client fixtures redaction-safe: no `storage_uri`, `managed-artwork://`, `source_uri`, `cache_uri`, raw content hash, local path, token, raw file name, or provider URL.

## Acceptance Criteria

- [x] Missing `confirm=true` on `/admin/v1/artwork/artifacts/cleanup` is rejected and preserves selected/unselected artifacts.
- [x] Confirmed cleanup still removes only unselected cleanup candidates and preserves selected artwork.
- [x] `cargo nextest run -p nako-api admin_contract --no-fail-fast` passes.
- [x] `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast` passes.
- [x] `cargo nextest run -p nako-server admin_managed_artwork_cleanup --no-fail-fast` passes.
- [x] `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` passes.
- [x] `npm run check --prefix apps/admin-web` passes.
- [x] Generated TypeScript artifacts are produced from the Rust generator, not hand-edited.

## Definition of Done

- Code, generated artifacts, specs, task evidence, and focused tests are updated together.
- `cargo fmt --all -- --check`, `git diff --check`, and Trellis task validation pass.
- Commit with a Conventional Commit message, then archive the task in a separate chore commit.

## Technical Approach

Create a cleanup-specific query type in `crates/nako-server/src/http/query.rs` instead of reusing the lifecycle query. This keeps read-only lifecycle filters separate from executable cleanup confirmation. Update the existing server test to assert the new rejection path first, then call `?confirm=true` for the existing success path. Add generated route/query/response types in `crates/nako-api/src/admin_contract.rs`, regenerate both TypeScript contract copies, then add the Admin Web client method and focused client test.

## Decision (ADR-lite)

Context: Jellyfin's comparable image cleanup code treats image deletion as a derived maintenance action after identifying dead image paths and filtering to known image extensions. Nako already has repository-owned cleanup candidates and redaction-safe response DTOs, but the HTTP route currently executes without explicit confirmation.

Decision: Require `confirm=true` before generating the cleanup route for Admin Web. Keep the cleanup target discovery server-owned and do not accept raw artifact/file identity in the request body.

Consequences: The route becomes safer to expose through generated Admin contracts. Existing unconfirmed callers must update to an explicit confirmation query, which is acceptable for an Admin maintenance mutation.

## Out of Scope

- No change to the repository cleanup algorithm.
- No Admin Web page button or confirmation modal in this slice.
- No deletion of stray files; that is covered by the separate stray-file remediation route.
- No Public Client API exposure.
- No durable job, scheduler, migration, or background runtime change.

## Research References

- [`research/jellyfin-artwork-cleanup-comparison.md`](research/jellyfin-artwork-cleanup-comparison.md) - Jellyfin comparison for conservative image cleanup behavior.

## Technical Notes

- Relevant Nako files:
  - `crates/nako-server/src/http/admin.rs`
  - `crates/nako-server/src/http/query.rs`
  - `crates/nako-server/src/http/tests/addons.rs`
  - `crates/nako-api/src/admin/managed_artwork.rs`
  - `crates/nako-api/src/admin_contract.rs`
  - `apps/admin-web/src/adminApi/client.ts`
  - `apps/admin-web/src/adminApi/client.test.ts`
- Relevant specs:
  - `.trellis/spec/nako-api/backend/quality-guidelines.md`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
