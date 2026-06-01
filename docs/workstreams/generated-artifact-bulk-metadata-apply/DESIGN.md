# Generated Artifact Bulk Metadata Apply - Design

Status: Active
Last updated: 2026-06-01

## Problem

`generated-artifact-metadata-authority-apply` shipped a safe one-artifact
Metadata Authority workflow, but operators still need to review and apply many
accepted metadata Generated Artifacts without clicking through each item.

Naive bulk apply is risky:

- selection can accidentally include stale, unsupported, or fixture-only
  artifacts;
- a single failed item must not hide successful item outcomes;
- retries must be idempotent per item, not just per button click;
- long-running work must not run inside a request handler;
- Web must show redacted aggregate and per-item facts without exposing raw
  prompts, payloads, Source Locators, paths, or secrets.

## Target State

Nako supports an Admin-only bulk metadata apply flow:

1. operator selects accepted metadata Generated Artifacts explicitly or through
   a bounded query selector;
2. Nako returns a read-only bulk apply plan with aggregate counters and
   per-artifact redacted plan summaries;
3. operator confirms the batch with an explicit idempotency key;
4. Nako enqueues durable batch execution instead of mutating all items in the
   HTTP request path;
5. execution applies each artifact through the existing single-artifact
   Metadata Authority apply path and records per-item outcomes;
6. Admin API and Web expose batch status, partial failures, skipped/noop
   counts, and replay-safe result facts.

## Scope

- Bulk selection request and redacted read-only plan DTOs.
- Admin API route for bulk apply plan.
- Durable bulk apply request/job model and repository persistence if needed.
- Per-item idempotency derived from batch identity plus artifact identity.
- Worker execution through existing single-artifact apply service logic.
- Admin read model for batch status and per-item result summaries.
- Web Admin bulk plan, confirm, and result display.
- Focused Rust/Web tests and docs gates.

## Non-Goals

- No provider-specific Generated Artifact mapping breadth.
- No outcome repair/search UI beyond the batch result needed for this flow.
- No change to Generated Artifact review acceptance semantics.
- No automatic application of newly accepted artifacts.
- No bypass of `MetadataApplication`, field locks, stale-target checks, or
  catalog/search projection commits.
- No broad durable job scheduler redesign.
- No Admin settings restoration.

## Architecture Direction

### Bulk Plan Is Read-Only

Bulk plan construction may call or reuse the existing one-artifact
`plan_generated_artifact_metadata_apply` logic, but it must not commit
Canonical Metadata or apply outcomes.

The plan response should include:

- requested selection facts;
- accepted and skipped counts;
- per-artifact target, status, reasons, field-action counts, and safe
  fingerprints;
- explicit reason when an artifact is not executable;
- no raw artifact JSON, prompts, provider payload, Source Locator, path, token,
  or secret values.

### Bulk Mutation Is Durable

The confirm route should enqueue a durable batch/job and return a stable batch
identity. It should not apply an unbounded batch synchronously.

Each item should use a deterministic per-item idempotency key scoped by batch
identity and artifact id. Replaying the same batch must not duplicate item
mutations.

### Partial Failure Is Normal

Bulk execution should record and surface `applied`, `noop`, `failed`,
`skipped`, and `stale` style summaries. A failed artifact should not roll back
successful artifacts from the same batch unless a later design explicitly adds
transactional all-or-nothing mode.

### Web Must Stay Honest

Fixture/fallback Web data can render a plan, but it must not claim to execute a
live mutation. Live mutation controls require an explicit operator preparation
step and a stable UI idempotency key.

## Stop Conditions

Return to planner coordination before implementation continues if the lane
requires:

- provider-specific mapping beyond the current neutral metadata suggestion
  shape;
- a new global durable job priority/scheduler policy;
- schema changes outside automation/job/outcome ownership;
- changing Public Client API;
- exposing raw artifact payloads or host paths in Admin/Web DTOs;
- raising Web bundle budgets instead of narrowing UI scope.

## First Executable Task

Start with `GABMA-020`: read-only bulk apply-plan contract and route.

This task should prove selection and redacted planning before any bulk mutation
or durable job behavior is implemented.
