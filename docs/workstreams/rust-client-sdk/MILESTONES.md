# Rust Client SDK Foundation Milestones

Status: Completed
Last updated: 2026-05-17

## M35.0 Scope And Boundary Baseline

Status: completed.

Outcome: M35 has a dedicated workstream, crate boundary, dependency policy,
task ledger, and validation gates.

Primary evidence:

- `docs/workstreams/rust-client-sdk/DESIGN.md`
- `docs/workstreams/rust-client-sdk/TODO.md`

## M35.1 Crate Skeleton And Boundary Guard

Status: completed.

Outcome: `crates/nako-client` exists as an Apache-2.0 SDK crate with clean
dependencies.

Exit criteria:

- `cargo check -p nako-client --tests`
- `cargo tree -p nako-client`

## M35.2 Async JSON Client Surface

Status: completed.

Outcome: the Rust SDK can call the core JSON public routes through a mockable
async transport.

Exit criteria:

- `cargo nextest run -p nako-client --no-fail-fast`

## M35.3 Public Inventory And Leakage Checks

Status: completed.

Outcome: route coverage and internal/admin leakage rejection are test-visible.

Exit criteria:

- `cargo nextest run -p nako-client --no-fail-fast`

## M35.4 Docs And Closeout

Status: completed.

Outcome: M35 closes only after every Rust SDK foundation requirement is covered
by code, tests, docs, or an explicit follow-on.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run -p nako-client --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo tree -p nako-client`
- `cargo tree -p nako-client-protocol`
- `npm run check --prefix sdk/typescript`
- `git diff --check`
- Workstream status is updated to completed.
