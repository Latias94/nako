# Subtitle Import Plan Preview TODO

Status: Complete
Last updated: 2026-05-28

## M0 - Lane Setup

- [x] SIPP-010 [owner=codex] [deps=none] [scope=docs/workstreams/subtitle-import-plan-preview]
  Goal: Open the import-plan preview workstream and lock the non-write
  boundary.
  Validation: `git diff --check`.
  Review: Design must require host-owned selected refs and reject direct paths,
  provider payloads, and writes.
  Evidence: `DESIGN.md`.
  Handoff: DONE 2026-05-28. Continue with SIPP-020.

## M1 - Plan DTO And API Contract

- [x] SIPP-020 [owner=codex] [deps=SIPP-010] [scope=crates/nako-api]
  Goal: Add Admin subtitle import-plan request/response DTOs and generated
  TypeScript contract entries.
  Validation: `cargo nextest run -p nako-api subtitle_import_plan --no-fail-fast`;
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`.
  Review: DTOs must not include raw paths, Source Locators, provider URLs,
  subtitle text, artifact ids, or backup URIs.
  Evidence: `crates/nako-api/src/extension.rs`;
  `crates/nako-api/src/admin_contract.rs`.
  Handoff: DONE 2026-05-28. Added safe import-plan request/response DTOs and
  generated Admin TypeScript contract entries.

## M2 - Host Preview Endpoint

- [x] SIPP-030 [owner=codex] [deps=SIPP-020] [scope=crates/nako-server]
  Goal: Implement selected-reference import-plan preview using host-owned
  candidate session data and media source validation.
  Validation: `cargo nextest run -p nako-server addon_subtitle_import_plan --no-fail-fast`;
  `cargo check -p nako-api -p nako-server --tests`.
  Review: Endpoint must reject raw browser payload fields and media
  item/source mismatch, and must not write files.
  Evidence: `crates/nako-server/src/app/addons.rs`;
  `crates/nako-server/src/http/addons.rs`;
  `crates/nako-server/src/http/tests/addons.rs`.
  Handoff: DONE 2026-05-28. Implemented selected-reference import-plan preview
  with media item/source validation and redaction-safe output.

## M3 - Closeout

- [x] SIPP-040 [owner=codex] [deps=SIPP-020,SIPP-030] [scope=docs/workstreams/subtitle-import-plan-preview]
  Goal: Run final gates, update evidence, and close or split remaining work.
  Validation: final gate set in `EVIDENCE_AND_GATES.md`.
  Review: Remaining Library File Write apply and refresh work must be split.
  Evidence: `EVIDENCE_AND_GATES.md`; `HANDOFF.md`.
  Handoff: DONE 2026-05-28. Final gates passed; Library File Write apply and
  subtitle fact refresh remain split follow-ons.
