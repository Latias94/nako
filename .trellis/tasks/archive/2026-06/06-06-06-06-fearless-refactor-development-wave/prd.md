# 06-06 Fearless Refactor And Development Wave

## Goal

Continue high-value Nako development and fearless refactoring until
2026-06-06 10:00 Asia/Shanghai, using small verified Trellis slices. If a slice
finishes early, continue by reviewing architecture, adding findings to the
problem ledger, and starting the next bounded implementation or refactor task.

## What I Already Know

- The user authorizes autonomous development, issue discovery, and code
  commits for this wave.
- Current branch is `main`; the working tree was clean when the wave started.
- The previous cross-lane architecture audit is completed and source hash
  policy work is archived.
- Recent source hash audit recommends Admin manual triggering first, then
  duplicate relationship idempotency, then read-only reconciliation planning.
- Admin/API contracts, generated contracts, durable job scheduling, DB schema,
  source identity repositories, and redaction surfaces are shared conflict
  zones.

## Operating Rules

- Work in small slices that can be reviewed and committed independently.
- Prefer the correct long-term boundary over tiny local patches when evidence
  shows a real design problem.
- Delete obsolete code, redundant helpers, and shallow pass-throughs only when
  tests and call-site evidence support deletion.
- Do not invent abstractions for hypothetical future adapters.
- Keep source identity, source fingerprint evidence, and duplicate
  relationship mutation separate unless a task explicitly changes that
  contract.
- Use `cargo fmt` when practical and prefer `cargo nextest run` for Rust tests.
- Commit only verified task-scoped changes with Conventional Commit messages.
- Push after successful commits unless the next slice needs local-only review.

## Wave Plan

1. Build and maintain `research/problem-ledger.md`.
2. Open child tasks for each implementation/refactor slice.
3. Start with `admin-source-fingerprint-hash-trigger-first-slice`.
4. Continue to duplicate relationship idempotency and read-only reconciliation
   planning if time remains.
5. If source identity work blocks, switch to storage/source identity PostgreSQL
   runtime harness or VFS cache remediation planning.

## Candidate Backlog

| Priority | Candidate | Type | Why |
| --- | --- | --- | --- |
| P1 | Admin source fingerprint hash trigger first slice | Feature | Existing source hash queue/executor path needs an operator trigger. |
| P1 | Source duplicate relationship idempotent pair upsert | Correctness/refactor | SQLite has pair uniqueness; PostgreSQL needs equivalent idempotency before automatic reconciliation. |
| P1 | Source duplicate reconciliation read-only plan | Feature foundation | Enables safe duplicate suggestions without mutation. |
| P2 | Source hash retry/requeue Admin command | Feature | Reuses durable job semantics; useful after manual trigger. |
| P2 | Storage/source identity PostgreSQL runtime harness | Verification | Catches backend drift before more source identity mutation. |
| P2 | VFS cache non-destructive remediation plan | Feature/refactor | Ready follow-on from previous storage audit. |
| P3 | Admin contract generator deepening | Refactor | Worth doing only if this wave keeps touching Admin generated contracts. |
| P3 | Admin Web route search/helper cleanup | Refactor | Lower-risk frontend cleanup after backend contract tasks. |

## Acceptance Criteria

- [ ] A problem ledger records findings, decisions, and deferred candidates.
- [ ] At least one child implementation/refactor slice is completed and
      committed.
- [ ] Each completed slice has focused validation evidence.
- [ ] Specs or research notes are updated when new durable contracts are
      learned.
- [ ] Work remains commit-scoped and the final working tree is clean or clearly
      explained.

## Definition Of Done

- New or changed behavior has focused tests or explicit docs-only rationale.
- `git diff --check` passes before commit.
- Rust code changes run focused `cargo nextest` or `cargo check` gates.
- Generated contracts are regenerated from source, not hand-edited.
- Journal records completed sessions and commits.
- The wave can be resumed from Trellis task files without relying on chat.

## Out Of Scope

- No broad unbounded rewrite without a child task and refactor brief.
- No hidden background work outside durable job/runtime boundaries.
- No automatic Source Duplicate Relationship mutation before idempotency,
  plan/apply, redaction, and rollback semantics are explicit.
- No cross-library duplicate-source behavior until Library Access implications
  are specified.
- No unrelated formatting churn.

## Technical Notes

- Previous source hash policy research:
  `.trellis/tasks/archive/2026-06/06-05-source-hash-triggering-reconciliation-policy/research/source-hash-triggering-reconciliation-policy.md`
- Source hash trigger/reconciliation spec:
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
- Parent cross-lane synthesis:
  `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/synthesis.md`
- Storage/library/control-plane audit:
  `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/storage-library-control-plane.md`

## Decision (ADR-lite)

**Context**: The user wants continuous development and fearless refactoring
until 2026-06-06 10:00, with permission to commit and continue automatically
when a slice finishes.

**Decision**: Run this as a Trellis parent wave with child tasks. Use the
source fingerprint hash Admin trigger as the first implementation slice because
it has high value, clear prior research, and bounded shared-surface risk. Keep
finding and scoring additional architecture/refactor candidates in a persistent
problem ledger.

**Consequences**: The wave can keep moving without asking for each next slice,
but each implementation still has its own scoped PRD, validation, and commit.
Shared Admin/API/DB/runtime surfaces remain serialized by child task ownership.
