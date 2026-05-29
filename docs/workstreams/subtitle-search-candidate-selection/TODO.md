# Subtitle Search Candidate Selection TODO

Status: Complete
Last updated: 2026-05-28

## M0 - Lane Setup

- [x] SSCS-010 [owner=codex] [deps=none] [scope=docs/workstreams/subtitle-search-candidate-selection]
  Goal: Open the follow-on workstream from `subtitle-complete-chain` and lock
  the non-write boundary.
  Validation: `git diff --check` on workstream docs.
  Review: Design must explicitly stop before import planning and Library File
  Write apply.
  Evidence: `DESIGN.md`; `WORKSTREAM.json`.
  Handoff: DONE 2026-05-28. Continue with SSCS-020.

## M1 - Typed Client Boundary

- [x] SSCS-020 [owner=codex] [deps=SSCS-010] [scope=crates/nako-addon-client]
  Goal: Add a typed `AddonResource::Subtitle` client helper with schema and
  `subtitle_read` scope checks.
  Validation: `cargo nextest run -p nako-addon-client subtitle --no-fail-fast`;
  `cargo check -p nako-addon-client --tests`.
  Review: Helper must reject wrong request/response schemas and missing grants
  before exposing provider payloads to app code.
  Evidence: `crates/nako-addon-client/src/lib.rs`.
  Handoff: DONE 2026-05-28. Added typed subtitle search helpers and coverage
  for schema, grant, and payload failures.

## M2 - Host Search And Selection API

- [x] SSCS-030 [owner=codex] [deps=SSCS-020] [scope=crates/nako-api,crates/nako-server]
  Goal: Add Admin/App subtitle search and selected-reference endpoints that
  return redaction-safe candidate cards and record opaque candidate refs.
  Validation: `cargo nextest run -p nako-api subtitle --no-fail-fast`;
  `cargo nextest run -p nako-server addon_subtitle --no-fail-fast`;
  `cargo check -p nako-api -p nako-server --tests`.
  Review: Responses must not include subtitle text, download URLs, provider
  tokens, Source Locators, local paths, or file write targets.
  Evidence: `crates/nako-api/src/extension.rs`;
  `crates/nako-server/src/app/addons.rs`;
  `crates/nako-server/src/http/addons.rs`;
  `crates/nako-server/src/http/tests/addons.rs`.
  Handoff: DONE 2026-05-28. Added safe subtitle search cards, short-lived
  host session storage, and selected-reference API.

## M3 - Contract And Closeout

- [x] SSCS-040 [owner=codex] [deps=SSCS-020,SSCS-030] [scope=crates/nako-api,docs/workstreams/subtitle-search-candidate-selection]
  Goal: Regenerate/update Admin TypeScript route/type contract and record final
  evidence.
  Validation: `cargo fmt --all -- --check`; path-scoped `git diff --check`;
  final focused test commands from M1 and M2.
  Review: Workstream evidence must list any skipped broad gates and why.
  Evidence: `crates/nako-api/src/admin_contract.rs`;
  `EVIDENCE_AND_GATES.md`; `HANDOFF.md`.
  Handoff: DONE 2026-05-28. Contract generated for both Admin web surfaces and
  final gates passed.
