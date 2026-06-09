# Overnight Architecture Refactor And Jellyfin Comparison Campaign

## Goal

Continuously improve Nako until 2026-06-08 10:00 +08:00 by comparing Nako's
architecture with `repo-ref/jellyfin`, finding concrete architecture gaps, and
shipping verified fearless-refactor or reliability slices. Each medium slice may
be committed independently after focused validation.

## What I Already Know

- The user explicitly approved autonomous development, fearless refactoring,
  continued architecture review, and self-commits for verified slices.
- The current git working directory started clean on `main`.
- Recent completed work shipped VFS cache repair durable jobs, manual
  enqueue/execute/retry, disk-scan scheduler integration, and Admin Jobs
  diagnostics projection.
- Remaining high-value architecture areas include Storage/VFS reliability,
  Control Plane scheduling, Admin diagnostics, Source Fingerprint / Source
  Duplicate Relationship productization, and trace context.
- `repo-ref/jellyfin` exists locally and is reference material only.

## Reference-Code Boundary

- Use Jellyfin to study behavior, capability boundaries, operator workflows,
  failure handling, and architecture trade-offs.
- Do not copy, translate line by line, or import Jellyfin source, comments,
  migrations, tests, schemas, assets, or generated code.
- Implement original Nako code against Nako domain terms, ADRs, specs, and
  tests.

## Requirements

- Start with read-only Nako/Jellyfin comparison across storage, library scan,
  jobs/scheduler, diagnostics, and cache/artifact lifecycle.
- Convert findings into medium, independently verifiable Trellis tasks.
- Prefer fearless refactors that deepen modules, delete accidental complexity,
  shrink caller interfaces, and improve locality.
- Prefer non-destructive reliability slices before destructive cache mutation
  or automatic repair execution.
- Keep raw storage identity, paths, backend URLs, credentials, raw errors,
  fingerprints, digests, and job payloads out of Admin/Public surfaces.
- Use focused validation before each commit:
  - `cargo fmt --all -- --check` when Rust files change;
  - `cargo nextest run` focused by package/test where practical;
  - `cargo check -p ... --tests` for changed Rust package surfaces;
  - frontend checks only when frontend files change.
- Update architecture maps, specs, and task evidence when behavior or durable
  guidance changes.

## Initial Candidate Slices

1. VFS cache repair automated policy planner / dry-run eligibility, with no
   automatic execution or destructive cache mutation.
2. VFS cache purge/delete/invalidation design and non-mutating contract tests.
3. Source Duplicate Relationship reconciliation suggestion diagnostics, without
   automatic Media Source merge.
4. Broader job-kind scheduler migration for one narrow job family.
5. Unified trace context consistency for source hash / VFS repair job paths.
6. Deep module refactors discovered during comparison, especially shallow
   mappers or duplicated redaction/action-plan logic.

## Acceptance Criteria

- [ ] Comparison research identifies concrete Nako/Jellyfin differences and
      translates them into Nako-native opportunities.
- [ ] At least one medium refactor or development slice is completed, verified,
      documented, and committed.
- [ ] Each completed slice has Trellis evidence and focused validation output.
- [ ] If a slice finishes early, the campaign continues with the next highest
      value architecture issue without waiting for user input.
- [ ] Any remaining open risks are written into task evidence or architecture
      follow-ons.

## Definition Of Done

- Work continues until the time target, an external blocker, or user direction.
- Completed slices are committed with Conventional Commit messages.
- The final journal entry summarizes completed commits, validation, and next
  backlog candidates.

## Out Of Scope

- Copying Jellyfin implementation artifacts.
- Unreviewed schema migrations.
- Public API expansion without explicit task evidence and generated contract
  updates.
- Destructive cache purge/delete/invalidation execution before a Nako-owned
  policy and audit model exists.
- Automatic source merge/reconciliation without operator-visible policy and
  undo/repair evidence.
