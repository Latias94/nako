# TypeScript SDK Package Hardening Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Starting Repro

- `taru-api` can emit TypeScript SDK code.
- There is no TypeScript package directory.
- There is no committed TypeScript compile gate.
- Initial strict `tsc` probing found generated runtime type errors around
  exact optional property assignments and typed query records.

## Gate Set

### TypeScript Package Gate

```bash
npm run generate --prefix sdk/typescript
npm run check --prefix sdk/typescript
```

### Targeted Rust Gate

```bash
cargo check -p taru-api --examples
cargo nextest run -p taru-api --no-fail-fast
```

### Workspace Closeout Gate

```bash
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
cargo tree -p taru-client-protocol
git diff --check
```

## Evidence Anchors

- `crates/taru-api/src/sdk.rs`
- `crates/taru-api/examples/emit-typescript-sdk.rs`
- `sdk/typescript/package.json`
- `sdk/typescript/tsconfig.json`
- `sdk/typescript/src/index.ts`
- `docs/api/HTTP_API.md`
- `docs/workstreams/typescript-sdk-package/`

## Prompt-To-Artifact Checklist

- Establish TypeScript SDK package boundary:
  workstream docs and `sdk/typescript`.
- Define generated-source policy:
  package README, generated file banner, and generation command.
- Add local TypeScript toolchain:
  package dev dependency and lockfile.
- Make generation repeatable:
  package script invoking `taru-api`.
- Prove generated code compiles:
  `npm run check --prefix sdk/typescript`.
- Preserve M33 public-surface checks:
  `cargo nextest run -p taru-api --no-fail-fast`.
- Keep npm publishing, UI, Flutter/Dart, and Rust SDK out of scope:
  docs and handoff follow-ons.
- Validate:
  final gate output recorded before closeout.

## Recorded Evidence

### TSP-010 Scope And Boundary Baseline

- Workstream docs define `sdk/typescript` as the package location, local
  TypeScript dependency policy, generated-source policy, non-goals, task
  ledger, evidence anchors, and gate set.

### TSP-020 Strict Compile Fix

- `crates/taru-api/src/sdk.rs` now emits strict-TypeScript-compatible runtime
  types for optional bearer token state, request options, and query input.
- A temporary `npx -y -p typescript@5.9.3 tsc --noEmit ...` compile probe
  passed against emitted SDK output before package wiring.
- `cargo nextest run -p taru-api --no-fail-fast`: passed, 10 tests before the
  package sync test was added.

### TSP-030 Package Skeleton And Generation Command

- `sdk/typescript/package.json` defines local TypeScript tooling and
  `generate`, `check`, and `verify` scripts.
- `sdk/typescript/tsconfig.json` runs a strict DOM/fetch-compatible compile
  gate with `exactOptionalPropertyTypes` and `noUncheckedIndexedAccess`.
- `sdk/typescript/src/index.ts` is the committed generated SDK package entry.
- `sdk/typescript/package-lock.json` pins TypeScript `5.9.3`.
- `npm run generate --prefix sdk/typescript`: passed.
- `npm run check --prefix sdk/typescript`: passed.

### TSP-040 Contract Sync Checks

- `crates/taru-api/examples/emit-typescript-sdk.rs` supports
  `--output <path>` for repeatable package generation without shell
  redirection.
- `typescript_package_entry_matches_generator_output` compares the committed
  package entry against `taru_api::sdk::typescript_sdk()`.
- `cargo nextest run -p taru-api --no-fail-fast`: passed, 11 tests.
- Existing SDK tests continue to verify public route method/path coverage,
  auth/version/error/pagination runtime details, and admin/internal leakage
  rejection.

### TSP-050 Docs And Closeout

- `docs/api/HTTP_API.md` documents the TypeScript SDK package generation and
  compile commands.
- `docs/GOALS.md`, `docs/ROADMAP.md`, `docs/README.md`, and
  `docs/workstreams/README.md` record M34 as completed.
- Closeout validation:
  - `npm run generate --prefix sdk/typescript`: passed.
  - `npm run check --prefix sdk/typescript`: passed.
  - `cargo fmt --all -- --check`: passed.
  - `cargo check --workspace --tests`: passed.
  - `cargo check -p taru-api --examples`: passed.
  - `cargo nextest run -p taru-api --no-fail-fast`: passed, 11 tests.
  - `cargo nextest run --workspace --no-fail-fast`: passed, 264 tests.
  - `cargo tree -p taru-client-protocol`: normal dependency is `serde`; dev
    dependency is `serde_json`; no server/internal crate dependencies.
  - `git diff --check`: passed with Git CRLF normalization warnings only.
