# TypeScript SDK Package Hardening Milestones

Status: Completed
Last updated: 2026-05-17

## M34.0 Scope And Boundary Baseline

Status: completed.

Outcome: M34 has a dedicated workstream, package boundary, generated-source
policy, task ledger, and validation gates.

Primary evidence:

- `docs/workstreams/typescript-sdk-package/DESIGN.md`
- `docs/workstreams/typescript-sdk-package/TODO.md`

## M34.1 Strict Compile Fix

Status: completed.

Outcome: the generated SDK runtime compiles under strict TypeScript settings.

Exit criteria:

- strict temporary TypeScript compile probe passes;
- `cargo nextest run -p nako-api --no-fail-fast` passes.

## M34.2 Package Skeleton And Generation Command

Status: completed.

Outcome: `sdk/typescript` is a minimal package that can refresh generated
source and run its own compile gate.

Exit criteria:

- `npm run generate --prefix sdk/typescript`
- `npm run check --prefix sdk/typescript`

## M34.3 Contract Sync Checks

Status: completed.

Outcome: Rust tests prove the committed TypeScript package entry stays in sync
with the `nako-api` generator and preserves the M33 public-surface checks.

Exit criteria:

- `cargo nextest run -p nako-api --no-fail-fast`

## M34.4 Docs And Closeout

Status: completed.

Outcome: M34 closes only after every package hardening requirement is covered
by concrete code, tests, docs, or an explicit follow-on.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo check -p nako-api --examples`
- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo tree -p nako-client-protocol`
- `npm run generate --prefix sdk/typescript`
- `npm run check --prefix sdk/typescript`
- `git diff --check`
- Workstream status is updated to completed.

Primary evidence:

- `sdk/typescript/package.json`
- `sdk/typescript/tsconfig.json`
- `sdk/typescript/src/index.ts`
- `crates/nako-api/src/sdk.rs`
- `crates/nako-api/examples/emit-typescript-sdk.rs`
- `docs/workstreams/typescript-sdk-package/EVIDENCE_AND_GATES.md`
