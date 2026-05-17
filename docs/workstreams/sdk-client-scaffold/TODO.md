# SDK Generation And Client Integration Scaffold TODO

Status: Completed
Last updated: 2026-05-17

## M33.0 Scope And Boundary Baseline

- [x] SDK-010 [owner=planner] [deps=none] [scope=docs/workstreams/sdk-client-scaffold]
  Goal: Freeze M33 SDK generation boundary, first target language, generated-output policy, non-goals, and validation gates.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: this workstream.
  Handoff: Continue with SDK-020 before adding Dart/Flutter.

## M33.1 TypeScript SDK Scaffold Generator

- [x] SDK-020 [owner=codex] [deps=SDK-010] [scope=crates/taru-api/src/sdk.rs, crates/taru-api/examples]
  Goal: Add a TypeScript SDK scaffold generator backed by the M32 OpenAPI document.
  Validation: `cargo nextest run -p taru-api --no-fail-fast`, `cargo check -p taru-api --examples`.
  Evidence: `cargo run -p taru-api --example emit-typescript-sdk` emits TypeScript client code.
  Handoff: Keep output dependency-free and do not add Node/Dart packages.

## M33.2 SDK Contract Smoke Checks

- [x] SDK-030 [owner=codex] [deps=SDK-020] [scope=crates/taru-api/src/sdk.rs]
  Goal: Prove the SDK scaffold covers public route inventory, auth/version/error/pagination behavior, and excludes admin/internal routes.
  Validation: `cargo nextest run -p taru-api --no-fail-fast`.
  Evidence: SDK generator tests cover route methods, auth/version/error/pagination runtime details, and forbidden admin/internal terms.
  Handoff: Split full TypeScript compile/package testing into a follow-on if no local toolchain exists.

## M33.3 Docs And Closeout

- [x] SDK-040 [owner=planner] [deps=SDK-030] [scope=docs/api/HTTP_API.md, docs/GOALS.md, docs/ROADMAP.md, docs/workstreams/sdk-client-scaffold]
  Goal: Document SDK generation/checking commands and close M33 with a prompt-to-artifact audit.
  Validation: `cargo fmt --all -- --check`, `cargo check --workspace --tests`, `cargo nextest run --workspace --no-fail-fast`, `cargo tree -p taru-client-protocol`, `git diff --check`.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Handoff: Record follow-ons for TypeScript package publishing, Dart/Flutter SDK, OpenAPI route serving, and concrete client apps.
