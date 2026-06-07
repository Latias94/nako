# Overnight Fearless Refactor And Development Plan

## Goal

Plan a continuous Nako development window through 2026-06-07 22:00
Asia/Shanghai that can alternate between evidence-driven delivery,
architecture problem discovery, and bounded fearless refactoring. The plan must
keep M1 release evidence protected while preparing a sustainable M2 reliability
and storage/VFS refactor queue.

## What I Already Know

- Current task was created after the user asked for continuous fearless
  refactor/development planning.
- Current working tree is clean except for this Trellis task directory.
- M1 release convergence evidence already passed focused `release-fast`,
  `playback`, `container`, `postgres`, and `workspace` gates in archived
  Trellis evidence.
- `docs/ROADMAP.md` selects Product-Operator M1 as the current anchor and M2
  as large-library reliability.
- `docs/architecture/LANES.md` says new M1 implementation should open only from
  concrete failed ladder modes or Admin coverage gaps.
- `docs/architecture/STORAGE_VFS.md` lists the next credible storage/VFS lanes:
  durable VFS cache remediation, source fingerprint reconciliation
  productization, and PostgreSQL runtime harness evidence.
- OpenDAL 0.57.0 is available on crates.io and supports storage services such
  as local filesystem, WebDAV, and S3, plus layers for retry, timeout, tracing,
  metrics, throttle, and capability checks.

## Assumptions

- The target deadline is 2026-06-07 22:00 Asia/Shanghai.
- The main branch remains the integration base.
- Bounded commits are acceptable after each verified medium slice.
- Production dependency changes require an explicit design decision. OpenDAL
  should not be added directly as a production dependency without an adapter
  spike proving it preserves Nako's storage domain contracts.
- The immediate value is not broad feature expansion; it is creating a durable
  issue pool and completing the highest-confidence slices from that pool.

## Requirements

- Preserve M1 evidence confidence. Do not open M1 feature work unless a fresh
  release gate exposes a concrete blocker.
- Find multiple problem candidates so development can continue after the first
  refactor completes.
- Keep every implementation slice small enough to validate with focused
  `cargo nextest` or frontend checks before moving on.
- Prefer deletion, module deepening, and boundary simplification over shallow
  compatibility wrappers.
- For storage work, keep `nako-vfs` as the domain boundary: `StorageUri`,
  Source Locator redaction, source fingerprinting, cache repair authority,
  storage health, and deterministic staging remain Nako-owned semantics.
- Treat OpenDAL as a possible adapter behind `StorageBackend`, not as a
  replacement for Nako's VFS/product model.

## Candidate Problem Pool

### P0: M1 RC Closeout And Planning Drift

Intent: Convert scattered green evidence into a clear release-candidate
decision and remove roadmap/goal drift.

Candidate slices:

- run or document final `m1-release-ladder.ps1 -Mode all` policy;
- verify packaging dry-run shape;
- update `docs/GOALS.md` if the M1 convergence entry still reads in progress;
- record whether any release-candidate gate remains intentionally skipped.

Exit signal:

- M1 is either "RC-ready except publication" or has a named blocker task.

### P1: OpenDAL Storage Adapter Spike

Intent: Decide whether OpenDAL reduces backend complexity without deleting
Nako-specific storage semantics.

Candidate slices:

- write an adapter design note comparing current `LocalFsBackend` and
  `WebDavBackend` contracts with OpenDAL `Operator` capabilities;
- add no production dependency in the first slice unless the design note proves
  a narrow adapter is safe;
- if approved, prototype an internal `OpenDalBackend` behind `StorageBackend`
  for one non-production feature-gated backend or test-only harness.

Exit signal:

- clear decision: reject, defer, or open `storage-opendal-adapter-first-slice`.

### P1: M2 Storage/VFS Reliability

Intent: Move from M1 diagnostics toward larger-library reliability.

Candidate slices:

- durable VFS cache repair queue first slice;
- source fingerprint reconciliation productization;
- PostgreSQL runtime harness for storage/source identity query paths;
- per-backend staging budget and cleanup diagnostics deepening.

Exit signal:

- one reliability path changes from read-only diagnostics to a bounded,
  non-destructive repair or runtime parity mechanism.

### P1: Library Watcher And Intake Stability

Intent: Make incremental intake predictable for large files, slow copies, and
watcher churn.

Candidate slices:

- watcher runtime/product integration plan;
- stable-candidate state transitions and scan trigger debounce tests;
- operator-visible watcher start/status/error diagnostics.

Exit signal:

- the first watcher productization slice has tests and does not bypass durable
  job/runtime supervision boundaries.

### P2: Control-Plane Trace Context And Job Observability

Intent: Make failures easier to route across HTTP, jobs, VFS, FFmpeg, and
addon work without exposing secrets.

Candidate slices:

- unify trace context propagation into one more job path;
- add redaction-safe Admin drilldown for one proven stuck job class;
- avoid broad incident bundle work until a concrete operator flow needs it.

Exit signal:

- one cross-layer path has better correlation and focused tests.

### P2: Playback Remote Resilience

Intent: Improve playback behavior on remote storage without starting broad M3
playback scope.

Candidate slices:

