# Client SDK Contract Inventory And Streaming Builders Milestones

Status: Completed
Last updated: 2026-05-17

## M36.0 Scope And License Baseline

Status: completed.

Outcome: M36 has a dedicated workstream and freezes the ADR 0022 license
boundary.

Primary evidence:

- `docs/workstreams/client-sdk-contract/DESIGN.md`
- `docs/workstreams/client-sdk-contract/TODO.md`

## M36.1 Shared Public Route Inventory

Status: completed.

Outcome: the public client route inventory is owned by the permissive protocol
crate and consumed by `nako-api`.

Exit criteria:

- `cargo check -p nako-client-protocol --tests`
- `cargo nextest run -p nako-client-protocol --no-fail-fast`
- `cargo nextest run -p nako-api --no-fail-fast`

## M36.2 Rust SDK Inventory And Streaming Builders

Status: completed.

Outcome: `nako-client` uses the shared route inventory and can build
future-safe streaming requests without owning byte streaming policy.

Exit criteria:

- `cargo check -p nako-client --tests`
- `cargo nextest run -p nako-client --no-fail-fast`

## M36.3 Cross-SDK Drift And Leakage Gates

Status: completed.

Outcome: OpenAPI, TypeScript SDK, and Rust SDK all agree on public route
membership while admin/internal surfaces stay excluded.

Exit criteria:

- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo nextest run -p nako-client --no-fail-fast`
- `npm run check --prefix sdk/typescript`

## M36.4 Docs And Closeout

Status: completed.

Outcome: M36 closes only after every inventory, builder, license, and non-goal
requirement is covered by code, tests, docs, or explicit follow-on.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo tree -p nako-client-protocol`
- `cargo tree -p nako-client`
- `git diff --check`
