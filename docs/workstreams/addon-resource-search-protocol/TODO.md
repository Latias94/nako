# Addon Resource Search Protocol - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

- [x] ARSP-010 [owner=Codex] [deps=none] [scope=docs/workstreams/addon-resource-search-protocol]
  Goal: Freeze resource-search host protocol intent, authority, non-goals, and validation plan.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: docs/workstreams/addon-resource-search-protocol/DESIGN.md
  Handoff: First implementation slice is ARSP-020.

## M1 - Protocol Vocabulary And DTOs

- [x] ARSP-020 [owner=Codex] [deps=ARSP-010] [scope=crates/nako-addon-protocol]
  Goal: Add `resource_search` resource, `acquisition_search_read` scope, typed request/response DTOs, link taxonomy, provider execution status, and finality contracts.
  Validation: `cargo nextest run -p nako-addon-protocol resource_search --no-fail-fast`
  Review: Check wire names, serde shapes, manifest validation, response validation, and redaction-safe debug output.
  Evidence: crates/nako-addon-protocol/src/lib.rs
  Handoff: Complete. Downloader, link-check, and candidate-write behavior remain out of scope.

## M2 - Typed Client Call

- [x] ARSP-030 [owner=Codex] [deps=ARSP-020] [scope=crates/nako-addon-client]
  Goal: Add a typed resource-search client helper over the existing generic addon resource call machinery.
  Validation: `cargo nextest run -p nako-addon-client resource_search --no-fail-fast`
  Review: Ensure timeout, retry, protocol-version, scope-grant, and safe error behavior remain inherited from generic resource calls.
  Evidence: crates/nako-addon-client/src/lib.rs
  Handoff: Complete. The helper stays on the generic resource-call path and enforces the read-scope/schema contract.

## M3 - Host Call Boundary

- [x] ARSP-040 [owner=Codex] [deps=ARSP-030] [scope=crates/nako-server/src/app/addons.rs,crates/nako-api/src/extension.rs]
  Goal: Define the host service/admin diagnostic seam for calling a resource-search addon with explicit limits, granted scope, and redaction-safe diagnostics.
  Validation: `cargo nextest run -p nako-server addon_resource_search --no-fail-fast`
  Review: Keep API/admin DTOs shielded from raw provider payloads and addon exception text.
  Evidence: crates/nako-server/src/app/addons.rs
  Handoff: Complete. The admin diagnostic seam returns safe counts/provider summaries only; acquisition conversion remains separate.

## M4 - Acquisition Handoff

- [ ] ARSP-050 [owner=unassigned] [deps=ARSP-040] [scope=crates/nako-core/src/acquisition_intake.rs,crates/nako-server/src/app]
  Goal: Record or implement the host-owned conversion from a selected resource-search result to an acquisition intake candidate.
  Validation: `cargo nextest run -p nako-server acquisition_intake addon_resource_search --no-fail-fast`
  Review: Search results remain candidates only; downloader execution and cloud-drive save are separate scopes and lanes.
  Evidence: crates/nako-core/src/acquisition_intake.rs
  Handoff: Split downloader hooks and link checking to follow-ons.

## M5 - Docs, Migration, And Closeout

- [ ] ARSP-060 [owner=planner] [deps=ARSP-020 ARSP-030 ARSP-040 ARSP-050] [scope=docs,crates/nako-addon-protocol]
  Goal: Update docs, record final gates, and explicitly hand off `nako-official-addons` manifest migration.
  Validation: `cargo fmt --all -- --check`; focused nextest gates from completed tasks; `git diff --check`
  Review: Run review-workstream and verify-rust-workstream before closeout.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Close or split remaining server/UI/official-addon work.