- remote playback timeout/circuit-breaker behavior audit;
- player error recovery task only if fresh M1/M3 evidence exposes a blocker;
- no LL-HLS/CMAF, remote workers, or hardware tone-map work unless explicitly
  promoted.

Exit signal:

- either a documented deferral or a concrete blocker-driven playback task.

## Recommended Schedule To 2026-06-07 22:00

### Wave 0: Plan Lock And Context Setup

Timebox: immediate.

- Finish this PRD.
- Record OpenDAL research.
- Curate `implement.jsonl` and `check.jsonl`.
- Decide first implementation slice from the candidate pool.

### Wave 1: M1 Closeout Guard

Timebox: 60-90 minutes.

- Check whether M1 goal state and release evidence are internally consistent.
- If only docs drift exists, fix docs.
- If a gate must be rerun, run the cheapest gate that answers the question.

Stop condition:

- any release gate fails in a way that requires implementation.

### Wave 2: Storage/OpenDAL Decision Spike

Timebox: 90-150 minutes.

- Compare current `StorageBackend` requirements to OpenDAL capabilities.
- Decide adapter shape and risk.
- Prefer a written decision and narrow tests before adding dependency weight.

Stop condition:

- the spike would require schema/public config/API changes before proving value.

### Wave 3: First M2 Reliability Slice

Timebox: 2-3 hours.

Default pick after the spike: durable VFS cache repair queue first slice, unless
OpenDAL research reveals a safer storage adapter preparatory refactor.

Validation:

- focused package check;
- focused `cargo nextest` for changed storage/server paths;
- redaction tests if Admin diagnostics or DTOs change.

### Wave 4: Continue Or Split

Timebox: 2-3 hours.

- If Wave 3 is small and verified, commit it and choose the next adjacent slice.
- If Wave 3 grows, split follow-ons in PRD/evidence instead of widening the
  patch.

### Wave 5: Broad Quality Gate

Timebox: 90-150 minutes.

- Run package-focused checks first.
- Run broader workspace or M1 ladder mode only when changed scope justifies it.
- Record all skipped environment gates explicitly.

### Wave 6: Finish And Replan

Timebox: final 45-60 minutes before 22:00.

- Update relevant architecture/spec notes when patterns changed.
- Commit only verified task-owned changes.
- Archive or leave task active with a clear next action.
- Record remaining problem pool so the next session can continue.

## OpenDAL Preliminary Decision

OpenDAL should be treated as a serious candidate for an internal adapter spike,
not as an immediate replacement for `nako-vfs`.

Reasons to evaluate:

- supports many storage services behind one Rust operator model;
- has built-in retry, timeout, tracing/metrics, throttle, and capability
  layers;
- covers local filesystem, WebDAV, S3-compatible storage, and other future
  remote storage targets.

Reasons not to replace the Nako VFS boundary:

- Nako already owns product semantics that OpenDAL does not know about:
  Source Locator redaction, Source Fingerprint evidence, storage health,
  cache repair authority, deterministic staging, library-scoped backend
  routing, and Admin-safe diagnostics.
- Current WebDAV behavior is intentionally read-only in Nako's M1 surface,
  while OpenDAL's WebDAV service advertises write/delete/copy/rename
  capabilities; Nako must still enforce its own capability policy.
- Timeout and retry layer ordering matters in OpenDAL. Nako still needs its
  own bounded runtime/resource policy around scan/probe/playback paths.

Recommendation:

- Do not add OpenDAL as a production dependency in the first implementation
  wave.
- First create `storage-opendal-adapter-decision-spike` or fold it into this
  planning task as a research/design deliverable.
- If approved after the spike, add OpenDAL behind `StorageBackend` only, with
  explicit feature flags and tests proving redaction, capability narrowing,
  range reads, and backend health semantics.

## Acceptance Criteria

- [ ] `prd.md` captures the 2026-06-07 22:00 plan and candidate problem pool.
- [ ] OpenDAL evaluation is recorded with current source links and a clear
      recommendation.
- [ ] `implement.jsonl` and `check.jsonl` contain real context entries.
- [ ] The first implementation slice is chosen before code edits begin.
- [ ] Every implemented slice has focused validation evidence.
- [ ] If production dependencies, public API, schema, or config shape change,
      the relevant ADR/architecture docs are updated before completion.

## Definition Of Done

- Task context is complete enough for implementation/check agents.
- At least one bounded development slice is either completed and verified, or
  explicitly split with next actions.
- Any code changes are formatted, tested with focused gates, and committed with
  a Conventional Commit message when verified.
- Lessons that prevent repeat mistakes are captured in Trellis spec or
  architecture docs.

## Out Of Scope

- Publishing an actual release artifact or crates.io/GHCR package without
  explicit release approval.
- Replacing `nako-vfs` wholesale with OpenDAL.
- Broad M3 playback feature expansion such as LL-HLS/CMAF, DRM, hardware
  tone-map execution, or remote transcode workers.
- Public Client metadata governance or provider mutation undo unless promoted
  by evidence.

## Technical Notes

- Local command `cargo search opendal --limit 1` reported `opendal = "0.57.0"`.
- See `research/opendal-storage-layer.md` for the OpenDAL source summary.
- Relevant local authority:
  - `docs/ROADMAP.md`
  - `docs/architecture/LANES.md`
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md`
  - `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`
