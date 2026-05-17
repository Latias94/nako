# TypeScript SDK Package Hardening TODO

Status: Completed
Last updated: 2026-05-17

## M34.0 Scope And Boundary Baseline

- [x] TSP-010 [owner=planner] [deps=none] [scope=docs/workstreams/typescript-sdk-package]
  Goal: Freeze the package directory, generated-source policy, local TypeScript
  dependency policy, non-goals, and validation gates.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json,
  and HANDOFF.md exist and agree.
  Evidence: this workstream.
  Handoff: Continue with TSP-020 before broadening to npm publishing or other
  SDK languages.

## M34.1 Strict Compile Fix

- [x] TSP-020 [owner=codex] [deps=TSP-010] [scope=crates/taru-api/src/sdk.rs]
  Goal: Fix the generated TypeScript runtime so the emitted SDK compiles under
  strict TypeScript settings.
  Validation: temporary `npx -p typescript tsc --noEmit` probe and
  `cargo nextest run -p taru-api --no-fail-fast`.
  Evidence: generator tests and package compile gate.
  Handoff: Keep the runtime dependency-free and avoid weakening strictness just
  to hide generated-code issues.

## M34.2 Package Skeleton And Generation Command

- [x] TSP-030 [owner=codex] [deps=TSP-020] [scope=sdk/typescript, crates/taru-api/examples]
  Goal: Add the minimal TypeScript SDK package, package-local TypeScript
  dependency, repeatable generation command, generated source entry, and
  compile command.
  Validation: `npm run generate --prefix sdk/typescript` and
  `npm run check --prefix sdk/typescript`.
  Evidence: package files, generated `src/index.ts`, and lockfile.
  Handoff: Do not publish npm packages in this slice.

## M34.3 Contract Sync Checks

- [x] TSP-040 [owner=codex] [deps=TSP-030] [scope=crates/taru-api/src/sdk.rs, sdk/typescript]
  Goal: Prove the committed package entry is synchronized with the Rust
  generator and still excludes admin/internal surfaces.
  Validation: `cargo nextest run -p taru-api --no-fail-fast`.
  Evidence: Rust tests compare package source with generator output or otherwise
  prove sync, route coverage, and leakage rejection.
  Handoff: Keep sync validation in Rust so OpenAPI route drift fails without
  requiring npm for every focused generator test.

## M34.4 Docs And Closeout

- [x] TSP-050 [owner=planner] [deps=TSP-040] [scope=docs/api/HTTP_API.md, docs/GOALS.md, docs/ROADMAP.md, docs/workstreams/typescript-sdk-package]
  Goal: Document generation/check commands and close M34 with a
  prompt-to-artifact audit.
  Validation: `cargo fmt --all -- --check`, `cargo check --workspace --tests`,
  `cargo check -p taru-api --examples`,
  `cargo nextest run -p taru-api --no-fail-fast`,
  `cargo nextest run --workspace --no-fail-fast`,
  `cargo tree -p taru-client-protocol`,
  `npm run generate --prefix sdk/typescript`,
  `npm run check --prefix sdk/typescript`, and `git diff --check`.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Handoff: Record follow-ons for npm publishing, Dart/Flutter SDK, Rust SDK,
  OpenAPI runtime serving, and concrete client apps.
