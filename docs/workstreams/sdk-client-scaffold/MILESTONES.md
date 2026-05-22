# SDK Generation And Client Integration Scaffold Milestones

Status: Completed
Last updated: 2026-05-17

## M33.0 Scope And Boundary Baseline

Status: completed.

Outcome: M33 has a dedicated workstream, scope boundary, task ledger, and
validation gates.

Primary evidence:

- `docs/workstreams/sdk-client-scaffold/DESIGN.md`
- `docs/workstreams/sdk-client-scaffold/TODO.md`

## M33.1 TypeScript SDK Scaffold Generator

Status: completed.

Outcome: `nako-api` can emit a dependency-free TypeScript SDK scaffold.

Exit criteria:

- `cargo check -p nako-api --examples`
- `cargo nextest run -p nako-api --no-fail-fast`

## M33.2 SDK Contract Smoke Checks

Status: completed.

Outcome: SDK generator tests prove route coverage, auth/error/version behavior,
pagination support, and admin/internal leakage rejection.

Exit criteria:

- `cargo nextest run -p nako-api --no-fail-fast`

## M33.3 Docs And Closeout

Status: completed.

Outcome: M33 closes only after every explicit SDK scaffold requirement is
covered by concrete code, tests, docs, or an explicit follow-on.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo tree -p nako-client-protocol`
- `git diff --check`
- Workstream status is updated to completed.

Primary evidence:

- `docs/api/HTTP_API.md`
- `docs/workstreams/sdk-client-scaffold/EVIDENCE_AND_GATES.md`
- `crates/nako-api/src/sdk.rs`
- `crates/nako-api/examples/emit-typescript-sdk.rs`
