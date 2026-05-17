# OpenAPI And Public Client SDK Contract Milestones

Status: Completed
Last updated: 2026-05-17

## M32.0 Scope And Boundary Baseline

Status: completed.

Outcome: M32 has a dedicated ADR, workstream, starting audit, task ledger, and
validation gates.

Primary evidence:

- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/workstreams/openapi-client-contract/DESIGN.md`
- `docs/workstreams/openapi-client-contract/TODO.md`

## M32.1 Protocol Response Hygiene Slice

Status: completed.

Outcome: public playback session response shape is protocol-owned and does not
expose server-local output paths.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run -p taru-client-protocol --no-fail-fast`
- `cargo nextest run -p taru-api --no-fail-fast`
- `cargo nextest run -p taru-server http::tests::playback --no-fail-fast`

## M32.2 OpenAPI Artifact Slice

Status: completed.

Outcome: `taru-api` can produce/check the first Public Client API OpenAPI v1
contract.

Exit criteria:

- `cargo nextest run -p taru-api --no-fail-fast`
- OpenAPI checker verifies public route inventory, bearer auth, API version
  headers, error envelopes, and internal/admin leakage rejection.

## M32.3 Server Route Contract Evidence Slice

Status: completed.

Outcome: HTTP behavior, docs, and OpenAPI route inventory agree for the public
client surface.

Exit criteria:

- `cargo nextest run -p taru-server http::tests --no-fail-fast`

## M32.4 Closeout

Status: completed.

Outcome: M32 closes only after every explicit OpenAPI/Public Client SDK
contract requirement is covered by concrete code, docs, tests, or an explicit
follow-on.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo tree -p taru-client-protocol`
- OpenAPI checker passes.
- `git diff --check`
- Workstream status is updated to completed.
