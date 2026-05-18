# Admin Catalog Governance Read Model

Status: completed for M60

This workstream adds the first code-backed Admin API v1 catalog governance
read model. It focuses on unknown and low-confidence Media Items so the future
admin console can review scanner/local-inference quality before Taru adds
repair mutations.

## Scope

- `GET /admin/v1/catalog/governance/items`.
- Admin-owned redacted DTOs in `taru-api::admin`.
- A focused repository port that keeps local inference, provider mapping, and
  duplicate relationship query shape inside the catalog/database boundary.
- Route, DTO, repository, redaction, auth, and Public Client API boundary
  tests.

## Non-Goals

- No catalog repair mutation.
- No provider rematch mutation.
- No NFO import/export behavior change.
- No Source Variant, Edition, or Duplicate UI workflow.
- No Public Client API, public OpenAPI, SDK, or `taru-client-protocol` change.

## Links

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
