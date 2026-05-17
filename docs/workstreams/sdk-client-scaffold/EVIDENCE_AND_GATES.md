# SDK Generation And Client Integration Scaffold Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Starting Repro

- OpenAPI v1 JSON can be emitted by `taru-api`.
- There is no SDK generator or client wrapper.
- There is no static check that a generated client covers public paths or
  rejects admin/internal routes.

## Gate Set

### Targeted Iteration Gate

```bash
cargo fmt --all -- --check
cargo check --workspace --tests
```

### SDK Focus Gate

```bash
cargo check -p taru-api --examples
cargo nextest run -p taru-api --no-fail-fast
```

### Protocol Direction Gate

```bash
cargo tree -p taru-client-protocol
```

This must not show dependencies on `taru-core`, `taru-streaming`,
`taru-transcode`, or `taru-server`.

### Broader Closeout Gate

```bash
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Anchors

- `crates/taru-api/src/openapi.rs`
- `crates/taru-api/src/sdk.rs`
- `crates/taru-api/examples/emit-typescript-sdk.rs`
- `docs/api/HTTP_API.md`
- `docs/workstreams/sdk-client-scaffold/`

## Prompt-To-Artifact Checklist

- Establish SDK/client integration scaffold boundary:
  workstream docs.
- Generate TypeScript/Web/CLI scaffold from OpenAPI:
  `taru_api::sdk::typescript_sdk()` and example emitter.
- Cover auth/version/error handling:
  generated client code and tests.
- Cover core public route calls:
  generated method inventory and tests.
- Keep Dart/Flutter, package publishing, and UI out of scope:
  docs and handoff follow-ons.
- Reject admin/internal leakage:
  SDK generator tests.
- Validate:
  final gate output recorded before closeout.

## Recorded Evidence

### SDK-010 Scope And Boundary Baseline

- Workstream docs define the first SDK target, generated-output policy,
  non-goals, task ledger, evidence anchors, and gate set.

### SDK-020 TypeScript SDK Scaffold Generator

- `crates/taru-api/src/sdk.rs` generates a dependency-free TypeScript SDK
  scaffold from the M32 OpenAPI document.
- `crates/taru-api/examples/emit-typescript-sdk.rs` emits generated TypeScript
  with `cargo run -p taru-api --example emit-typescript-sdk`.
- The scaffold includes OpenAPI-derived TypeScript interfaces, `TaruClient`,
  `TaruApiError`, bearer auth injection, API version inspection, error
  envelope parsing, pagination helpers, playback capability helpers, and core
  route methods.

### SDK-030 SDK Contract Smoke Checks

- `cargo nextest run -p taru-api --no-fail-fast`: passed, 10 tests.
- SDK tests verify public route method/path coverage.
- SDK tests verify auth/version/error/pagination runtime details.
- SDK tests reject admin/internal route groups, secret references, local output
  paths, provider raw response terms, and internal crate names.

### SDK-040 Docs And Closeout

- `docs/api/HTTP_API.md` documents the TypeScript SDK scaffold generation
  command and runtime behavior.
- `docs/GOALS.md`, `docs/ROADMAP.md`, `docs/README.md`, and
  `docs/workstreams/README.md` record M33 as completed.
- `cargo run -q -p taru-api --example emit-typescript-sdk` smoke output starts
  with generated headers, API version constants, public path inventory, and
  OpenAPI-derived interfaces.
- Closeout validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check --workspace --tests`: passed.
  - `cargo check -p taru-api --examples`: passed.
  - `cargo nextest run -p taru-api --no-fail-fast`: passed, 10 tests.
  - `cargo nextest run --workspace --no-fail-fast`: passed, 263 tests.
  - `cargo tree -p taru-client-protocol`: normal dependency is `serde`; dev
    dependency is `serde_json`; no server/internal crate dependencies.
  - `git diff --check`: passed with Git CRLF normalization warnings only.
