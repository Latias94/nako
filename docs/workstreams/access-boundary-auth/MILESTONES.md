# Access Boundary And Token Authentication Milestones

Status: Completed
Last updated: 2026-05-17

## M31.0 Scope And Boundary Baseline

Status: completed.

Outcome: M31 has a dedicated ADR, workstream, starting audit, task ledger, and
validation gates.

Primary evidence:

- `docs/adr/0024-inbound-token-authentication-boundary.md`
- `docs/workstreams/access-boundary-auth/DESIGN.md`
- `docs/workstreams/access-boundary-auth/TODO.md`

## M31.1 Protocol And Config Slice

Status: completed.

Outcome: public auth failure codes and inbound auth config are stable.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check -p nako-client-protocol --tests`
- `cargo nextest run -p nako-client-protocol --no-fail-fast`
- `cargo nextest run -p nako-server config --no-fail-fast`

## M31.2 HTTP Middleware Slice

Status: completed.

Outcome: non-health HTTP routes are protected by bearer token auth when enabled.

Exit criteria:

- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server http::tests::system --no-fail-fast`

## M31.3 Docs And Route Evidence Slice

Status: completed.

Outcome: docs and route tests make the auth boundary auditable.

Exit criteria:

- `cargo nextest run -p nako-server http::tests --no-fail-fast`

## M31.4 Closeout

Status: completed.

Outcome: M31 closes only after every explicit auth/access-boundary requirement
is covered by concrete code, tests, docs, or an explicit follow-on.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo tree -p nako-client-protocol`
- `git diff --check`
- Workstream status is updated to completed.
