# Addon Resource Search Product Flow - TODO

Status: Closed
Last updated: 2026-05-28

## M0 - Lane Open

- [x] RSPF-010 [owner=Codex] [deps=none] [scope=docs/workstreams/addon-resource-search-product-flow]
  Goal: Open the product-flow lane and freeze boundaries.
  Validation: Workstream docs exist and agree.
  Review: Confirm this lane does not reopen downloader/cloud-drive scope.
  Evidence: DESIGN.md
  Handoff: First executable slice is RSPF-020.

## M1 - Admin Contract For Product Search

- [x] RSPF-020 [owner=Codex] [deps=RSPF-010] [scope=crates/nako-api/src/extension.rs,crates/nako-api/src/admin_contract.rs]
  Goal: Add Admin DTOs and route constants for product resource search and explicit selection.
  Validation: `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  Review: Contract must not expose raw URL/password/context/provider exception fields.
  Evidence: crates/nako-api/src/extension.rs
  Handoff: Complete. Product DTOs expose display-safe results, redacted link summaries, and opaque `search_id`/`selection_id` values.

## M2 - Search Session And Safe Results

- [x] RSPF-030 [owner=Codex] [deps=RSPF-020] [scope=crates/nako-server/src/app/addons.rs]
  Goal: Implement host-owned resource-search execution that returns display-safe result cards and opaque selection IDs.
  Validation: `cargo nextest run -p nako-server addon_resource_search_product --no-fail-fast`
  Review: Use existing typed client helper, apply host limits, keep diagnostic route separate, and store raw links only in a transient host session.
  Evidence: crates/nako-server/src/app/addons.rs
  Handoff: Complete. Product search creates a bounded in-memory host session, returns display-safe result cards, and keeps raw links/passwords out of the browser response.

## M3 - Explicit Selection To Intake

- [x] RSPF-040 [owner=Codex] [deps=RSPF-030] [scope=crates/nako-server/src/app/addons.rs,crates/nako-server/src/app/acquisition_intake.rs]
  Goal: Convert an opaque selected result/link into a `resource_search_selection` intake candidate.
  Validation: `cargo nextest run -p nako-server addon_resource_search_product --no-fail-fast`; `cargo nextest run -p nako-server acquisition_intake --no-fail-fast`
  Review: The selection route must not accept raw link payloads from the browser.
  Evidence: crates/nako-server/src/app/addons.rs
  Handoff: Complete. Selection reads the raw link only from the host session, records a `resource_search_selection` candidate, and reports true idempotent replay.

## M4 - HTTP Routes And Generated Contracts

- [x] RSPF-050 [owner=Codex] [deps=RSPF-020 RSPF-030 RSPF-040] [scope=crates/nako-server/src/http/addons.rs,apps/admin-web/src/adminApi/generated/contract.ts,web/src/api/admin/generated/contract.ts]
  Goal: Expose product search/selection routes and refresh generated Admin TypeScript contracts.
  Validation: `cargo nextest run -p nako-server addon_resource_search_product --no-fail-fast`; `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  Review: Route names should not blur diagnostic and product behavior.
  Evidence: generated Admin API contracts
  Handoff: Complete. HTTP routes now match the generated Admin contract route constants from RSPF-020.

## M5 - Verify And Close

- [x] RSPF-060 [owner=planner] [deps=RSPF-020 RSPF-030 RSPF-040 RSPF-050] [scope=docs/workstreams/addon-resource-search-product-flow]
  Goal: Record final gates, close this Nako API lane, and split Admin UI plus official-addon migration.
  Validation: focused nextest gates; `cargo fmt --all -- --check`; `cargo check -p nako-api -p nako-server --tests`; `git diff --check`
  Review: Run review-workstream and verify-rust-workstream before closeout.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Complete. The Nako API lane is closed; UI, official-addon migration, link-check, downloader, cloud-drive save, and password persistence remain follow-ons.
