# VFS Cache Durable Repair Queue First Slice

## Goal

Close the active task against current repository evidence and prevent duplicate
implementation of the VFS cache durable repair queue first slice. The same
first-slice behavior is already shipped on `main`; this task should record that
evidence, freeze the already-shipped boundary, and route any remaining M2
storage reliability work to a narrower follow-on.

## What I Already Know

- The task was created after `docs/ROADMAP.md` and
  `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md` listed
  `vfs-cache-durable-repair-queue-first-slice` as a conditional M2 candidate.
- Current `docs/architecture/STORAGE_VFS.md` says the VFS cache repair durable
  job contract, internal enqueue seam, single-job executor, Admin manual
  enqueue/execute/retry routes, disk-scan scheduler integration, and internal
  retry seam shipped as of 2026-06-07.
- Current `docs/architecture/CONTROL_PLANE.md` records the same shipped
  control-plane behavior under
  `vfs-cache-repair-durable-job-contract-and-execution`.
- Recent commits show the implementation was already delivered:
  - `22a818e1 feat(storage): add VFS cache repair durable enqueue`
  - `494ffb3e feat(storage): add VFS cache repair durable executor`
  - `1ca7fa7d feat(storage): add VFS cache repair admin commands`
  - `ca9d1398 feat(storage): schedule VFS cache repair jobs`
  - `57ff4413 feat(storage): expose VFS cache repair retry route`
- `crates/nako-core`, `crates/nako-api`, and `crates/nako-server` already
  expose `JobKind::VfsCacheRepair`, `VfsCacheRepairJobInput`, Admin route
  inventory entries, enqueue/execute/retry DTOs, scheduler integration, and
  focused server tests.

## Requirements

- Do not reimplement the already-shipped durable queue first slice.
- Treat the active task as a planning/evidence-correction task unless the user
  explicitly retargets it.
- Preserve the shipped first-slice boundary:
  - durable input remains redaction-safe;
  - enqueue remains non-mutating for cache/backend/library data;
  - execution reuses selected-target refresh authority;
  - retry preserves source failed jobs for audit;
  - scheduler execution stays under the existing `disk.scan` runtime budget;
  - Admin commands return only redaction-safe job and summary facts.
- Record that the next VFS cache repair implementation work is not this task
  name, but a more precise follow-on such as:
  - `vfs-cache-repair-durable-remediation-diagnostics` for broader operator
    job diagnostics and drilldown;
  - `vfs-cache-repair-automated-policy-first-slice` for explicitly approved
    automated execution policy;
  - `vfs-cache-repair-cache-invalidation-design` for purge/delete/invalidation
    semantics;
  - `storage-vfs-postgresql-runtime-harness-next-slice` for broader backend
    parity evidence.
- If a follow-on is opened, require a fresh PRD that names the exact behavior
  gap and references the shipped first-slice evidence above.

## Acceptance Criteria

- [x] `prd.md` records that the named first slice is already shipped on `main`.
- [x] `implement.jsonl` and `check.jsonl` contain real spec context entries, not
      the seeded `_example` rows.
- [x] No Rust or generated contract files are changed by this task unless the
      user explicitly retargets it to a new implementation slice.
- [x] Follow-on candidates are scoped narrowly enough that a future implement
      agent will not duplicate enqueue/executor/Admin/scheduler/retry behavior.
- [x] Task validation passes.

## Definition Of Done

- Task context is complete and points future agents at the correct storage,
  control-plane, API, DB, and VFS specs.
- The active task is either archived as superseded by shipped evidence or
  explicitly retargeted by the user before any code edits begin.
- If retargeted, the PRD is rewritten before implementation and the follow-on
  must run focused `cargo nextest` gates for the changed packages.

## Technical Approach

This task should not dispatch an implementation agent for the current title.
The right technical action is evidence correction:

- use the shipped architecture docs as source of truth;
- capture the recent commits that already delivered the first slice;
- configure Trellis context for a possible follow-on without including code
  files in JSONL;
- leave implementation to a separately named task only after the remaining M2
  behavior is chosen.

## Decision (ADR-lite)

**Context**: The roadmap and M1 Admin diagnostics matrix originally listed
`vfs-cache-durable-repair-queue-first-slice` as a future candidate when selected
refresh and read-only remediation were insufficient. Since then, the repository
has already shipped the durable job contract, queue, manual Admin commands,
scheduler execution, and retry path.

**Decision**: Do not implement this task as named. Record it as superseded by
current `main` evidence and route remaining work to a more precise follow-on.

**Consequences**: This avoids duplicate routes, duplicate job contracts, and
drift in redaction/runtime boundaries. Future work can focus on the remaining
M2 gaps: broader diagnostics, automated policy, destructive cache semantics
design, or PostgreSQL parity evidence.

## Out Of Scope

- No new Admin route, DTO, generated TypeScript contract, or OpenAPI change.
- No new durable job kind, resource class, queue schema, executor, retry path,
  scheduler loop, or runtime resource budget.
- No cache purge, delete, invalidation, backend configuration mutation, or
  library file write.
- No change to existing VFS cache repair tests.
- No PostgreSQL parity harness expansion in this task unless the user retargets
  the task explicitly.

## Technical Notes

- Roadmap source:
  - `docs/ROADMAP.md`
  - `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md`
- Shipped boundary source:
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/CONTROL_PLANE.md`
- Predecessor planning evidence:
  - `.trellis/tasks/archive/2026-06/06-06-vfs-cache-repair-non-destructive-remediation-plan-first-slice/prd.md`
  - `.trellis/tasks/archive/2026-06/06-03-05c-storage-runtime-postgres-parity-harness/prd.md`
  - `.trellis/tasks/archive/2026-06/06-06-06-06-overnight-fearless-refactor-development-plan/prd.md`
- Relevant spec context is curated in `implement.jsonl` and `check.jsonl`.
