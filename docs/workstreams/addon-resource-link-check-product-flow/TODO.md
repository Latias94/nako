# Addon Resource Link Check Product Flow - TODO

Status: Closed
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

- [x] RLCPF-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-resource-link-check-product-flow]
  Goal: Freeze product route scope, non-goals, and evidence gates.
  Validation: workstream docs exist and agree.
  Evidence: docs/workstreams/addon-resource-link-check-product-flow/DESIGN.md
  Handoff: Planner opened this lane on 2026-05-28.

## M1 - API And Server Flow

- [x] RLCPF-020 [owner=Codex] [deps=RLCPF-010] [scope=crates/nako-api,crates/nako-server]
  Goal: Add opaque selection link-check product route and safe response DTO.
  Validation: cargo nextest run -p nako-server addon_resource_link_check --no-fail-fast
  Review: Confirm request body cannot carry raw URL/password/context.
  Evidence: crates/nako-server/src/app/addons.rs
  Handoff: DONE. Added a host-owned link-check route that retrieves selected
  links by opaque search/selection ids and rejects browser-submitted raw link
  material.

## M2 - Contract And Docs

- [x] RLCPF-030 [owner=Codex] [deps=RLCPF-020] [scope=crates/nako-api,docs/workstreams/addon-resource-link-check-product-flow]
  Goal: Update static admin contract and closeout evidence.
  Validation: cargo nextest run -p nako-api admin_contract --no-fail-fast
  Review: Confirm Admin UI is not changed.
  Evidence: crates/nako-api/src/admin_contract.rs
  Handoff: DONE. Refreshed both generated Admin TypeScript contract copies and
  added safe link-check DTOs to the static contract.

## M3 - Verification And Closeout

- [x] RLCPF-040 [owner=Codex] [deps=RLCPF-030] [scope=workspace]
  Goal: Verify and close the lane.
  Validation: cargo nextest run -p nako-server addon_resource_link_check --no-fail-fast; cargo nextest run -p nako-api admin_contract --no-fail-fast; cargo fmt --all -- --check; cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-server --tests
  Review: Record residual risks and follow-ons.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: DONE. Targeted server/API tests, format check, and focused cargo
  check passed; product UI, checker providers, downloader execution, and
  durable password/code handling remain split follow-ons.
