# Generated Artifact Provider Mapping Breadth - Handoff

Status: Active
Last updated: 2026-06-02

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
- `GAPM-040` extended bulk apply summaries, batch snapshots, Admin DTOs, HTTP
  responses, generated TypeScript contracts, and Web Admin read-model mapping
  with Provider Mapping apply/skip/noop counters.
- Provider Subject and Provider Mapping primitives already exist in
  `nako-core`, `nako-db`, `nako-metadata`, and Admin Catalog Governance read
  models.

## Active Task

- Task ID: `GAPM-060`
- Lane: `library-metadata-control-plane`
- Status: ready
- Owner: planner

Goal: verify lane evidence, close the Provider Mapping breadth workstream, and
split any deeper Provider Mapping follow-ons if needed.

## Recommended Verification Start

Read:

- `CONTEXT.md`
- `DESIGN.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/CONTROL_PLANE.md`

Then inspect the current code/document slices:

- `web/src/api/admin/generated/contract.ts`
- `web/src/api/admin/read-models-data-source.ts`
- `web/src/api/admin/mutations-data-source.ts`
- `web/src/features/admin/admin-generated-artifacts.tsx`
- `web/src/features/admin/admin-generated-artifact-metadata-apply.tsx`
- existing Web contract/route tests under `web/src/test/`
- `docs/workstreams/generated-artifact-provider-mapping-breadth/EVIDENCE_AND_GATES.md`
- `docs/workstreams/generated-artifact-provider-mapping-breadth/TASKS.jsonl`

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
- `GAPM-040`: bulk apply plan summaries and persisted batch summaries now
  expose Provider Mapping apply/skip/noop counters. Server and HTTP tests prove
  those counters flow through bulk plan, batch confirm/status/result responses,
  and execution still uses the one-artifact apply path. Generated Admin
  contracts under `apps/admin-web` and `web` are synchronized, and Web
  read-model mapping can carry the counters forward for `GAPM-050`.
- `GAPM-050`: Web Admin now renders Provider Mapping plan/result facts beside
  Canonical Metadata field actions in the single-artifact Metadata Authority
  apply route, single-apply result state, bulk plan summaries, bulk per-item
  rows, and bulk batch result rows. Data-source contract tests, route/route
  state tests, type-check, bundle budget, and browser smoke confirm the UI is
  redaction-safe, honest about fixture fallback, and responsive on desktop and
  mobile widths.

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
- Bulk apply reuses the single-artifact Metadata Authority apply path and only
  aggregates Provider Mapping counters from the per-artifact plans; it does not
  add a second provider mapping executor.

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

- None for `GAPM-060`.
- Re-run PostgreSQL parity only if closeout broadens beyond this Web-only slice
  and changes repository transaction behavior.

## Parallelism

`GAPM-050` is complete in the current worktree. Keep `GAPM-060` serial so the
closeout reflects fresh evidence, consistent task ledger state, and any split
follow-ons.
