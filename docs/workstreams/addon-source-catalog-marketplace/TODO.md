# Addon Source Catalog And Marketplace - TODO

Status: Completed
Last updated: 2026-05-24

## M0 - Boundary Freeze

- [x] ASCM-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-source-catalog-marketplace,docs/adr,docs/workstreams]
  Goal: Freeze the addon source catalog / marketplace problem, target state,
  non-goals, and first discovery slice without folding package signing or
  process supervision into the lane.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json,
  and HANDOFF.md agree on the chosen slice and split follow-ons.
  Review: Confirm the catalog boundary stays separate from the existing
  manager-plan surface and does not require process/container authority.
  Evidence: docs/workstreams/addon-source-catalog-marketplace/DESIGN.md
  Handoff: DONE. Boundary frozen around read-only source listing, browse
  metadata, and descriptor resolution. Package signing, provider breadth,
  rollback/update execution, task-path smoke, authenticated outbound task
  credentials, and process supervision are follow-ons.

## M1 - Catalog Model

- [x] ASCM-020 [owner=codex] [deps=ASCM-010] [scope=docs,guides,crates/nako-server,crates/nako-api]
  Goal: Define the first addon source catalog model and the minimal discovery /
  resolution surface for a user-visible marketplace entry.
  Validation: Addon source docs and API shapes are explicit enough that the
  first browse/install-candidate slice can be implemented without revisiting
  the boundary split.
  Review: Keep package signing, provider breadth, and process/container
  supervision out of the catalog bootstrap slice.
  Evidence: EVIDENCE_AND_GATES.md.
  Handoff: DONE. Added Admin DTOs for catalog sources, entries, resolution, and
  lifecycle-boundary flags; the model reuses `AddonInstallDescriptor` and
  `AddonInstallGuide`.

## M2 - Discovery Surface

- [x] ASCM-030 [owner=codex] [deps=ASCM-020] [scope=crates/nako-server,crates/nako-api,docs]
  Goal: Implement the smallest catalog-facing discovery surface that can list
  addon sources and resolve installable addon metadata without signing or
  process supervision.
  Validation: focused server tests plus a repeatable smoke or docs workflow for
  listing/resolving one catalog source.
  Review: Confirm the catalog does not absorb package signing, marketplace
  trust roots, or process/container supervision.
  Evidence: EVIDENCE_AND_GATES.md.
  Handoff: DONE. Implemented `GET /admin/v1/addons/catalog/sources`,
  `GET /admin/v1/addons/catalog/entries`, and
  `GET /admin/v1/addons/catalog/entries/{entry_id}/resolve` for the built-in
  official source without registration, routing-plan, job, or process side
  effects.

## M3 - Closeout Or Split

- [x] ASCM-060 [owner=planner] [deps=ASCM-030] [scope=docs/workstreams/addon-source-catalog-marketplace]
  Goal: Close the lane or split marketplace/package-signing/provider-breadth
  and process-supervision follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: DONE. Lane closed after final docs and fresh gates. Remaining addon
  work is split into explicit product/ecosystem follow-ons.
