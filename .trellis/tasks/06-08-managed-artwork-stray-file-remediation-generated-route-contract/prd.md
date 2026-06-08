# Managed artwork stray-file remediation generated route contract

## Goal

Move the existing Admin route `POST /admin/v1/artwork/artifacts/remediate-stray-files` out of the explicit generated-contract exclusion list and into the generated Admin route/TypeScript contract surface. This gives Admin Web a typed, generated call path for the already-confirmed remediation command while preserving the existing operator safety and redaction boundaries.

## Requirements

- Add a generated Admin route key for `POST /admin/v1/artwork/artifacts/remediate-stray-files`.
- Keep the route Admin-only; do not expose it through Public Client route inventories, OpenAPI public outputs, or generated Public SDKs.
- Model the route as an explicit confirmed mutation:
  - query: `confirm?: boolean`, `file_scan_limit?: number`
  - body: empty POST body from Admin Web client
- Generate TypeScript DTOs for the existing Rust response shape:
  - `AdminManagedArtworkArtifactStrayFileCleanupResponse`
  - `AdminManagedArtworkArtifactStrayFileCleanupSummary`
  - `AdminManagedArtworkArtifactStrayFileCleanupItem`
  - `AdminManagedArtworkArtifactStrayFileCleanupStatus`
  - generated query type if the local generator pattern requires named query DTOs
- Remove only the matching suffix from `admin_contract_route_exclusions()`.
- Regenerate both generated Admin TypeScript contract copies from `nako-api`:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Add an Admin Web client method, tentatively `remediateManagedArtworkArtifactStrayFiles(query)`, that uses `NAKO_ADMIN_ROUTES` and sends `POST` with an empty JSON body.
- Add focused tests proving:
  - generated route inventory includes the route
  - Admin Web client sends `confirm=true&file_scan_limit=...`
  - Admin Web client uses `POST` and an empty body
  - generated/client-facing contract does not expose `storage_uri`, `managed-artwork://`, `source_uri`, `cache_uri`, `content_hash`, raw paths, tokens, or similar sensitive storage material

## Acceptance Criteria

- [x] `cargo nextest run -p nako-api admin_contract --no-fail-fast` passes.
- [x] `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast` passes.
- [x] `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` passes.
- [x] `npm run check --prefix apps/admin-web` passes or any unrelated pre-existing blocker is recorded.
- [x] Generated TypeScript artifacts are produced from the Rust generator, not hand-edited.
- [x] The response/query contract remains redaction-safe and Admin-only.

## Definition of Done

- Code, generated artifacts, and tests are updated together.
- Focused Rust and Admin Web validation gates are run.
- `cargo fmt --all -- --check` and `git diff --check` are run.
- Task context is valid through `python ./.trellis/scripts/task.py validate`.
- Commit with a Conventional Commit message after verification.

## Technical Approach

Use the existing pattern from recently generated Admin mutation routes. Update `crates/nako-api/src/admin_contract.rs` as the source of truth, regenerate TypeScript artifacts with the existing generator commands, then update `apps/admin-web/src/adminApi/client.ts` and `client.test.ts` to exercise the generated route key and query serialization. Server behavior should not change except route inventory exclusion removal.

## Decision (ADR-lite)

Context: `remediate-stray-files` is a mutating Managed Artwork maintenance route, but it already has an explicit `confirm=true` boundary and server tests that protect against deleting tracked/active files and leaking raw storage material.

Decision: Generate the route contract now, while keeping higher-risk routes such as artifact cleanup excluded until a dedicated confirmation and artifact-deletion policy task is completed.

Consequences: Admin Web gains a typed command path for one safe maintenance mutation. The remaining exclusion list shrinks and route inventory drift risk decreases without broadening Public Client API scope.

## Out of Scope

- No change to remediation deletion policy or storage traversal semantics.
- No Admin Web page button or full UI workflow in this slice.
- No generated contract for `artwork/artifacts/cleanup`.
- No Public Client API exposure.
- No schema, migration, durable job, or background runtime change.

## Technical Notes

- Relevant DTO source: `crates/nako-api/src/admin/managed_artwork.rs`.
- Relevant generated contract source: `crates/nako-api/src/admin_contract.rs`.
- Relevant route and query source: `crates/nako-server/src/http/admin.rs`, `crates/nako-server/src/http/query.rs`.
- Relevant Admin Web client source: `apps/admin-web/src/adminApi/client.ts`, `apps/admin-web/src/adminApi/client.test.ts`.
- Existing server redaction/safety evidence: `crates/nako-server/src/http/tests/addons.rs` and managed artwork remediation workstream docs.
- Specs loaded: `nako-api/backend`, `nako-server/backend`, `admin-web/frontend`, shared cross-layer and code-reuse guides.
