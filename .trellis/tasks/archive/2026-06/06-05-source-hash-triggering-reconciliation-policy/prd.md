# Source fingerprint hash triggering and reconciliation policy

## Goal

Decide how source fingerprint hash work should be triggered and how persisted
hash evidence may feed Source Duplicate Relationship suggestions before adding
more queue, Admin/API mutation, or automatic reconciliation behavior.

## Requirements

- Compare possible triggers: scan-originated enqueue, Admin manual enqueue,
  retry/requeue, policy-backed automatic scheduling, and operator-only
  diagnostics.
- Decide whether source hash evidence should create or update
  Source Duplicate Relationship records, and under what confidence/staleness
  rules.
- Preserve the domain rule that Source Fingerprint is evidence, not Source
  identity.
- Keep ADR 0053 control-plane boundaries: no raw background work hidden outside
  durable jobs or runtime supervision.
- Identify Admin/API DTO and redaction implications.

## Acceptance Criteria

- [x] The audit recommends a trigger policy for source fingerprint hash work.
- [x] The audit recommends whether duplicate relationship mutation is in scope
      for the next implementation slice.
- [x] Automatic reconciliation risks, rollback/undo needs, and operator
      visibility are documented.
- [x] Parallel conflict surfaces with scan scheduling, durable jobs, Admin DTOs,
      and source identity repositories are listed.
- [x] A first bounded follow-on task is recommended.

## Definition of Done

- Research output is written under this task or linked from the parent audit.
- No production code changes are made in this audit task.
- `git diff --check` passes.

## Out of Scope

- No new Admin/Public API route in this audit task.
- No source hash enqueue or retry implementation.
- No automatic Source Duplicate Relationship mutation.
- No schema migration.
- No source-hash-specific runtime loop.

## Technical Notes

- Parent audit: `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/`
- Key research:
  - `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/storage-library-control-plane.md`
  - `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/synthesis.md`
- Important docs/ADRs:
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - ADR 0016, 0017, 0053
