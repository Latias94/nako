# Public Client Source Locator Redaction TODO

Status: Completed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

- [x] PCLR-010 [owner=codex] [deps=none] [scope=docs/workstreams/public-client-source-locator-redaction]
  Goal: Open the lane with problem, target state, non-goals, gates, and first
  executable audit task.
  Validation: `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`.
  Handoff: Continue with PCLR-020 before changing public DTOs.

## M1 - Exposure Audit And Contract Decision

- [x] PCLR-020 [owner=codex] [deps=PCLR-010] [scope=crates/taru-client-protocol,crates/taru-api,crates/taru-server/src/http,docs/api]
  Goal: Classify each raw locator exposure as Public Client, Admin API,
  internal execution, or test fixture, then choose the public replacement or
  redaction policy.
  Validation: `rg "locator|input_locator" crates/taru-client-protocol
  crates/taru-api crates/taru-server/src/http`; `git diff --check`.
  Review: contract decision recorded; remove fields in PCLR-030.
  Evidence: audit notes in `EVIDENCE_AND_GATES.md`.
  Handoff: Public Client DTOs must remove raw source locator fields. Internal
  storage/playback execution keeps full locators. Admin-only diagnostics may
  add redacted locator summaries in separate Admin API work.

## M2 - Public DTO And Server Mapping

- [x] PCLR-030 [owner=codex] [deps=PCLR-020] [scope=crates/taru-client-protocol,crates/taru-api,crates/taru-server/src/http/tests]
  Goal: Remove, replace, or redact public locator fields in protocol DTOs and
  server mapping while preserving internal storage/playback locators.
  Validation: `cargo check -p taru-client-protocol --tests`; `cargo check -p
  taru-api --tests`; focused `cargo nextest run -p taru-server
  <public-route-filter> --no-fail-fast`.
  Review: no blocking public contract or leakage findings in self-review.
  Evidence: route tests prove public JSON omits raw locators.
  Handoff: OpenAPI/SDK sync was completed alongside DTO removal because
  `taru-api` validates generated client consistency.

## M3 - OpenAPI, SDK, And Docs Sync

- [x] PCLR-040 [owner=codex] [deps=PCLR-030] [scope=crates/taru-api,sdk,docs/api]
  Goal: Update OpenAPI, SDK inventory/generation checks, and HTTP API docs to
  match the redacted Public Client contract.
  Validation: existing OpenAPI/SDK checks from client contract lanes; `cargo
  fmt --all -- --check`; `git diff --check`.
  Review: no blocking public contract or leakage findings in self-review.
  Evidence: `EVIDENCE_AND_GATES.md`, OpenAPI/SDK diffs, HTTP API docs.
  Handoff: Continue with PCLR-050 closeout.

## M4 - Closeout

- [x] PCLR-050 [owner=planner] [deps=PCLR-040] [scope=docs/workstreams/public-client-source-locator-redaction]
  Goal: Close the lane or split narrower follow-ons.
  Validation: `verify-rust-workstream` records fresh final gate evidence.
  Review: no blocking review findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Lane is closed; public locator exposure should not re-enter future
  DTOs without a contract decision.
