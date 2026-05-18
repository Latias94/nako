# Public Client Source Locator Redaction Milestones

Status: Proposed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

Exit criteria:

- Public locator leakage problem is explicit.
- Public Client, Admin API, and internal boundaries are separated.
- First executable task is an audit and contract decision.

Primary evidence:

- `DESIGN.md`
- `TODO.md`

## M1 - Exposure Audit And Contract Decision

Exit criteria:

- Every `locator` / `input_locator` exposure is classified.
- Public DTO replacement policy is chosen.
- Compatibility risk is recorded before wire-shape changes.

Primary gates:

- `rg "locator|input_locator" crates/taru-client-protocol crates/taru-api crates/taru-server/src/http`
- `git diff --check`

## M2 - Public DTO And Server Mapping

Exit criteria:

- Public protocol DTOs no longer expose raw locators.
- `taru-api` mapping redacts or replaces locator fields.
- Server route tests prove public JSON shape.

Primary gates:

- `cargo check -p taru-client-protocol --tests`
- `cargo check -p taru-api --tests`
- focused `cargo nextest run -p taru-server <public-route-filter> --no-fail-fast`

## M3 - OpenAPI And SDK Sync

Exit criteria:

- OpenAPI artifact no longer advertises raw public locators.
- TypeScript/Rust SDK generation or inventory tests agree with DTO changes.
- HTTP API docs describe the redaction policy.

Primary gates:

- `cargo nextest run -p taru-api --no-fail-fast`
- SDK/OpenAPI generation checks used by existing client contract lanes
- `git diff --check`

## M4 - Closeout

Exit criteria:

- Fresh validation evidence is recorded.
- Admin diagnostics or compatibility follow-ons are split if needed.
- `WORKSTREAM.json` and `HANDOFF.md` reflect final state.
