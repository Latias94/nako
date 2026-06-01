# Generated Artifact Apply Repair Actions

Status: Active
Last updated: 2026-06-02

## Why This Lane Exists

`generated-artifact-apply-operations-repair` shipped a redaction-safe recovery
queue and `web-admin-generated-artifact-recovery-ui` made it visible in Web
Admin. Operators can now see stale, failed, skipped, noop, and resolved apply
state, but the product still lacks a bounded repair action that is explicitly
tied to the visible recovery context.

The risk is subtle: adding a generic "retry" button would bypass the
architecture that already makes Metadata Authority apply safe. Repair actions
must not replay a stale plan blindly. They must route through the same planning,
freshness, idempotency, authorization, and audit semantics as the existing
single-artifact and bulk apply flows.

## Source Coverage

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| Product goal and prior closeouts | Covered | `docs/workstreams/generated-artifact-apply-operations-repair/CLOSEOUT.md`; `docs/workstreams/web-admin-generated-artifact-recovery-ui/CLOSEOUT.md` | Confirms recovery is read-only and repair action is a separate follow-on. |
| Admin/API boundary | Covered | `docs/adr/0027-admin-api-boundary-for-web-console.md` | Repair actions are Admin API only and must stay redacted. |
| Control-plane boundary | Covered | `docs/adr/0053-application-control-plane-boundary.md`; `docs/architecture/CONTROL_PLANE.md` | Repair must not hide durable or supervised work inside a one-off helper. |
| Metadata apply semantics | Covered | `docs/workstreams/generated-artifact-metadata-authority-apply/DESIGN.md` | Existing single-artifact apply owns freshness, idempotency, and audit. |
| Bulk apply semantics | Covered | `docs/workstreams/generated-artifact-bulk-metadata-apply/DESIGN.md` | Existing bulk apply owns durable batch execution and per-item outcomes. |
| Code seam | Covered | `crates/nako-server/src/app/automation.rs`; `crates/nako-core/src/automation.rs`; `crates/nako-api/src/admin/automation.rs`; `web/src/api/admin/mutations-data-source.ts`; `web/src/features/admin/admin-generated-artifact-metadata-apply.tsx`; `web/src/features/admin/admin-generated-artifact-recovery.tsx` | First task must prove whether a wrapper is needed or existing apply routes are enough. |
| Related repository | Out of scope | `nako-official-addons` | Repair acts on host-owned Generated Artifact apply outcomes, not addon packaging. |

## Target State

When this lane closes:

1. Operators have a repair path from the recovery queue that is not a blind
   retry.
2. The execution kernel is the existing Metadata Authority single-artifact
   apply path or durable bulk apply path unless `GAARA-020` proves a narrow
   recovery wrapper is necessary.
3. Any wrapper, if added, carries only recovery-context guards such as prior
   outcome id, batch id, attention class, expected artifact id, and operator
   idempotency key; it does not duplicate metadata application logic.
4. Target freshness is rechecked immediately before mutation.
5. Idempotent replay returns the existing durable outcome or batch state.
6. Admin API and Web responses remain redaction-safe.
7. Tests prove stale-target rejection, idempotent replay, no raw leakage, and
   Web confirmation ergonomics.

## In Scope

- Audit the current apply and bulk apply seams for repair suitability.
- Add a read-only repair preparation contract if existing routes do not expose
  enough recovery context.
- Add a narrow Admin mutation wrapper only if it adds real guard value over the
  existing apply routes.
- Add Web Admin repair preparation/confirmation UX from the recovery queue.
- Add focused Rust/Web tests and update generated Admin TypeScript contracts if
  public Admin DTOs change.
- Update workstream and architecture docs.

## Out Of Scope

- No automatic repair loop.
- No raw plan replay from an old failed outcome.
- No provider identity/depth precision work.
- No Public Client API changes.
- No broad durable-job scheduler redesign.
- No schema migration unless the repair context cannot be proven from existing
  outcome and batch records.

## Architecture Direction

### Prefer Existing Apply As The Execution Kernel

`AutomationService::apply_generated_artifact_metadata` already replans, checks
target freshness, records an idempotent outcome, and returns a redacted result.
`create_generated_artifact_metadata_bulk_apply_batch` and
`execute_generated_artifact_metadata_bulk_apply_batch` already provide durable
batch semantics and per-item outcomes.

The first implementation task must prove whether those seams are enough for
repair. If they are enough, the work should focus on recovery-context UX and
tests instead of adding backend mutation surface.

Read-only recon on 2026-06-02 supports this direction: no new metadata
mutation core is needed. The likely choices are Web-only repair preparation
over existing apply routes, or a narrow Admin wrapper that adds recovery-row
guards before delegating to existing apply behavior.

### Add Wrapper Only For Guards

A repair wrapper is justified only if it prevents operator or client mistakes
that existing apply routes cannot prevent, for example:

- confirming against the wrong artifact after a stale recovery row;
- losing the prior outcome or batch context in audit;
- applying a row that is now `resolved` or no longer `needs_repair`;
- confusing replay-only/noop state with actionable repair.

The wrapper must call existing apply or bulk apply logic rather than copying
metadata mutation code.

### Web Must Stay Explicit

Recovery rows should expose a deliberate repair preparation step. Live mutation
must require an idempotency key and clear confirmation. Fixture/fallback mode
must not claim to execute repair.

## Stop Conditions

Return to planner coordination before implementation continues if the lane
requires:

- changing Public Client API;
- exposing raw artifact JSON, prompts, provider payloads, Source Locators,
  local paths, bearer tokens, secrets, or idempotency keys;
- adding a second metadata apply executor;
- schema changes outside automation apply outcome or batch ownership;
- a new durable-job priority policy;
- repair semantics that need provider-depth precision first.

## First Executable Task

Start with `GAARA-020`: repair action seam audit and preparation contract.

This task should answer whether repair needs a new backend mutation, a narrow
guard wrapper, or only Web UX over existing apply routes. It should add or
update tests around stale-target rejection, idempotent replay, and recovery
context before any wider mutation UX ships.
