# Library and storage follow-on parallel wave

## Goal

Coordinate the next parallel wave after wave 05 without reopening broad
architecture work. The selected wave shape is Option A: watcher runtime
productization, staging attribution persistence, and a narrow Jellyfin watcher
reference lane.

## Selected Plan

Option A is selected by the user as of 2026-06-03.

The parent task is coordination-only. Implementation and research work belongs
in the three child tasks:

* `06-03-06a-library-watcher-runtime-productization`
* `06-03-06b-storage-staging-attribution-persistence`
* `06-03-06c-targeted-jellyfin-watcher-reference`

No open product-shape question remains in this parent task. Option B
PostgreSQL suite expansion is deferred, and Option C two-lane execution is not
selected.

## Wave Rationale

Wave 05 landed scoped staging admission, queued scan fairness, PostgreSQL
storage-runtime parity harness evidence, and a `nako-library::intake`
stable-candidate evidence foundation. The next useful step is to productize the
watcher path while tightening storage attribution authority. A narrow Jellyfin
reference lane can de-risk watcher runtime decisions without becoming a broad
Jellyfin audit or blocking Nako implementation work.

## Child Lanes

### 06a: watcher runtime productization

Goal: wire the shipped stable-candidate intake seam into a supervised runtime
and scheduled reconciliation path so filesystem events can become safe,
bounded library intake work.

Scope:

* runtime lifecycle for library watcher work through existing server/control
  plane boundaries;
* event normalization, debounce/stable-candidate evaluation, and duplicate
  event coalescing;
* enqueueing or scheduling follow-up scan/intake work through existing scan
  and durable runtime authorities;
* redaction-safe watcher and intake diagnostics.

Non-goals:

* no staging attribution persistence or storage schema work;
* no Jellyfin research output beyond consuming any explicit 06c findings;
* no second scan executor, hidden background runtime, or raw `tokio::spawn`
  scheduler bypass;
* no broad remote-storage watcher commitment for backends that do not provide
  trustworthy watch events.

### 06b: staging attribution persistence

Goal: persist authoritative staging attribution so ambiguous same-root or
multi-endpoint library staging records can be reported honestly instead of
inventing false per-library ownership.

Scope:

* storage/VFS attribution authority for staging records and policy slices;
* repository/schema changes only where needed to persist attribution facts;
* SQLite/PostgreSQL contract coverage for changed persistence behavior;
* redaction-safe Admin/server diagnostics or scan-admission reads that consume
  the attribution.

Non-goals:

* no watcher runtime, filesystem event handling, or debounce behavior;
* no broad PostgreSQL runtime suite expansion beyond gates needed for changed
  attribution persistence;
* no cache repair operator workflow or source fingerprint escalation policy;
* no raw path, source locator, source fingerprint, backend credential, or host
  filesystem disclosure.

### 06c: targeted Jellyfin watcher reference

Goal: collect behavior-level reference notes from Jellyfin only where they
answer watcher/event/debounce questions that affect 06a productization.

Scope:

* narrow review of Jellyfin library monitor, file refresher, monitor delay, and
  watcher/scan coordination behavior;
* decision-oriented notes for Nako watcher lifecycle, debounce delay, event
  coalescing, suppression during planned writes, and fallback reconciliation;
* explicit licensing hygiene: cite reference paths and summarize behavior, but
  do not copy, translate, or port implementation code.

Non-goals:

* no full Jellyfin audit;
* no Jellyfin schema/API/plugin/playback comparison;
* no Nako code changes and no docs/architecture changes;
* no binding recommendation that overrides Nako's ADR 0053 control-plane
  boundary.

## Requirements

* Keep the parent task as a parent wave and not an implementation lane.
* Preserve ADR 0053 control-plane boundaries for watcher/runtime work.
* Preserve storage/VFS redaction and attribution safety.
* Keep 06a and 06b independent enough to run in parallel; coordinate only if
  one lane needs a shared schema, DTO, or runtime contract decision.
* Keep 06c as reference/research only and scoped to watcher/event/debounce
  semantics.

## Acceptance Criteria

* [x] Option A is recorded as selected.
* [x] The parent task has three child tasks.
* [x] Each child task has clear goal, scope, non-goals, and curated
      `implement.jsonl` / `check.jsonl`.
* [x] The Jellyfin lane is explicitly narrow reference/research, not a broad
      Jellyfin audit.

## Definition of Done

* Parent and child task docs are reviewable on `main`.
* No implementation code is changed by this planner setup.
* No `docs/architecture` files are changed by this planner setup.
* Trellis JSONL validation passes for the parent and child tasks.

## Out of Scope

* No implementation in this parent planning task.
* No broad Jellyfin comparison.
* No reopening already-merged wave 05 scope.
* No branch, worktree, schema, API, or architecture-doc change in this setup
  task.

## Technical Notes

* Relevant architecture docs:
  * `docs/architecture/LANES.md`
  * `docs/architecture/LIBRARY_PIPELINE.md`
  * `docs/architecture/STORAGE_VFS.md`
  * `docs/architecture/CONTROL_PLANE.md`
* Relevant recent evidence:
  * `.trellis/tasks/archive/2026-06/06-03-05-next-architecture-parallel-wave/evidence.md`
  * `.trellis/tasks/archive/2026-06/06-03-05d-library-watcher-debounce-intake-stability/evidence.md`
  * `.trellis/tasks/archive/2026-06/06-03-05a-staging-budget-per-backend-policy/evidence.md`
