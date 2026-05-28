# Addon Resource Link Check Contract - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

- [x] RLC-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-resource-link-check-contract]
  Goal: Freeze the contract target state, non-goals, and validation gates.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json exist and agree.
  Evidence: docs/workstreams/addon-resource-link-check-contract/DESIGN.md
  Handoff: Planner opened this lane on 2026-05-28.

## M1 - Protocol Vocabulary And DTOs

- [x] RLC-020 [owner=Codex] [deps=RLC-010] [scope=crates/nako-addon-protocol]
  Goal: Add first-class link-check resource, scope, schema constants, request/response DTOs, status vocabulary, and protocol tests.
  Validation: cargo nextest run -p nako-addon-protocol resource_link_check --no-fail-fast
  Review: Confirm response DTOs do not expose raw URL/password fields.
  Evidence: crates/nako-addon-protocol/src/lib.rs
  Handoff: DONE. Protocol vocabulary, schemas, DTOs, and redaction tests are in place.

## M2 - Client Helper

- [x] RLC-030 [owner=Codex] [deps=RLC-020] [scope=crates/nako-addon-client]
  Goal: Add typed `call_addon_resource_link_check` helper with manifest/scope/schema validation.
  Validation: cargo nextest run -p nako-addon-client resource_link_check --no-fail-fast
  Review: Confirm helper rejects missing scope and wrong schemas before callers consume payloads.
  Evidence: crates/nako-addon-client/src/lib.rs
  Handoff: DONE. Product/server route integration remains a later lane.

## M3 - Verification And Closeout

- [x] RLC-040 [owner=Codex] [deps=RLC-030] [scope=workspace]
  Goal: Verify and close the contract slice.
  Validation: cargo nextest run -p nako-addon-protocol -p nako-addon-client resource_link_check --no-fail-fast; cargo fmt --all -- --check; cargo check -p nako-addon-protocol -p nako-addon-client --tests
  Review: Record residual risks and follow-ons.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: DONE. Open server/product integration only after this protocol contract is stable.
