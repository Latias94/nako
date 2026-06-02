# Provider Governance Public Contract Evidence

Date: 2026-06-02
Selected slice: Public Client API governance.

## Selection

Chose the minimal Public Client API governance slice because the current
Candidate Review, Provider Mapping application, durable batch, and Admin/Web
governance chain already preserves Admin-only mutation and redaction semantics.
This slice only adds focused negative contract tests proving durable Candidate
Review and provider governance route/type names do not enter Public OpenAPI or
generated SDK output.

## Audit Summary

- Candidate Review detail/apply, queue, batch plan/apply, and durable batch
  status are registered under Admin API route suffixes in `nako-api`.
- Existing route inventory coverage proves all Admin route suffixes stay out of
  the Public Client route inventory.
- Public OpenAPI and generated SDK tests already rejected generic Admin,
  internal, addon, automation, storage, jobs, and raw provider response terms.
- The missing trust/visibility gap was explicit coverage for provider
  governance route/type names such as `metadata/candidate-reviews`,
  `catalog/governance`, `ProviderMappingReview`, and
  `MetadataCandidateReview`.

## Boundaries Preserved

- No schema changes.
- No new Public Client API route.
- No generated contract refresh needed because route/DTO output is unchanged.
- No Provider Mapping write path change; root-only writes remain untouched.
- No raw provider payload, raw provider response, idempotency key, or source
  fingerprint field was added to Public OpenAPI or SDK output.

## Validation

- `cargo nextest run -p nako-api provider_governance_routes_and_types --no-fail-fast`
  passed: 3 tests.
- `cargo nextest run -p nako-api admin_contract public_openapi_excludes_provider_governance_routes_and_types typescript_sdk_excludes_provider_governance_routes_and_types kotlin_sdk_excludes_provider_governance_routes_and_types provider_governance_route_shapes --no-fail-fast`
  passed: 9 tests.
- `cargo nextest run -p nako-api --no-fail-fast` passed: 79 tests.
- `cargo nextest run -p nako-client-protocol public_route_inventory --no-fail-fast`
  passed: 3 tests.
- `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`
  passed: 6 tests.
- `cargo nextest run -p nako-db metadata_candidate_review --no-fail-fast`
  passed: 3 tests.
- `cargo nextest run -p nako-server metadata_candidate_review --no-fail-fast`
  passed: 8 tests.
- `cargo check -p nako-core -p nako-metadata -p nako-api -p nako-server -p nako-db --tests`
  passed.
- `npm --prefix apps/admin-web ci` installed lockfile dependencies without
  package-lock changes.
- `npm --prefix apps/admin-web run check` passed.
- `npm --prefix apps/admin-web run test` passed: 6 files / 160 tests.
- `python -m json.tool ./.trellis/tasks/06-02-03c-provider-governance-audit-public-contract/task.json`
  passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with LF/CRLF normalization warnings only.

## Reviewer Follow-Up

- Strengthened the test-only forbidden term inventory so Public OpenAPI,
  TypeScript SDK, and Kotlin SDK checks cover snake_case, hyphen-case,
  compact/camelCase-lowered, and human-readable description variants for
  Candidate Review, Provider Governance, Provider Mapping review, batch apply,
  idempotency key, source fingerprint, raw provider payload, and raw provider
  response terms.
- Added a provider-governance-specific Public Client route inventory guard for
  Candidate Review and Provider Mapping review route shapes without blocking
  unrelated future Public Client routes.
- Fixed the final newline in `task.json`.
- No schema, Public route, DTO, Provider Mapping write path, metadata service,
  server route, or database adapter path was changed.

## Fresh Integration Evidence

Date: 2026-06-02

- Synced current `main` into the worktree with `git merge --no-edit main`; no
  conflicts.
- `cargo fmt --all -- --check` passed.
- `cargo nextest run -p nako-api provider_governance_routes_and_types --no-fail-fast`
  passed: 3 tests.
- `cargo nextest run -p nako-api --no-fail-fast` passed: 79 tests.
- `cargo nextest run -p nako-client-protocol public_route_inventory --no-fail-fast`
  passed: 3 tests.
- `cargo nextest run -p nako-metadata candidate_review_application -p nako-db metadata_candidate_review -p nako-server metadata_candidate_review --no-fail-fast`
  passed: 18 tests.
- `cargo check -p nako-core -p nako-metadata -p nako-api -p nako-server -p nako-db --tests`
  passed.
- `python ./.trellis/scripts/task.py validate 06-02-03c-provider-governance-audit-public-contract`
  passed.
- `git diff --check` passed.
