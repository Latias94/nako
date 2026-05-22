# Public API Contract Hardening Milestones

Status: Completed
Last updated: 2026-05-17

## M30.0 Scope And Contract Baseline

Status: completed.

Outcome: M30 has a dedicated workstream, ADR, contract audit, task ledger, and
validation gates.

Exit criteria:

- Public API versioning and error envelope problem is explicit.
- Public client route subset vs server-admin/internal scope is explicit.
- First implementation slice is selected.
- Top-level docs point to the M30 lane.

Primary evidence:

- `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
- `docs/workstreams/public-api-contract/DESIGN.md`
- `docs/workstreams/public-api-contract/TODO.md`

## M30.1 Protocol Error Vocabulary Slice

Status: completed.

Outcome: the stable public error-code vocabulary lives in
`nako-client-protocol`, while the JSON envelope remains compatible with the
current `code/message` shape.

Deliverables:

- Protocol-owned public error-code constants or enum.
- Serialization tests for the public error envelope.
- `nako-api` re-export path for server adapter use.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check -p nako-client-protocol --tests`
- `cargo nextest run -p nako-client-protocol --no-fail-fast`
- `cargo tree -p nako-client-protocol`

## M30.2 Server Error Mapping And Version Identity Slice

Status: completed.

Outcome: server HTTP errors map through the protocol-owned public vocabulary,
and `/health` proves API version identity for v1 clients.

Deliverables:

- Server error mapping uses protocol-owned codes.
- Tests cover public error status/code/message behavior.
- `/health` tests cover protocol version identity.

Exit criteria:

- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server http::tests::system --no-fail-fast`
- `cargo nextest run -p nako-server http::tests::playback --no-fail-fast`

## M30.3 Public Route Contract Evidence Slice

Status: completed.

Outcome: the first public client route set has route-level evidence for
success envelopes, pagination metadata, API version identity, and error
envelopes.

Deliverables:

- Catalog/search/list/detail route evidence.
- Library source/list route evidence.
- Playback decision/probe/session error route evidence where public.
- HTTP API docs updated with public/internal route boundary.

Exit criteria:

- `cargo nextest run -p nako-server http::tests --no-fail-fast`
- `cargo tree -p nako-client-protocol`

## M30.4 Closeout

Status: completed.

Outcome: M30 closes only after every prompt requirement is covered by concrete
code, tests, docs, or an explicit follow-on.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo tree -p nako-client-protocol`
- `git diff --check`
- Workstream status is updated to completed.
