# Generated Artifact Provider Mapping Breadth - Handoff

Status: Active
Last updated: 2026-06-01

## Current State

This lane is open as the next focused follow-on after
`generated-artifact-bulk-metadata-apply`.

Current source state:

- Generated Artifact review acceptance stages artifacts but does not mutate
  Canonical Metadata.
- One-artifact and bulk Metadata Authority apply can apply supported Canonical
  Metadata fields with field locks, stale-target checks, durable outcomes, and
  Web confirmation.
- `GAPM-020` extended `AutomationAppService::plan_generated_artifact_metadata_apply`
  with read-only Provider Mapping proposal planning and counters.
- `GAPM-030` extended final one-artifact metadata apply so applyable Provider
  Subject proposals persist accepted Provider Mappings through the generated
  artifact metadata apply outcome transaction.
- Provider Subject and Provider Mapping primitives already exist in
  `nako-core`, `nako-db`, `nako-metadata`, and Admin Catalog Governance read
  models.

## Active Task

- Task ID: `GAPM-040`
- Lane: `library-metadata-control-plane`
- Status: ready
- Owner: codex

Goal: surface Provider Mapping counters and outcomes through bulk plan, batch
result, Admin DTOs, HTTP routes, and generated TypeScript contracts.

## Recommended Implementation Start

Read:

- `CONTEXT.md`
- `DESIGN.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/generated-artifact-metadata-authority-apply/CLOSEOUT.md`
- `docs/workstreams/generated-artifact-bulk-metadata-apply/CLOSEOUT.md`
- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/CONTROL_PLANE.md`

Then inspect the current code slices:

- `crates/nako-core/src/automation.rs`
- `crates/nako-core/src/media/provider.rs`
- `crates/nako-core/src/repository/metadata.rs`
- `crates/nako-api/src/admin/automation.rs`
- `crates/nako-server/src/app/automation.rs`
- `crates/nako-server/src/http/admin.rs`
- generated Admin TypeScript contract outputs if Admin DTOs change.

## Completed Evidence

- `GAPM-020`: core/API/server plan types now include
  `provider_mappings`, Provider Mapping action/reason enums, safe Provider
  Subject plan facts, and apply/skip/noop mapping counters.
- `GAPM-020`: server tests cover valid Provider Mapping proposals, unsupported
  provider proposals, missing subject keys, read-only behavior, redaction, and
  no Provider Mapping writes during planning. Applyable Provider Mapping plans
  were deferred until `GAPM-030` added persistence.
- `GAPM-020`: Admin DTO tests and generated TypeScript contract tests pass;
  `apps/admin-web` and `web` generated Admin contracts are synchronized.
- `GAPM-020`: `docs/api/HTTP_API.md` documents single-artifact metadata
  apply-plan Provider Mapping entries.
- `GAPM-030`: final single-artifact apply now writes Provider Subjects and
  accepted Provider Mappings only during Metadata Authority apply, never during
  review acceptance or read-only planning.
- `GAPM-030`: Provider Mapping writes are included in the same generated
  artifact metadata apply outcome transaction as Canonical Metadata
  persistence; SQLite and PostgreSQL repository implementations share the same
  commit contract.
- `GAPM-030`: server tests cover first apply, idempotent replay, candidate
  promotion to accepted, rejected-mapping preservation, stale-target
  pre-mutation failure, and mixed metadata-field/provider-mapping outcomes.

## Decisions

- Use `GAPM` as the task prefix.
- First executable task was read-only plan support, not mutation.
- Review acceptance must remain staging-only.
- Provider Mapping apply happens during final Metadata Authority apply, not in
  addon intake or review acceptance.
- Start with host-interpreted Provider Subject proposal shapes and Nako
  provider terminology; do not pass through raw provider payloads.
- Provider Mapping persistence uses existing Provider Subject and Provider
  Mapping repositories through the generated artifact outcome transaction.
- Final apply uses `MetadataSource::User` for Provider Mappings because the
  Admin confirmation is the authority boundary.

## Watchpoints

- Do not expose raw artifact JSON, prompts, provider payloads, Source Locators,
  paths, tokens, secrets, or idempotency keys.
- Do not create a second provider mapping implementation for bulk apply.
- Do not bypass target freshness checks.
- Do not let Provider Mapping changes ignore durable outcome replay.
- Do not broaden into provider search/depth, hierarchy repair, or conflict
  diagnostics inside this lane.
- Do not change addon protocol or `nako-official-addons` unless a later planner
  decision explicitly scopes that work.

## Blockers

- None for `GAPM-040`.
- Re-run PostgreSQL parity if additional repository transaction behavior
  changes.

## Parallelism

Keep `GAPM-040` serial until the generated Admin contract shape is stable.
After `GAPM-040` is accepted, `GAPM-050` Web work can run independently from
docs closeout verification if the DTO/result fields are fixed.
