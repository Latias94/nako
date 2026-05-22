# Public API Contract Hardening TODO

Status: Completed
Last updated: 2026-05-17

## M30.0 Scope And Contract Baseline

- [x] PAC-010 [owner=planner] [deps=none] [scope=docs/adr, docs/workstreams/public-api-contract]
  Goal: Freeze the public API versioning/error-envelope problem, target state, public/internal route boundary, and validation gates.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md, and ADR 0023 exist and agree.
  Evidence: docs/workstreams/public-api-contract/DESIGN.md and docs/adr/0023-public-api-versioning-and-error-envelope-contract.md.
  Handoff: Continue with PAC-020 before changing server error behavior.

## M30.1 Protocol Error Vocabulary Slice

- [x] PAC-020 [owner=codex] [deps=PAC-010] [scope=crates/nako-client-protocol, crates/nako-api]
  Goal: Move the stable public error-code vocabulary into `nako-client-protocol` while preserving the existing `code/message` JSON shape.
  Validation: cargo fmt --all -- --check, cargo check -p nako-client-protocol --tests, cargo nextest run -p nako-client-protocol --no-fail-fast, cargo tree -p nako-client-protocol.
  Evidence: Protocol crate exposes stable public error codes without importing server/internal crates.
  Handoff: Completed. Route path versioning and OpenAPI generation remain follow-ons.

## M30.2 Server Error Mapping And Version Identity Slice

- [x] PAC-030 [owner=codex] [deps=PAC-020] [scope=crates/nako-server/src/http/{error,system,query}.rs, crates/nako-server/src/http/tests]
  Goal: Make server HTTP error mapping and API version identity auditable against the protocol-owned public vocabulary.
  Validation: cargo check -p nako-server --tests, cargo nextest run -p nako-server http::tests::system --no-fail-fast, cargo nextest run -p nako-server http::tests::playback --no-fail-fast.
  Evidence: Tests cover status/code/message behavior for public error categories and `/health` version identity.
  Handoff: Completed. Server-admin/internal routes reuse the baseline envelope but remain outside the first public compatibility promise.

## M30.3 Public Route Contract Evidence Slice

- [x] PAC-040 [owner=codex] [deps=PAC-030] [scope=docs/api/HTTP_API.md, crates/nako-server/src/http/tests/{catalog,library,playback,system}.rs]
  Goal: Prove catalog/library/playback/system public routes return stable success envelopes, pagination metadata, and error envelopes.
  Validation: cargo nextest run -p nako-server http::tests --no-fail-fast.
  Evidence: Route-level tests and HTTP API docs map public route surfaces to stable protocol behavior.
  Handoff: Completed. OpenAPI, SDK generation, auth, and multi-version route negotiation are follow-ons.

## M30.4 Closeout

- [x] PAC-050 [owner=planner] [deps=PAC-040] [scope=docs/workstreams/public-api-contract]
  Goal: Close M30 with a prompt-to-artifact audit against every explicit goal requirement.
  Validation: cargo fmt --all -- --check, cargo check --workspace --tests, cargo nextest run --workspace --no-fail-fast, cargo tree -p nako-client-protocol, git diff --check.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Handoff: Completed. Remaining API versioning, SDK generation, and broader admin/internal migration work is split out.
