# TODO

## ACG-010: Open M60 Planning Docs

Status: completed.

Goal: Define the M60 catalog governance read-model boundary, non-goals,
redaction policy, and validation gates.

Scope:

- `docs/GOALS.md`
- `docs/workstreams/admin-catalog-governance-read-model/*`
- `docs/workstreams/README.md`

Validation:

- Documentation names the route, redaction rules, and Public Client API
  boundary.

Handoff: Completed in this workstream.

## ACG-020: Add Catalog Governance Repository Port

Status: completed.

Goal: Keep the governance query shape inside a narrow repository port rather
than exposing SQLite joins to HTTP handlers.

Scope:

- `crates/nako-core/src/repository/catalog_governance.rs`
- `crates/nako-db/src/catalog_governance.rs`
- focused SQLite tests

Validation:

- SQLite test proves unknown and low-confidence Media Items are listed while a
  high-confidence item is excluded.

Handoff: Completed in this workstream.

## ACG-030: Add Redacted Admin DTOs And Route

Status: completed.

Goal: Expose the read model through Admin API v1 without leaking raw evidence,
source locators, local paths, secrets, or Public Client API artifacts.

Scope:

- `crates/nako-api/src/admin.rs`
- `crates/nako-server/src/app/catalog.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/query.rs`
- route tests

Validation:

- Admin route test covers filtering, redaction, and response shape.
- Auth test covers route protection.
- Public OpenAPI/SDK exclusion checks remain green.

Handoff: Completed in this workstream.

## ACG-040: Update API And Console Planning Docs

Status: completed.

Goal: Mark the catalog governance queue as live API-backed data and leave
repair/rematch workflows as planned follow-ups.

Scope:

- `docs/api/HTTP_API.md`
- `docs/workstreams/admin-web-console/ADMIN_API_MATRIX.md`
- `docs/workstreams/admin-web-console/V0_CONTEXT.md`

Validation:

- Docs do not imply mutation support.

Handoff: Completed in this workstream.
