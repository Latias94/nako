# Addon Architecture Deepening — Milestones

Status: Completed
Last updated: 2026-05-21

## M0 — Authority Freeze

Exit criteria:

- Stale Addon ADR statuses or notes reflect shipped reality.
- This workstream is indexed and linked from goal/workstream trackers.
- Deferred Addon Manager, Addon Task runtime, and Addon Event Subscription
  runtime breadth remain explicit non-goals unless promoted by a later task.

## M1 — Addon Side Effect Runtime Depth

Status: Complete. AAD-020 added the runtime boundary; AAD-030 added
fingerprinted idempotency.

Exit criteria:

- Addon Side Effect lifecycle has one deep runtime Interface.
- Per-permission Adapters return a common apply result shape.
- Validation failure, apply failure, replay, and redaction behavior are tested
  through the runtime or route Interface.
- Same-key different-request idempotency returns conflict.

## M2 — Protected Write Payload Contracts

Status: Complete. AAD-040 moved shipped Protected Write payload shapes to
`nako-addon-protocol` Addon-facing DTOs and updated server/docs/reference Addon
usage.

Exit criteria:

- Canonical Metadata patch, Addon Artwork Candidate proposal, and Library File
  Write command payloads are explicit Interfaces.
- Addon author docs no longer require reading private server structs to know
  shipped payload shapes.
- Server Adapters parse typed payloads rather than ad hoc JSON contracts.

## M3 — Addon Manifest And Protocol Depth

Status: Complete. AAD-050 added compatible manifest declarations and
registration validation for Addon Entry Points, Addon Hosted Pages, Addon
Configuration Schema, Secret Reference fields, Addon Event Subscriptions, and
Addon Tasks.

Exit criteria:

- Addon Manifest validation is deepened through a validated manifest Module or
  equivalent plan object.
- Addon Entry Point, Addon Hosted Page, Addon Configuration Schema, Secret
  Reference, Addon Event Subscription, and Addon Task concepts are represented
  as manifest declarations or explicitly deferred with rationale.
- Compatible additions preserve current Addon Protocol Version rules.

## M4 — Library File Write Runtime

Status: Complete. AAD-060 introduced an Addon Library File Write runtime seam
for the shipped MediaSource-targeted NFO Export path, keeping NFO Export as the
first Adapter and preserving deferred subtitle/sidecar breadth.

Exit criteria:

- NFO Export runs behind a Library File Write runtime seam.
- Target derivation, writability, policy, backup, and redacted reports are
  owned by the runtime seam.
- No Addon-facing response leaks Source Locator, storage URI, local path,
  remote handle, backup URI, or raw file content.

## M5 — Admin Addon API

Status: Complete. AAD-070 migrated Addon registration, list, and detail
management to `/admin/v1/addons`; removed the old root `/addons` management
surface; and introduced Admin summary/detail DTOs that no longer expose raw
registration persistence records or `manifest_json`.

Exit criteria:

- Addon administration has `/admin/v1/addons` route coverage.
- Admin Addon DTOs shield persistence records.
- Public Client API, public OpenAPI, and SDK boundaries remain free of admin
  Addon surfaces.

## M6 — Protocol Boundary And Persistence Parity

Status: Complete. AAD-080 split the HTTP caller helper into the permissive
`nako-addon-client` crate, leaving `nako-addon-protocol` as a dependency-light
wire-contract and validation crate. AAD-090 verified the touched Addon
persistence slice and made request fingerprints part of the clean SQLite and
PostgreSQL base schemas instead of carrying a compatibility migration/fallback.

Exit criteria:

- `nako-addon-protocol` dependency and license boundary is re-audited after
  manifest/payload changes.
- SQLite and PostgreSQL Addon state semantics are aligned for all touched
  behavior.
- PostgreSQL opt-in contracts are present and run when a test URL is available.

## M7 — Closeout

Status: Complete. AAD-100 reviewed the lane, found no required split, and
closed with fresh formatting, workspace check, focused Addon nextest, full
workspace nextest, and diff evidence. PostgreSQL opt-in contracts were skipped
because `NAKO_TEST_POSTGRES_URL` was not set.

Exit criteria:

- AAD-010 through AAD-100 are complete or split into named follow-ons.
- `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, and `WORKSTREAM.json` reflect current
  state.
- Final formatting, workspace check, focused nextest, workspace nextest when
  practical, and diff gates have fresh evidence.
