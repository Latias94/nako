# Generated Artifact Metadata Authority Apply - Design

Status: Closed
Last updated: 2026-06-01

## Problem

Nako can now review Generated Artifact proposals, but acceptance only marks the
artifact as accepted. Operators still have no host-owned way to inspect the
generated metadata as a field-level Canonical Metadata application plan, compare
it with locks and current item state, and commit it through Metadata Authority.

Directly making `accept` apply metadata would violate ADR 0004, ADR 0007, ADR
0015, and ADR 0027:

- automation and Addons submit facts, suggestions, or side effects;
- Canonical Metadata is mutated only by Nako-owned policy;
- field locks, NFO/local authority, provider priority, and library refresh mode
  must be respected;
- Admin API responses must remain redaction-safe and must not expose raw
  prompts, raw payloads, source locators, paths, or secrets.

## Current Boundary

Generated Artifact review is deliberately a staging action:

- `crates/nako-core/src/automation.rs` defines
  `GeneratedArtifactAcceptanceActionKind::StageMetadataAuthorityReview` and a
  boundary with `requires_metadata_authority_apply: true`.
- `crates/nako-server/src/app/automation.rs` marks accepted/rejected status but
  does not call metadata application.
- `crates/nako-api/src/admin/automation.rs` exposes the acceptance boundary and
  redacted payload summaries, not raw generated payloads.
- `web-admin-generated-artifact-review-mutations` closed the frontend review
  mutation lane and explicitly split Metadata Authority apply as a follow-on.
- `crates/nako-server/src/app/metadata_application.rs` already owns a
  server-side metadata application module for Addon metadata writeback. It
  applies `nako-core::MetadataMergePolicy`, resolves library refresh mode,
  projects catalog state, and emits safe apply reports.

The gap is not a missing button. The gap is a missing apply-plan contract and
audited apply lifecycle between Accepted Generated Artifact and Canonical
Metadata mutation.

## Target State

An accepted metadata Generated Artifact can be applied through a separate
Metadata Authority workflow:

1. Admin requests an apply plan for an accepted metadata Generated Artifact.
2. Nako validates target freshness and parses the generated payload into a
   supported metadata patch/fact shape.
3. Nako computes a redacted, field-level plan showing candidate values,
   current values, skipped locked fields, blocked reasons, and whether the
   plan is executable.
4. Admin confirms apply with an idempotency key.
5. Nako revalidates the plan, runs the host-owned metadata application policy,
   updates catalog/search projection, persists an apply result/audit record,
   and returns a redacted outcome.
6. Web Admin shows that `accept` and `apply` are separate authority decisions.

## Scope

- `nako-core`: pure Generated Artifact metadata apply-plan/result vocabulary if
  the DTO needs to be shared across API/server/tests.
- `nako-api`: Admin DTOs for apply plan and apply result.
- `nako-db`: persistence only if the apply result needs its own idempotent audit
  row instead of reusing existing artifact status.
- `nako-server`: Generated Artifact apply-plan orchestration, target freshness,
  payload parsing, MetadataApplication delegation, catalog/search commit, and
  Admin routes.
- `web/`: Admin apply-plan/confirmation route after backend contracts exist.
- Docs/workstreams: evidence, gates, and closeout records.

## Non-Goals

- Do not make review acceptance mutate Canonical Metadata.
- Do not expose raw `artifact_json`, prompts, provider responses, local paths,
  Source Locators, or credentials in Admin API or Web UI.
- Do not add provider-specific TMDB/Douban/Bangumi mapping policy in this lane
  unless the Generated Artifact payload contract requires a minimal neutral
  metadata patch shape.
- Do not move server `MetadataApplication` into `nako-metadata` just for reuse;
  the previous cross-path audit kept repository/catalog orchestration in
  `nako-server`.
- Do not implement bulk apply before the one-artifact authority workflow is
  proven.

## Refactor Brief

Intent: remove the future trap where "accepted Generated Artifact" is mistaken
for "Canonical Metadata was changed". The new boundary makes review, authority
planning, and apply separate, testable states.

Scope: Generated Artifact acceptance records, Admin DTOs/routes, server
automation orchestration, host metadata application policy, optional persistence
for idempotent apply audit, and Web Admin confirmation.

Deletion plan: remove or avoid UI/API wording that implies accept equals apply;
avoid direct raw-payload-to-metadata shortcuts; delete any temporary fixture
apply path if it appears during Web work.

Boundary plan: Generated Artifact review owns proposal status. Metadata
Authority apply owns field-level diff, locks, stale-target checks, and final
mutation. Admin API owns redacted wire contracts. Web Admin owns operator
confirmation and result display.

Testing plan: characterize the current no-mutation behavior first, then add
apply-plan DTO tests, server app tests, SQLite/PostgreSQL repository contract
tests if persistence is added, and Web Admin route/data-source tests once the
backend contract is real.

Risk plan: protect user-locked fields, reject stale targets before mutation,
preserve PostgreSQL parity for any schema work, keep long-running apply work
behind durable/runtime policy if it expands beyond a request-local metadata
commit, and treat raw generated payloads as privileged internals.

Workflow plan: this is a durable fearless-refactor lane. `GAMA-010` opened and
audited the boundary; execution starts with `GAMA-020` apply-plan contract.

## Closeout

Closed by `GAMA-070` on 2026-06-01. The target state is satisfied for the
one-artifact authority workflow: review acceptance, apply-plan, final apply,
durable idempotency outcomes, and Web confirmation are separate and covered by
focused backend/Web gates. See `CLOSEOUT.md` for final evidence, review result,
residual risks, and follow-ons.
