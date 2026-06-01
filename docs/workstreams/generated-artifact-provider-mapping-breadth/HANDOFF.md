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
- `AutomationAppService::plan_generated_artifact_metadata_apply` currently
  parses metadata suggestion payloads into Canonical Metadata fields only.
- Provider Subject and Provider Mapping primitives already exist in
  `nako-core`, `nako-db`, `nako-metadata`, and Admin Catalog Governance read
  models.

## Active Task

- Task ID: `GAPM-020`
- Lane: `library-metadata-control-plane`
- Status: ready
- Owner: codex

Goal: add read-only Provider Mapping proposal planning to the existing
Generated Artifact metadata apply plan.

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

## Decisions

- Use `GAPM` as the task prefix.
- First executable task is read-only plan support, not mutation.
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

- None for `GAPM-020`.

Later tasks may require a planner check if the worker finds that atomic outcome
persistence needs a schema or repository contract extension.

## Parallelism

Not safe yet. `GAPM-020` should run serially because it defines the public
Admin plan shape that later mutation, bulk, and Web tasks consume.

After `GAPM-020` is accepted:

- `GAPM-030` backend persistence should run before Web work.
- `GAPM-040` and `GAPM-050` can be considered for parallelization only if the
  generated Admin contract shape has stabilized.
