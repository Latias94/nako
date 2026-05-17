# TypeScript SDK Package Hardening

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

M33 proved that `taru-api` can emit a dependency-free TypeScript SDK scaffold,
but the generated TypeScript was only protected by Rust-side string checks.
M34 adds a real package and `tsc` gate so future Web and CLI clients can depend
on a compile-checked API surface.

## Relevant Authority

- ADRs:
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
- Existing workstreams:
  - `docs/workstreams/sdk-client-scaffold/`
  - `docs/workstreams/openapi-client-contract/`
- Code boundaries:
  - `crates/taru-api/src/sdk.rs`
  - `crates/taru-api/examples/emit-typescript-sdk.rs`
  - `sdk/typescript/`

## Starting Audit

- The repo has no existing JavaScript or TypeScript package convention.
- Node and npm are available locally, but `tsc` is not installed globally.
- The first strict compile probe exposed exact-optional-property and typed
  query-shape issues in the M33 generated client runtime.
- `.gitignore` does not yet exclude JavaScript dependency directories.

## Target State

- `sdk/typescript` is the canonical TypeScript SDK package directory.
- The package contains a generated source entry and minimal package metadata.
- Generation is repeatable from the Rust workspace command line.
- `npm run check` runs `tsc --noEmit` against the generated SDK.
- TypeScript is a package-local development dependency, not a global tool
  requirement.
- Generated code still covers auth, version, error envelope, pagination, and
  public route methods, while rejecting admin/internal leakage.

## In Scope

- Add `sdk/typescript/package.json`, `tsconfig.json`, README, and generated
  `src/index.ts`.
- Add a Rust example or command path that writes the generated SDK to a file.
- Fix the TypeScript runtime emitted by `taru-api` so strict compile succeeds.
- Add a Rust-side generation sync test or equivalent static check.
- Update API, roadmap, goal, and workstream docs.

## Out Of Scope

- npm publishing or registry release automation.
- Web, CLI, Flutter, Dart, or Rust SDK implementation.
- Browser e2e tests or a full mock server.
- Server public API expansion beyond small DTO/schema hygiene required by the
  compile contract.
- Any dependency from TypeScript packaging back into server/internal crates.

## Architecture Direction

Use `sdk/typescript` instead of `clients/typescript` because this lane creates
a reusable SDK package, not a concrete application. Keep the generated SDK in
`sdk/typescript/src/index.ts` so editors and consumers see the real API surface,
but make the file explicitly generated and refreshable from `taru-api`.

TypeScript is a package-local dev dependency. `node_modules` stays ignored and
uncommitted; `package-lock.json` is committed to pin the compile toolchain.

## Closeout Condition

This lane can close when:

- the package can regenerate its source from `taru-api`;
- `npm run check --prefix sdk/typescript` passes;
- Rust SDK generator tests still pass;
- docs record the generation and compile commands;
- full validation gates pass;
- and follow-ons for npm publishing, Dart/Flutter SDK, Rust SDK, and concrete
  clients are recorded outside M34.
