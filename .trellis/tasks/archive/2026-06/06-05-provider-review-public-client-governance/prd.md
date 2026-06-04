# Provider Review Public Client Governance

## Goal

Prevent Metadata Candidate Review, Provider Mapping governance, and related
hierarchy application surfaces from drifting into the Public Client API. This
task turns the repeated "no Public Client API route" follow-on from the
metadata governance lanes into executable contract guards.

## Requirements

- Keep Metadata Candidate Review list/detail/apply/batch/related-hierarchy
  routes Admin-only.
- Keep Provider Mapping governance route shapes out of `nako-client-protocol`
  Public Client route inventory.
- Keep Provider Mapping governance terms, request fields, idempotency keys, raw
  provider payload terms, and related hierarchy application terms out of Public
  OpenAPI and generated Public Client SDK output.
- Preserve the existing Admin API contract and Admin Web generated contract.
- Add only guardrails/tests unless an audit finds a real Public Client leak.

## Acceptance Criteria

- [ ] Public Client route inventory rejects provider governance and related
  hierarchy route shapes directly in the protocol crate.
- [ ] Public OpenAPI explicitly excludes all current Candidate Review Admin
  route shapes, including related hierarchy plan/apply.
- [ ] Public TypeScript SDK and OpenAPI provider-governance forbidden-term
  guards include related hierarchy application terms.
- [ ] Admin contract route inventory still includes the Admin routes and proves
  they stay out of Public Client inventory.
- [ ] Focused tests pass for `nako-client-protocol` and `nako-api`.

## Definition of Done

- `cargo fmt --all -- --check`
- Focused contract tests for Public Client route inventory, Public OpenAPI, SDK,
  and Admin contract pass.
- `python ./.trellis/scripts/task.py validate 06-05-provider-review-public-client-governance`
- No generated Public Client or Admin contract drift unless the generator
  intentionally changes output.

## Technical Approach

This is a governance hardening slice. The correct MVP is to deepen executable
contract tests around existing route inventories and generated contracts, not to
add a Public Client metadata governance API.

Implementation should:

- add a Public Client protocol test that forbids provider governance,
  Candidate Review, batch apply, idempotency, and related hierarchy route
  fragments;
- add related hierarchy paths to the explicit Public OpenAPI excluded-path list;
- extend the shared provider-governance forbidden term list with
  `related-hierarchy`, `related_hierarchy`, and related application-plan/apply
  spellings if missing.

## Decision (ADR-lite)

**Context**: Metadata governance lanes repeatedly split Public Client API
exposure as a future follow-on while implementing Admin-only review, apply,
batch, durable execution, and related hierarchy application.

**Decision**: Treat the current follow-on as a negative-contract governance
slice: no new Public Client API, only executable guards proving the public
surface stays clean.

**Consequences**: Future work that intentionally exposes any public provider
metadata concept must make an explicit contract decision and update these tests.
Operator mutation and review workflows remain under Admin API.

## Out of Scope

- No new Public Client route, DTO, SDK method, or OpenAPI path.
- No Admin route shape change.
- No metadata, provider, storage, database, or server runtime behavior change.
- No mutation-capable undo implementation.
- No Douban TV/episode endpoint work.

## Technical Notes

- Candidate source: `docs/architecture/LIBRARY_PIPELINE.md` and
  `docs/architecture/WORKSTREAM_LINKS.md` list
  `proposed:provider-review-public-client-governance`.
- Prior audit task:
  `.trellis/tasks/archive/2026-06/06-02-03c-provider-governance-audit-public-contract/`.
- Existing guards:
  - `crates/nako-api/src/lib.rs` shared forbidden public terms;
  - `crates/nako-api/src/openapi.rs` Public OpenAPI exclusion tests;
  - `crates/nako-api/src/sdk.rs` Public TypeScript SDK forbidden-term tests;
  - `crates/nako-api/src/admin_contract.rs` Admin route inventory vs Public
    Client inventory tests;
  - `crates/nako-server/src/http/tests/system.rs` Admin-only related hierarchy
    route auth tests.
- Research detail:
  `research/public-client-governance-audit.md`.
