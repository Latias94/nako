# Addon Resource Search Protocol - Milestones

Status: Active
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

Exit criteria:

- Workstream documents exist and agree on scope, non-goals, authorities, and
  task order.
- The official addon protocol proposal is referenced as input, not as the
  authoritative Nako contract.

## M1 - Protocol Vocabulary And DTOs

Exit criteria:

- `AddonResource::ResourceSearch` serializes as `resource_search`.
- `AddonScope::AcquisitionSearchRead` serializes as
  `acquisition_search_read`.
- Resource-search DTOs round-trip through serde with stable snake_case wire
  names.
- Manifest validation accepts a resource-search declaration only when required
  scopes are declared by the manifest.

## M2 - Typed Client Call

Exit criteria:

- `nako-addon-client` can call a resource-search addon through the existing
  resource-call path.
- Scope denial, missing resource declaration, protocol mismatch, retry, and
  safe error behavior remain inherited from existing machinery.

## M3 - Host Call Boundary

Exit criteria:

- Nako server has a clear app/admin seam for resource-search addon calls.
- Host policy owns limit, granted scope, timeout, retry, and diagnostics.
- Raw provider errors and tokens are not exposed through diagnostics.

## M4 - Acquisition Handoff

Exit criteria:

- Search result selection is clearly separated from downloader execution.
- Conversion to acquisition intake is host-owned, audited, and explicit through
  `resource_search_selection` candidates.
- Link-check/downloader/cloud-drive-save scopes are split to named follow-on
  workstreams.

## M5 - Closeout

Exit criteria:

- Fresh focused gates are recorded in `EVIDENCE_AND_GATES.md`.
- `WORKSTREAM.json` and `HANDOFF.md` reflect final status.
- Follow-on work for `nako-official-addons` migration is explicit.
