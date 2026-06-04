# long horizon architecture and refactor queue

## Goal

Build and execute a durable queue of high-leverage Nako architecture and product
tasks before 2026-06-04 10:00 Asia/Shanghai. The queue must produce actual
bounded implementation work, not only planning, while preserving Trellis task
context, verification, commits, spec updates, and archive evidence.

## What I Already Know

* Current Goal is active and explicitly authorizes autonomous Trellis task
  creation, execution, checking, committing, archiving, and sub-agent use.
* `git status` is clean on `main`, currently ahead of `origin/main` by 8
  commits.
* The latest completed transcode work deepened HLS server orchestration and
  grouped HLS FFmpeg command parts with exact argv coverage.
* `docs/architecture/LANES.md` reports no active implementation lane; completed
  06-02 task directories remain evidence, not active work.
* `docs/architecture/PLAYBACK.md` identifies resource admission queueing,
  remote workers, LL-HLS/CMAF, hardware tone-map execution, HEVC/AV1 output
  policy, subtitle burn-in, and player UX as split follow-ons.
* `docs/architecture/STORAGE_VFS.md` identifies cache repair operator actions,
  source fingerprint escalation, playback artifact/source-read pressure, scan
  scheduling, and PostgreSQL runtime harness work as split follow-ons.
* `docs/architecture/CONTROL_PLANE.md` keeps ADR 0053 as the baseline for
  durable jobs, runtime supervision, trace context, diagnostics, cache
  contracts, and API scale.

## Assumptions

* "无畏重构" means making the globally correct bounded change with clear
  ownership and tests, not doing risky broad rewrites without a task contract.
* Planning artifacts should guide implementation, but every selected child task
  must have its own PRD, context JSONL, evidence, verification, commit, and
  archive.
* We should prefer tasks that improve typed boundaries, locality, product
  capability, or operational safety.

## Requirements

* Produce a prioritized bounded task queue across playback/transcode,
  storage/VFS/library, control-plane, API/client, and web-product lanes.
* For each candidate, record value, risk, write scope, verification commands,
  docs/spec impact, and whether it can run in parallel.
* Start the first implementation task immediately after enough queue evidence
  exists.
* Use sub-agents only for concrete, bounded research/check/implementation work.
* Keep implementation work in child Trellis tasks rather than this queue task
  unless the change is purely queue metadata.
* Avoid public API, schema, or ADR changes unless the selected child task
  explicitly justifies them.

## Candidate Queue

### P0 - Playback / Transcode Resource Admission Queueing

Deepen the existing resource admission policy from immediate denial for selected
pressure paths toward typed bounded queue or waitlist behavior where product
semantics allow it.

Value: improves operator-visible playback reliability and aligns with ADR 0053
control-plane expectations.

Risk: may affect public playback error/status semantics; child task must avoid
schema changes unless explicitly approved by its PRD.

Likely scope: `crates/nako-server/src/app/playback/resource.rs`,
`crates/nako-server/src/app/playback/*`, selected `nako-api` DTOs only if
needed.

### P0 - HLS Subtitle Burn-In Planning Slice

Add or deepen the typed planning seam that decides when unsupported subtitle
formats require burn-in, keeping command assembly in `nako-transcode` and
selection semantics in playback/server boundaries.

Value: closes a visible Jellyfin-class playback gap while building on the HLS
builder refactor.

Risk: subtitle capability semantics can sprawl; child task must stay to one
format/decision slice or behavior-preserving prep.

Likely scope: `crates/nako-playback`, `crates/nako-transcode`, possibly
`crates/nako-server/src/app/playback`.

### P1 - HLS Seek Restart FFmpeg Command Identity

Make seek/restart command planning more explicit around start position,
generation identity, and FFmpeg seek flags without changing public routes in the
first slice.

Value: improves a known risk register item and prepares better first-watch UX.

Risk: timestamp/keyframe behavior is subtle; exact argv tests and lifecycle tests
are mandatory.

### P1 - Storage/VFS Cache Repair Operator Action Preview

Turn existing cache repair diagnostics into a bounded preview/action seam that
operators can inspect without exposing source locators or paths.

Value: converts diagnostics into operable recovery behavior.

Risk: can cross Admin API/Web/schema boundaries; first slice should be read-only
or preview-only if schema is not needed.

### P1 - Unified Trace Context First Slice

