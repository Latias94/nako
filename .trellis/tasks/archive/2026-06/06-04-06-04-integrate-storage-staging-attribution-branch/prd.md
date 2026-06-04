# Integrate Storage Staging Attribution Branch

## Goal

Port the completed `task/06-03-06b-storage-staging-attribution-persistence`
worktree onto current `main`, resolve merge conflicts, and either integrate the
storage staging attribution behavior safely or document why it must be split
again.

## What I Already Know

* `06a-library-watcher-runtime-productization` was a clean integrated residual
  and has been removed.
* `06b-storage-staging-attribution-persistence` is clean but has two patches not
  present on `main`:
  * `6da64fdc feat(storage): persist staging attribution`
  * `fc90957c chore(task): archive staging attribution task`
* The original 06b task is completed and archived inside the worktree.
* `git merge-tree --write-tree main task/06-03-06b-storage-staging-attribution-persistence`
  reports conflicts in DB/storage/API/server files and task evidence.

## Requirements

* Preserve the original 06b product intent: persisted authoritative staging
  attribution for storage pressure, diagnostics, and scan-admission behavior.
* Reconcile current `main` changes instead of force-merging stale branch state.
* Keep ambiguous same-root and multi-endpoint cases explicit; do not invent
  false library/source ownership.
* Preserve SQLite/PostgreSQL repository contract parity.
* Preserve public/Admin redaction boundaries: no raw source locators, local
  paths, source fingerprints, backend credentials, etags, headers, or host
  filesystem details in diagnostics.
* Keep generated Admin contracts in sync only if the integrated API shape still
  requires them.
* Keep the integration scoped to storage staging attribution; do not absorb
  watcher runtime, Jellyfin watcher reference research, cache repair workflows,
  or broader PostgreSQL runtime suite expansion.

## Acceptance Criteria

* [ ] `06b` behavior is ported onto current `main` or a smaller follow-up split
      is documented with evidence.
* [ ] Merge conflicts are resolved without reverting unrelated current `main`
      work.
* [ ] SQLite and PostgreSQL staging attribution migrations/adapters/tests remain
      coherent.
* [ ] Admin/API contract changes are regenerated or removed according to the
      final API shape.
* [ ] Storage/server diagnostics remain redaction-safe.
* [ ] Focused DB/server/API/Admin gates pass.
* [ ] `cargo fmt --all -- --check` and `git diff --check` pass.
* [ ] If integrated, the stale `06b` worktree and branch are removed after the
      accepted changes are on `main`.

## Technical Approach

1. Inspect the current `06b` patch and current `main` versions of conflict
   files.
2. Apply the implementation intentionally onto current `main` using small,
   reviewable edits rather than a blind merge commit.
3. Resolve schema versioning and generated contract drift explicitly.
4. Run focused gates before considering full workspace validation.
5. Commit with a Conventional Commit message, archive/update task evidence, and
   push only after all gates pass.

## Known Conflict Files

* `.trellis/spec/nako-db/backend/database-guidelines.md`
* `.trellis/tasks/archive/2026-06/06-03-06-03-06-library-storage-follow-on-parallel-wave/task.json`
* `.trellis/tasks/archive/2026-06/06-03-06b-storage-staging-attribution-persistence/prd.md`
* `.trellis/tasks/archive/2026-06/06-03-06b-storage-staging-attribution-persistence/task.json`
* `crates/nako-api/src/admin/storage.rs`
* `crates/nako-core/src/staging.rs`
* `crates/nako-db/migrations/0003_staging_attribution.sql`
* `crates/nako-db/migrations/postgres/0003_staging_attribution.sql`
* `crates/nako-db/src/postgres.rs`
* `crates/nako-db/src/postgres/vfs_staging.rs`
* `crates/nako-db/src/sqlite/migrations.rs`
* `crates/nako-db/src/sqlite/staging.rs`
* `crates/nako-server/src/app/storage.rs`
* `crates/nako-server/src/http/tests/system.rs`

## Definition Of Done

* Relevant specs and task evidence are updated.
* Focused and broad enough Rust gates pass.
* The main worktree remains clean after commit/push.
* Residual worktrees are either intentionally retained or cleaned with evidence.

## Out Of Scope

* Watch-folder runtime productization.
* Jellyfin watcher reference research.
* Cache repair operator workflows.
* Source fingerprint escalation policy.
* Broad PostgreSQL runtime suite expansion beyond gates required by changed
  attribution persistence.
