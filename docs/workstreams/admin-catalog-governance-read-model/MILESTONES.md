# Milestones

## M60: Admin Catalog Governance Item Queue

Status: completed.

Outcome: The admin console can list unknown and low-confidence Media Items
through a safe Admin API v1 read model.

Deliverables:

- `CatalogGovernanceRepository` and SQLite adapter.
- `GET /admin/v1/catalog/governance/items`.
- Redacted Admin DTOs for governance item rows and Local Inference summaries.
- Focused repository, DTO, HTTP route, auth, and Public Client API boundary
  tests.
- Updated API and admin console planning docs.

Exit criteria:

- Unknown Media Items are listed.
- Non-unknown Media Items with best Local Inference confidence at or below the
  requested threshold are listed.
- High-confidence items are excluded from this queue.
- Rows include source count, representative source identity/file name,
  Local Inference confidence/inferred fields, Provider Mapping counts, and
  duplicate relationship count.
- Responses do not expose source locators, local paths, raw evidence values,
  raw provider responses, or secrets.
- Public Client API, public OpenAPI/SDK, and `nako-client-protocol` stay clean.