Propagate a typed request/trace identity through one high-value path such as
playback HLS start or library scan scheduling.

Value: improves debugging and future incident bundles.

Risk: cross-cutting surface; must be tiny and redaction-safe.

## Recommended First Implementation

Start with **Playback / Transcode Resource Admission Queueing** after research
confirms the narrowest path. It is the best first task because it is explicitly
called out by playback architecture, directly improves runtime reliability, and
has a prior completed task (`06-02-03b-playback-runtime-resource-admission`) that
already mapped denial behavior and follow-ons.

## Acceptance Criteria

* [ ] Research files exist for playback/transcode and storage/library/control
  plane next candidates.
* [ ] The queue is reduced to 3-5 concrete child tasks with write scope and
  validation gates.
* [ ] At least one child task is created, implemented, checked, committed, and
  archived before this queue is considered useful.
* [ ] Queue context links to all research/spec inputs needed by child tasks.
* [ ] Any newly learned reusable convention is written to `.trellis/spec/`.

## Definition Of Done

* This task records the selected queue and child-task outcomes.
* Completed child tasks have Conventional Commit commits.
* Quality gates listed in child PRDs pass, or failures are recorded with exact
  blocker details.
* Finished tasks are archived and the developer journal is updated.

## Out Of Scope

* Wholesale rewrite of playback, transcode, storage, or control-plane crates.
* Copying, translating, or importing code, comments, tests, migrations, or
  assets from `repo-ref/`.
* Binding this repository's `main` branch into a new worktree.
* Large public API or schema changes without a dedicated child task and explicit
  task-level decision.

## Research References

* [`research/playback-transcode-next-candidates.md`](research/playback-transcode-next-candidates.md) - playback/transcode lane: recommends the HLS admission policy seam as the next implementation task.
* [`research/storage-library-control-plane-next-candidates.md`](research/storage-library-control-plane-next-candidates.md) - storage/library/control-plane lanes: recommends `vfs-cache-repair-operator-actions` as the next bounded follow-on.

## Child Task Outcomes

* `06-04-06-04-hevc-av1-hls-output-policy-first-slice` selected the next
  playback/transcode follow-on after HLS admission, cache repair preview,
  subtitle burn-in, seek identity, trace context, watcher diagnostics, and
  source fingerprint policy had already been archived. It shipped typed HLS
  output codec policy vocabulary for H264, HEVC/H265, and AV1 while keeping
  H264/AAC as the only executable HLS output.
* `06-04-06-04-playback-release-gate-mode-first-slice` shipped an explicit
  playback release-gate mode for FFmpeg/FFprobe presence, transcode HLS tests,
  and self-host playback smoke coverage while leaving GPU hardware matrix smoke
  as a follow-on.
* `06-04-06-04-playback-hls-trace-context-first-slice` shipped the first
  playback runtime propagation of the HTTP request ID: HLS playlist route
  handlers convert the typed `HttpTraceContext` into an app-layer playback trace
  context, and HLS completion outbox events include only the normalized
  `request_id`. Broader job, VFS, FFmpeg, addon, remux/direct, and library scan
  propagation remain follow-ons.
* `06-04-06-04-hls-artifact-cache-control-headers-first-slice` shipped a
  conservative HTTP cache baseline for session-scoped HLS artifacts:
  playlist and segment responses now include `Cache-Control: no-store`, while
  ETag, immutable segment caching, and token-aware cache keys remain follow-ons.
* `06-04-06-04-selected-artwork-cache-control-headers-first-slice` shipped the
  next narrow HTTP cache-contract slice for authenticated selected artwork:
  public selected artwork image GET/HEAD responses now include
  `Cache-Control: private, max-age=86400` while preserving safe ETags, content
  headers, auth/access checks, and variant behavior. Conditional GET / 304,
  immutable or shared-cache semantics, derivative cache persistence, and
  selected-artwork invalidation remain follow-ons.

## Verification Plan

Each child task must choose focused gates from this ladder:

* `cargo fmt --all -- --check`
* `git diff --check`
* `cargo check -p <crate> --tests`
* `cargo clippy -p <crate> --tests -- -D warnings` when lint risk is material
* `cargo nextest run -p <crate> <filter> --no-fail-fast`
* Broader multi-crate checks when a public contract, schema, or cross-crate seam
  changes
