# Handoff

Status: completed for M60.

Current state:

- `CatalogGovernanceRepository` lists unknown and low-confidence Media Items.
- `GET /admin/v1/catalog/governance/items` returns redacted Admin DTO rows.
- The route accepts optional `library_id`, `max_confidence_milli`, `limit`, and
  `offset`.
- The DTO omits source locator and raw Local Inference `evidence_value`.

Next recommended follow-ups after M60:

- Add a Provider Mapping list/detail Admin read model.
- Add a duplicate Source review queue.
- Add NFO sidecar status read model.
- Only then consider repair/rematch mutations.
