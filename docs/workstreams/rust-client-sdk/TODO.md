# Rust Client SDK Foundation TODO

Status: Completed
Last updated: 2026-05-17

## M35.0 Scope And Boundary Baseline

- [x] RCS-010 [owner=planner] [deps=none] [scope=docs/workstreams/rust-client-sdk]
  Goal: Freeze the Rust SDK crate boundary, license policy, dependency
  direction, route coverage, non-goals, and validation gates.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json,
  and HANDOFF.md exist and agree.
  Evidence: this workstream.
  Handoff: Continue with RCS-020 before adding broad route coverage or
  streaming abstractions.

## M35.1 Crate Skeleton And Boundary Guard

- [x] RCS-020 [owner=codex] [deps=RCS-010] [scope=crates/taru-client]
  Goal: Add `crates/taru-client` as an Apache-2.0 SDK crate that depends on
  `taru-client-protocol` and runtime HTTP crates, not server/internal crates.
  Validation: `cargo check -p taru-client --tests`, `cargo tree -p taru-client`.
  Evidence: Cargo.toml license/dependency graph and initial tests.
  Handoff: Keep OpenAPI inventory validation out of `taru-client` dependencies
  unless the inventory is moved to a public crate.

## M35.2 Async JSON Client Surface

- [x] RCS-030 [owner=codex] [deps=RCS-020] [scope=crates/taru-client/src/lib.rs]
  Goal: Implement `TaruClient`, config, transport, errors, pagination helpers,
  playback capability helpers, and core JSON route methods.
  Validation: `cargo nextest run -p taru-client --no-fail-fast`.
  Evidence: mock transport tests for auth, version, errors, pagination, and
  route paths.
  Handoff: Streaming/raw body methods can remain deferred with explicit docs.

## M35.3 Public Inventory And Leakage Checks

- [x] RCS-040 [owner=codex] [deps=RCS-030] [scope=crates/taru-client/src/lib.rs]
  Goal: Verify SDK route coverage against the public route inventory and reject
  admin/internal/secret/local-path terms.
  Validation: `cargo nextest run -p taru-client --no-fail-fast`.
  Evidence: route inventory and leakage tests.
  Handoff: If duplication with `taru-api` becomes a maintenance risk, split a
  follow-on to move public route inventory into `taru-client-protocol`.

## M35.4 Docs And Closeout

- [x] RCS-050 [owner=planner] [deps=RCS-040] [scope=docs/api/HTTP_API.md, docs/GOALS.md, docs/ROADMAP.md, docs/workstreams/rust-client-sdk]
  Goal: Document Rust SDK usage, boundary, validation commands, and close M35
  with a prompt-to-artifact audit.
  Validation: `cargo fmt --all -- --check`, `cargo check --workspace --tests`,
  `cargo nextest run -p taru-client --no-fail-fast`,
  `cargo nextest run --workspace --no-fail-fast`,
  `cargo tree -p taru-client`, `cargo tree -p taru-client-protocol`,
  `npm run check --prefix sdk/typescript`, and `git diff --check`.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Handoff: Record follow-ons for crates.io publishing, full streaming, Rust
  CLI, Dart/Flutter SDK, and concrete client apps.
