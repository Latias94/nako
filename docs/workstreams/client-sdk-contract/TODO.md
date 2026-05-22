# Client SDK Contract Inventory And Streaming Builders TODO

Status: Completed
Last updated: 2026-05-17

## M36.0 Scope And License Baseline

- [x] CSC-010 [owner=planner] [deps=none] [scope=docs/workstreams/client-sdk-contract]
  Goal: Freeze the M36 scope, route inventory ownership, streaming-builder
  non-goals, and ADR 0022 license boundary.
  Validation: workstream docs exist and agree.
  Evidence: this workstream.
  Handoff: Continue with CSC-020 before moving SDK helpers into AGPL crates.

## M36.1 Shared Public Route Inventory

- [x] CSC-020 [owner=codex] [deps=CSC-010] [scope=crates/nako-client-protocol, crates/nako-api]
  Goal: Move public client route inventory into `nako-client-protocol` and
  refactor `nako-api` OpenAPI/tests plus TypeScript generator to consume it.
  Validation: `cargo check -p nako-client-protocol --tests`,
  `cargo nextest run -p nako-client-protocol --no-fail-fast`,
  `cargo nextest run -p nako-api --no-fail-fast`.
  Evidence: protocol inventory types/constants and nako-api usage.
  Handoff: Keep OpenAPI rendering in `nako-api`; only neutral route facts move.

## M36.2 Rust SDK Inventory And Streaming Builders

- [x] CSC-030 [owner=codex] [deps=CSC-020] [scope=crates/nako-client]
  Goal: Make `nako-client` consume the shared inventory and add request
  builders for direct stream, HEAD preflight, remux stream, HLS playlist, and
  HLS segment routes.
  Validation: `cargo check -p nako-client --tests`,
  `cargo nextest run -p nako-client --no-fail-fast`.
  Evidence: builder tests for method, path, query, auth, range, and path
  encoding behavior.
  Handoff: Do not execute streaming bodies or add download/player abstractions.

## M36.3 Cross-SDK Drift And Leakage Gates

- [x] CSC-040 [owner=codex] [deps=CSC-030] [scope=crates/nako-api, crates/nako-client, sdk/typescript]
  Goal: Prove OpenAPI, TypeScript SDK, and Rust SDK agree on the shared route
  inventory and still reject admin/internal/secret/local-path surfaces.
  Validation: `cargo nextest run -p nako-api --no-fail-fast`,
  `cargo nextest run -p nako-client --no-fail-fast`,
  `npm run check --prefix sdk/typescript`.
  Evidence: route inventory tests and generated TypeScript package sync.
  Handoff: If TypeScript generated output changes, refresh package source.

## M36.4 Docs And Closeout

- [x] CSC-050 [owner=planner] [deps=CSC-040] [scope=docs/api/HTTP_API.md, docs/GOALS.md, docs/ROADMAP.md, docs/workstreams/client-sdk-contract]
  Goal: Document the shared inventory boundary, streaming builder behavior,
  validation commands, and follow-ons.
  Validation: `cargo fmt --all -- --check`, `cargo check --workspace --tests`,
  `cargo nextest run --workspace --no-fail-fast`, `cargo tree -p
  nako-client-protocol`, `cargo tree -p nako-client`, `git diff --check`.
  Evidence: EVIDENCE_AND_GATES.md and closeout audit.
  Handoff: Record follow-ons for full streaming bodies, package publishing,
  Rust CLI, Dart/Flutter SDK, and concrete clients.
