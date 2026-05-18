# Public Client Source Locator Redaction

Status: Completed
Last updated: 2026-05-18

This workstream owns ARF-005 from the 2026-05-18 architecture review: Public
Client DTOs currently expose raw Media Source locators even though locators can
encode local or remote storage details. The lane is a narrow public contract
follow-up under the existing Public Client API, Public API Contract, and
OpenAPI authority.

The lane is now complete. Public Client DTOs, OpenAPI schema, generated SDK
output, route tests, and HTTP API docs all reflect the redacted contract.

Authoritative docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
