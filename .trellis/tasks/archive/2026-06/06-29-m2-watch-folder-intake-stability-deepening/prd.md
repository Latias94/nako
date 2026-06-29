# M2 Watch Folder Intake Stability Deepening

## Goal

Advance the M2 large-library reliability roadmap by deepening the existing
watch-folder intake runtime into a clearer, operator-verifiable reliability
slice. The current M1 evidence says no new M1 blocker should be opened without
failed ladder evidence; the next product foundation is incremental library
intake for self-hosted operators with large files, slow copies, and repeated
scan pressure.

## What I Already Know

* `docs/ROADMAP.md` marks M2 as large-library reliability: watcher/incremental
  scan, source hash scheduling, VFS repair, job retry, database parity, and
  backup/recovery gates.
* `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md` says remaining
  M1 follow-ons should be opened only from failed ladder or coverage-matrix
  evidence. No current evidence forces a Media Web/player or Admin repair
  blocker task.
* `docs/architecture/LIBRARY_PIPELINE.md` marks `Watcher/debounce` as weak and
  names `library-watcher-and-media-intake-stability` as the next direction.
* The code already has more than a pure planner:
  * `nako-library::intake` has stable-candidate evidence and intake planning.
  * `nako-server::app::watch_folder_runtime` supervises local watch-folder
    polling, discovers candidates, plans intake, records latest tick
    diagnostics, and admits library scans for newly stable candidates.
  * startup tests already cover stable observations, restart continuation,
    discovery failure backoff, queued/running scan reuse, planned-write
    suppression, and reconciliation completion.
* Admin overview and operator readiness already expose redaction-safe
  watch-folder coverage and latest tick pressure.

## Requirements

* Keep the task scoped to M2 watch-folder intake reliability; do not reopen M1
  blocker work unless new release ladder evidence appears.
* Preserve existing redaction boundaries: no raw local paths, Source Locators,
  storage URIs, filenames, credentials, durable input JSON, or raw backend
  errors in Admin/API diagnostics.
* Deepen the existing watch-folder runtime rather than adding a second intake
  scheduler or bypassing the durable scan admission path.
* Identify and implement the smallest missing product slice that makes watch
  intake more trustworthy for large-library operation. Candidate areas include:
  * stronger intake/tick diagnostics for operator evidence;
  * explicit bounded behavior for repeated stable candidates and blocked
    candidates;
  * route or readiness improvements only if the operator journey cannot inspect
    existing facts;
  * focused tests that prove no premature scan admission for copy-in-progress
    files and no duplicate scan storm when candidates remain stable.
* Keep OS-native file watcher integration, destructive storage repair,
  backend configuration mutation, and broad job-kind scheduler migration out of
  scope for this slice.

## Acceptance Criteria

* [x] The implemented slice has focused unit/app/route tests proving the new
      watch-folder intake behavior.
* [x] Watch-folder intake still admits scans only through existing
      `LibraryScanAppService` watch-folder admission.
* [x] Repeated ticks for already-ready candidates do not create unbounded scan
      churn.
* [x] Copy-in-progress or incomplete stability evidence does not trigger a
      premature scan.
* [x] Operator-facing diagnostics remain bounded and redaction-safe.
* [x] `docs/architecture/LIBRARY_PIPELINE.md` or related Trellis specs are
      updated if the task establishes a durable new convention.
      No spec update was needed because the existing `nako-server` quality
      guidelines already require watch-folder intake tests for size-only
      stability fallback, missing size evidence, and changed-size reset before
      scan admission.
* [x] Focused Rust tests pass with `cargo nextest`; broader checks are run only
      as risk requires.

## Verification

* Added app-level watch-folder runtime coverage for repeated stable candidates
  after a completed scan: the runtime keeps the candidate ready but does not
  enqueue or reuse another `LibraryScan` job.
* Added app-level missing-size evidence coverage: repeated watch-folder ticks
  keep the candidate inspecting, do not admit a scan, do not create a
  `LibraryScan` job, and keep serialized intake plans redaction-safe.
* Ran `cargo nextest run -p nako-server watch_folder_runtime_tick --no-fail-fast`:
  10 tests passed.
* `trellis-check` additionally reported:
  * `python3 ./.trellis/scripts/task.py validate ./.trellis/tasks/06-29-m2-watch-folder-intake-stability-deepening`: passed.
  * `cargo fmt --all -- --check`: passed.
  * `git diff --check`: passed with only LF/CRLF warning.
  * `cargo check -p nako-server --tests`: passed.
  * `cargo nextest run -p nako-library intake --no-fail-fast`: 10 tests passed.
  * `cargo check -p nako-api -p nako-server --tests`: passed.

## Definition Of Done

* Tests added or updated for the changed behavior.
* `cargo fmt --all` or focused formatting is run when Rust files change.
* Focused `cargo nextest` gates pass.
* Admin contract and Admin Web generated artifacts are updated only if API DTOs
  change.
* Trellis task context and architecture notes reflect any new reusable pattern.
* Changes are committed with a Conventional Commit message.

## Technical Approach

Start from the shipped watch-folder runtime and choose the smallest missing M2
slice after one more focused code pass. The default implementation direction is
to improve scan-admission stability and operator evidence inside the existing
runtime:

1. Keep pure intake decisions in `nako-library::intake` when the rule can be
   expressed without server/runtime state.
2. Keep runtime admission and latest-tick state in
   `crates/nako-server/src/app/watch_folder_runtime.rs`.
3. Keep durable scan creation/reuse behind `LibraryScanAppService`; do not
   enqueue jobs directly from new helper code.
4. Expose or refine Admin API/Web facts only if existing overview/readiness
   cannot prove the behavior.

## Decision (ADR-lite)

Context: M1 release convergence is already evidence-driven and should not be
expanded without a concrete ladder failure. M2 explicitly calls out
watcher/incremental scan productization as the next large-library reliability
foundation.

Decision: Open an M2 watch-folder intake stability deepening task instead of a
new M1 Media Web/player, Admin repair, or broad control-plane migration task.

Consequences: This keeps momentum on self-hosted media-server maturity while
respecting the M1 routing rule. The slice must be narrow enough to ship with
focused tests and must not pretend to solve full OS watcher integration.

## Out Of Scope

* OS-native file watcher daemon integration.
* Remote storage watcher support beyond existing local-root coverage.
* New destructive storage repair, purge, delete, or invalidation behavior.
* Backend configuration mutation.
* Broad durable-job scheduler migration.
* Media Web browse/player hardening unless release evidence exposes a concrete
  browse/play blocker.
* Automatic Source Duplicate Relationship merge or undo policy.

## Technical Notes

* Roadmap: `docs/ROADMAP.md`
* Goal map: `docs/GOALS.md`
* Library pipeline map: `docs/architecture/LIBRARY_PIPELINE.md`
* Control-plane map: `docs/architecture/CONTROL_PLANE.md`
* M1 diagnostics routing: `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md`
* Relevant specs:
  * `.trellis/spec/nako-library/backend/index.md`
  * `.trellis/spec/nako-server/backend/index.md`
  * `.trellis/spec/nako-api/backend/index.md`
  * `.trellis/spec/admin-web/frontend/index.md` if Admin Web changes are needed
