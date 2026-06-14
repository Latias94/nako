# M2 large-library reliability plan

## Goal

Define the first executable M2 slice for large-library reliability. The goal is
to move Nako beyond the M1 operator smoke path toward predictable behavior for
larger libraries, remote storage, retryable background work, and restart
recovery without opening broad feature work prematurely.

## What I Already Know

* The current roadmap defines M2 as large-library reliability: watcher and
  incremental scan, source hash scheduling, VFS repair, job priority/retry,
  SQLite/PostgreSQL parity, and backup/recovery gates.
* The M1 `fast` release ladder is green on current HEAD, so the next task does
  not need to fix an observed M1 failure.
* `docs/architecture/STORAGE_VFS.md` says source fingerprint hash scheduling,
  evidence persistence, VFS cache repair durable jobs, and Admin diagnostics
  foundations are already shipped.
* `docs/architecture/CONTROL_PLANE.md` says broader job-kind scheduler
  migration, recurring VFS cache repair scheduling/execution, source duplicate
  reconciliation policy, unified trace context, and API scale/cache contracts
  remain follow-ons.
* `docs/architecture/LANES.md` warns not to reopen completed M1 slices or stale
  candidate queues.

## Assumptions

* M2 should start from a narrow reliability slice, not a broad platform rewrite.
* The first slice should produce measurable operator value or failure evidence.
* We should prefer existing control-plane and storage/VFS seams over new raw
  background workers.
* If a slice adds durable work, it must follow ADR 0053 control-plane rules.

## Research References

* `docs/ROADMAP.md` - M2 product target and deferred breadth.
* `docs/GOALS.md` - current roadmap convergence state and evidence-driven task
  routing.
* `docs/architecture/STORAGE_VFS.md` - shipped storage/source/VFS repair
  foundations and follow-ons.
* `docs/architecture/CONTROL_PLANE.md` - durable job, scheduler, diagnostics,
  trace, and API scale follow-ons.
* `docs/architecture/LANES.md` - lane ownership and current task routing rules.

## Candidate First Slices

### Option A: Watcher and incremental scan reliability

Add a narrow M2 slice around watcher productization: stable observation,
debounce/retry behavior, queue pressure visibility, and restart-safe scan
admission evidence for larger local libraries.

Pros:

* Direct operator value for daily library updates.
* Builds on existing watch-folder runtime instead of inventing a new worker.
* Good bridge between M1 scan smoke and M2 reliability.

Cons:

* Mostly local-library value unless remote watcher semantics are explicitly
  deferred.
* Could grow into a broad incremental scan redesign if not tightly scoped.

### Option B: VFS cache durable repair automation policy

Promote the existing dry-run/manual VFS cache repair flows into a narrowly
controlled recurring or operator-enabled enqueue/execution policy.

Pros:

* Directly targets remote-storage reliability.
* Builds on shipped durable repair contracts and Admin automation plan/enqueue
  routes.
* Fits M2's repair/retry/recovery theme.

Cons:

* Higher risk because it may mutate repair queue state automatically.
* Needs strict non-destructive boundaries and redaction tests.

### Option C: Durable job drilldown and retry reliability evidence

Pick one already-shipped durable job class and deepen Admin Jobs drilldown,
retry, cancellation, or stuck-job diagnosis based on queue-pressure evidence.

Pros:

* Improves control-plane operability across reliability features.
* Can stay schema-light if it starts with diagnostics and tests.

Cons:

* Less user-visible than scan or VFS repair.
* Risk of becoming a generic job UI/platform task.

## Recommendation

Start with Option A. Watcher and incremental scan reliability is the clearest
M2-first product path because it exercises large-library behavior without
reopening VFS mutation policy or generic job platform breadth.

## Decision (ADR-lite)

**Context**: M2 needs a first slice that improves large-library reliability
without pulling the project back into broad platform work.

**Decision**: Choose watcher and incremental scan reliability as the first
implementation slice.

**Consequences**: The first M2 task should deepen watch-folder observation,
scan admission, restart safety, and queue pressure visibility while keeping
VFS repair automation and generic durable-job drilldown as follow-ons unless
a failure evidence forces a different choice.

## Requirements

* Write a focused implementation task for watcher and incremental scan
  reliability.
* Keep the first slice inside existing architecture lanes and control-plane
  rules.
* Define explicit non-goals so M2 does not become a broad rewrite.
* Identify the minimum code/spec/test context needed by implementation and
  check agents.

## Acceptance Criteria

* [ ] PRD identifies one recommended M2 first slice.
* [ ] PRD records 2-3 feasible alternatives with trade-offs.
* [ ] The chosen first slice has testable acceptance criteria.
* [ ] Out-of-scope boundaries prevent broad M2 platform creep.
* [ ] `implement.jsonl` and `check.jsonl` include relevant specs/research.

## Definition of Done

* Planning task is started and validated.
* Follow-on implementation task is clear enough to hand to a Trellis
  implementer.
* No production code changes are made in this planning slice.
* Worktree remains clean except for the planning task files.

## Out of Scope

* Implementing watcher/incremental scan behavior in this planning task.
* Implementing automatic VFS repair mutation.
* Adding new schema, Admin API, public API, or Web UI.
* Reopening completed M1 release convergence tasks.
* Running expensive release gates unless needed for planning evidence.

## Technical Notes

Likely context for Option A implementation:

* `docs/architecture/STORAGE_VFS.md`
* `docs/architecture/LIBRARY_PIPELINE.md`
* `docs/architecture/CONTROL_PLANE.md`
* `.trellis/spec/nako-server/backend/index.md`
* `.trellis/spec/nako-library/backend/index.md`
* `crates/nako-server/src/app/watch_folder_runtime.rs`
* `crates/nako-server/src/app/jobs.rs`
* `crates/nako-library/src/ingestion.rs`

## Open Questions

* None. The first slice is watcher and incremental scan reliability.
