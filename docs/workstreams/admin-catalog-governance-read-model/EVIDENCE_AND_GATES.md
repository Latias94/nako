# Evidence And Gates

## Required Gates

- `cargo fmt --all -- --check`
- `cargo check -p nako-core --tests`
- `cargo check -p nako-db --tests`
- `cargo nextest run -p nako-db catalog_governance --no-fail-fast`
- `cargo check -p nako-api --tests`
- `cargo nextest run -p nako-api admin_catalog_governance --no-fail-fast`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server http::tests::system --no-fail-fast`
- `cargo nextest run -p nako-api public_openapi --no-fail-fast`
- `cargo nextest run -p nako-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast`
- `git diff --check`
- `git diff --name-only -- crates/nako-client-protocol`

## Evidence Log

- 2026-05-18: M60 planning opened.
- 2026-05-18: `CatalogGovernanceRepository` added to keep governance SQL
  inside the repository boundary.
- 2026-05-18: `GET /admin/v1/catalog/governance/items` added with redacted
  Admin DTOs and focused route tests.
- 2026-05-18: Focused validation passed for SQLite governance query, Admin DTO
  redaction, Admin route filtering/redaction, system auth coverage, and public
  OpenAPI/SDK exclusion.
