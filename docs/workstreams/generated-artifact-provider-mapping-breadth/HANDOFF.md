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
- Provider Subject and Provider Mapping primitives already exist in
  `nako-core`, `nako-db`, `nako-metadata`, and Admin Catalog Governance read
  models.

## Active Task

- Task ID: `GAPM-030`
- Lane: `library-metadata-control-plane`
- Status: ready
- Owner: codex

Goal: make final Generated Artifact metadata apply upsert Provider Subjects and
accepted Provider Mappings idempotently through host-owned repositories.

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
  are intentionally deferred/non-executable until `GAPM-030` adds persistence,
  preventing final apply from claiming a partial metadata-only mutation.
- `GAPM-020`: Admin DTO tests and generated TypeScript contract tests pass;
  `apps/admin-web` and `web` generated Admin contracts are synchronized.
- `GAPM-020`: `docs/api/HTTP_API.md` documents single-artifact metadata
  apply-plan Provider Mapping entries.

## Decisions

- Use `GAPM` as the task prefix.
- First executable task was read-only plan support, not mutation.
- Review acceptance must remain staging-only.
- Provider Mapping apply should eventually happen during final Metadata
  Authority apply, not in addon intake or review acceptance.
- Start with host-interpreted Provider Subject proposal shapes and Nako
  provider terminology; do not pass through raw provider payloads.
- Provider Mapping persistence should use existing Provider Subject and
  Provider Mapping repositories, extending outcome transaction boundaries only
  if needed for atomic success claims.

## Watchpoints

- Do not expose raw artifact JSON, prompts, provider payloads, Source Locators,
  paths, tokens, secrets, or idempotency keys.
- Do not create a second provider mapping implementation for bulk apply.
- Do not bypass target freshness checks.
- Do not let Provider Mapping changes ignore durable outcome replay.
- Do not broaden into provider search/depth, hierarchy repair, or conflict
  diagnostics inside `GAPM-020`.
- Do not change addon protocol or `nako-official-addons` unless a later planner
  decision explicitly scopes that work.

## Blockers

- None for `GAPM-030`, but the worker must decide whether existing generated
  artifact apply outcome persistence needs a repository contract extension so
  Provider Subject/Mapping writes and outcome records do not produce partial
  success claims.

Run PostgreSQL parity if repository transaction behavior changes.

## Parallelism

Not safe yet. `GAPM-030` should run serially because it defines the persistence
and idempotency behavior that bulk and Web result tasks consume.

After `GAPM-030` is accepted, `GAPM-040` and `GAPM-050` can be considered for
parallelization only if the generated Admin contract shape and persistence
outcomes are stable.
