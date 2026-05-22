# OpenAPI And Public Client SDK Contract TODO

Status: Completed
Last updated: 2026-05-17

## M32.0 Scope And Boundary Baseline

- [x] OAS-010 [owner=planner] [deps=none] [scope=docs/adr, docs/workstreams/openapi-client-contract]
  Goal: Freeze the OpenAPI/Public Client SDK contract boundary, route scope, source-of-truth policy, non-goals, and gate set.
  Validation: ADR 0025, DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: `docs/adr/0025-openapi-public-client-sdk-contract.md` and this workstream.
  Handoff: Continue with OAS-020 before adding generated SDK packaging.

## M32.1 Protocol Response Hygiene Slice

- [x] OAS-020 [owner=codex] [deps=OAS-010] [scope=crates/nako-client-protocol, crates/nako-api, crates/nako-server/src/http/playback.rs]
  Goal: Move public playback session response shape to `nako-client-protocol` and remove local output path leakage from the client contract.
  Validation: `cargo fmt --all -- --check`, `cargo check --workspace --tests`, `cargo nextest run -p nako-client-protocol --no-fail-fast`, `cargo nextest run -p nako-api --no-fail-fast`, `cargo nextest run -p nako-server http::tests::playback --no-fail-fast`.
  Evidence: protocol-owned `TranscodeSessionResponse`; `nako-api::transcode_session_response_from_record`; tests assert no `output_path`.
  Handoff: Do not implement user sessions or RBAC here.

## M32.2 OpenAPI Artifact Slice

- [x] OAS-030 [owner=codex] [deps=OAS-020] [scope=crates/nako-api, docs/api]
  Goal: Implement the first Public Client API OpenAPI v1 artifact/generator covering health, library, catalog/search, probe, playback, session, auth, version, pagination, and error envelopes.
  Validation: `cargo nextest run -p nako-api --no-fail-fast`.
  Evidence: `nako_api::openapi::public_openapi_v1_json()` and `cargo run -p nako-api --example emit-openapi`.
  Handoff: Keep admin/internal routes out of the first public spec.

## M32.3 Server Route Contract Evidence Slice

- [x] OAS-040 [owner=codex] [deps=OAS-030] [scope=crates/nako-server/src/http/tests, docs/api/HTTP_API.md]
  Goal: Ensure the OpenAPI public route inventory matches current HTTP behavior and docs.
  Validation: `cargo nextest run -p nako-server http::tests --no-fail-fast`.
  Evidence: route-level tests, new `GET /libraries/{library_id}` behavior, and HTTP API documentation updates.
  Handoff: Split OpenAPI route serving, SDK generation, and admin API spec into follow-ons if still needed.

## M32.4 Closeout

- [x] OAS-050 [owner=planner] [deps=OAS-040] [scope=docs/workstreams/openapi-client-contract]
  Goal: Close M32 with prompt-to-artifact audit against every explicit requirement.
  Validation: `cargo fmt --all -- --check`, `cargo check --workspace --tests`, `cargo nextest run --workspace --no-fail-fast`, `cargo tree -p nako-client-protocol`, OpenAPI checker, `git diff --check`.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, and closeout gate output.
  Handoff: Record follow-ons for SDK generation, OpenAPI route serving, admin API contract, auth/session/RBAC, and future client app work.
