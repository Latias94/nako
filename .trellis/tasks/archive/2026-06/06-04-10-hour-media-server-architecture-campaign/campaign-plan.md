# 10-Hour Media Server Architecture Campaign Plan

Date: 2026-06-04
Mode: PLAN

## Program Action

Mode: PLAN
Now: Open a focused implementation campaign only after the user approves the
default lane mix below.
Why: Four independent read-only inspections found that Nako's best 10-hour
media-server return is not addon-runtime breadth, but safer media intake,
playback runtime reliability, artwork delivery performance, and self-hosted
diagnostic evidence.

## Evidence

Research artifacts:

- `research/library-metadata-catalog.md`
- `research/playback-transcode-streaming.md`
- `research/storage-vfs-operations.md`
- `research/addon-control-plane.md`

Architecture and spec authority:

- `CONTEXT.md`
- `docs/architecture/LANES.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/OPERATIONS_RELEASE.md`
- `.trellis/spec/nako-server/backend/directory-structure.md`
- `.trellis/spec/nako-server/backend/http-api-patterns.md`
- `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
- `.trellis/spec/nako-playback/backend/index.md`
- `.trellis/spec/nako-transcode/backend/index.md`
- `.trellis/spec/nako-vfs/backend/index.md`

## Recommended Goal

Ship a bounded "intake-to-playback reliability" campaign: copied media becomes
stable intake work instead of premature probe noise, active HLS/remux sessions
have a clearer runtime session boundary, selected artwork delivery gets a first
cache/performance slice, and self-hosted playback dependencies get smoke
evidence.

This is the default recommendation because it improves the normal media-server
loop a user feels first:

1. Add files to a Media Library.
2. Let Nako observe stable Media Sources safely.
3. Browse the resulting catalog with responsive Managed Artwork.
4. Play or retry media through Direct Play, Remux, or HLS without session
   lifecycle surprises.
5. Diagnose host playback prerequisites when FFmpeg or hardware is not ready.

## Default Parallel Lanes

### Lane A: Watch Folder Stable Intake Runtime Productization

Type: feature development with small interface deepening.

Owns:

- `crates/nako-library/src/intake.rs`
- `crates/nako-server/src/app/watch_folder_runtime.rs`
- watch-folder runtime app tests
- existing Admin overview diagnostics only if no new route contract is needed

Do not touch:

- metadata review apply routes
- artwork delivery
- playback HLS/remux modules
- Addon task runtime

10-hour slice:

- Deepen `tick_library` into a small watch-folder intake plan interface:
  discover, suppress, enqueue decision, redaction-safe summary.
- Preserve the rule that scan/probe execution goes through the existing durable
  library scan queue, not inline watcher work.
- Keep OS-native watcher daemon work out of scope; polling productization is
  enough for this campaign.

Why it belongs in the default goal:

- It is the most direct user-visible improvement from the Library/Metadata
  inspection.
- It lowers false failures during large file copies and slow local storage
  writes.
- It is highly parallel with playback and artwork work if it avoids new Admin
  API surface.

Gates:

- `cargo nextest run -p nako-library intake --no-fail-fast`
- `cargo nextest run -p nako-server watch_folder --no-fail-fast`
- `cargo nextest run -p nako-server library --no-fail-fast`
- `cargo check -p nako-library -p nako-server --tests`

Stop conditions:

- A new route or generated Admin contract becomes necessary.
- Duplicate scan jobs are enqueued under repeated stable candidates.
- Raw paths, Source Locators, fingerprints, etags, or backend URLs enter
  diagnostics.

### Lane B: Playback Transcode Runtime Session Module

Type: fearless refactor plus reliability.

Owns:

- `crates/nako-server/src/app/playback/hls_flow.rs`
- `crates/nako-server/src/app/playback/remux_flow.rs`
- focused playback runtime/session helper module
- HLS/remux app tests

Do not touch:

- FFmpeg argv construction in `nako-transcode`
- pure playback decision rules in `nako-playback`
- public DTOs, schema, or generated SDKs
- HLS artifact I/O pressure in the same edit window

10-hour slice:

- Extract a server-owned Playback Transcode Runtime session module around
  start/reuse/supersede/cancel/linkage behavior for HLS and Remux.
- Keep HLS and Remux as mode-specific adapters.
- Preserve existing lifecycle behavior while adding regression coverage for
  active-session reuse, supersede cancellation, failed startup correlation, and
  playback-session-to-transcode linkage.

Why it belongs in the default goal:

- Playback is the core media-server experience.
- This is the highest-leverage refactor found by both the old boundary review
  and the new Playback inspection.
- It prepares later remote stage/artifact pressure, seek polish, and player UX
  work without taking on a protocol change.

Gates:

- `cargo nextest run -p nako-server hls --no-fail-fast`
- `cargo nextest run -p nako-server remux --no-fail-fast`
- `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`

Stop conditions:

- The slice needs schema, public API, or durable queue semantics.
- The server module starts rebuilding FFmpeg argv or planner compatibility
  decisions.
- Direct Play -> Remux -> Transcode preference changes.

### Lane C: Selected Artwork Delivery Cache First Slice

Type: performance-oriented feature development.

Owns:

- `crates/nako-server/src/app/artwork*`
- `crates/nako-server/src/http/catalog.rs`
- selected artwork/catalog HTTP tests

Do not touch:

- metadata provider application
- catalog graph hydration behavior
- schema or binary derivative store unless explicitly approved

10-hour slice:

- Start with metadata-only ETag preflight or a small read-through derivative
  cache plan that preserves existing selected artwork ETags and private cache
  headers.
- If implementation risk is higher than expected, ship a documented derivative
  cache contract plus one focused server-side preflight optimization.
- Defer a durable binary derivative store unless cache invalidation and storage
  authority are explicitly accepted.

Why it belongs in the default goal:

- Catalog browsing feels broken when artwork is slow, even if metadata is
  correct.
- Current selected artwork routes already have ETag and variant foundations.
- This lane avoids the shared playback and metadata files.

Gates:

- `cargo nextest run -p nako-server catalog --no-fail-fast`
- `cargo nextest run -p nako-server artwork --no-fail-fast`
- `cargo check -p nako-server --tests`

Stop conditions:

- Cache invalidation cannot be expressed from selected artwork id, artifact id,
  update time, and variant dimensions.
- Persistence of derivative bytes requires new schema or unclear storage
  ownership.
- Auth or Library Access checks would be bypassed before 304/cache responses.

### Lane D: Hardware / Release Smoke Evidence

Type: operations and reliability.

Owns:

- release-gate or self-host smoke tests/scripts
- `nako-transcode` hardware diagnostics tests
- self-hosted playback smoke evidence
- release checklist notes only if the gate shape changes

Do not touch:

- executable HEVC/AV1 HLS output
- host-specific GPU requirements in normal CI
- playback session runtime modules

10-hour slice:

- Add a non-invasive smoke gate around FFmpeg/ffprobe discovery, CPU HLS
  readiness, redaction of local paths, and existing hardware report
  serialization.
- Treat real GPU devices as optional/skipped unless the runner exposes them.
- Keep this as evidence and diagnostics, not hardware feature execution.

Why it belongs in the default goal:

- Self-hosted playback only works if the deployed host can actually run FFmpeg.
- This lane is low-conflict and can run while Lane A/B/C implement code.
- It turns release/operations risk into repeatable evidence.

Gates:

- `cargo nextest run -p nako-transcode hardware --no-fail-fast`
- `cargo nextest run -p nako-server self_host_smoke --no-fail-fast`
- `cargo check -p nako-transcode -p nako-server --tests`

Stop conditions:

- CI would require specific GPU hardware or drivers.
- Diagnostics leak FFmpeg paths, local cache roots, Source Locators, or media
  names.
- The work starts promising HEVC/AV1 executable HLS output.

## 10-Hour Timeline

### Hour 0.0-0.5: Commander Preflight

- Confirm `main` baseline, dirty state, and no active implementation lane
  conflict.
- Open focused Trellis implementation tasks if the user approves the campaign.
- Freeze the shared-scope rule: only one worker may touch Admin API/generated
  contract files; default campaign avoids that shared surface.

### Hour 0.5-1.5: Micro-Design Checkpoints

- Lane A returns the `WatchFolderIntakePlan` shape and scan enqueue invariant.
- Lane B returns the Playback Transcode Runtime session interface and the
  exact HLS/remux tests it will preserve.
- Lane C returns cache invalidation facts and whether implementation is
  metadata-only, read-through cache, or plan-only.
- Lane D returns the smoke gate command shape and skip semantics.

Decision gate:

- If Lane A or C unexpectedly requires Admin API/generated contract changes,
  serialize that work under one owner or drop Lane C.
- If Lane B and a storage/playback pressure task both need `hls_flow.rs`, keep
  pressure work as a follow-on.

### Hour 1.5-6.5: Parallel Implementation

Run Lane A, B, C, and D in parallel only while their write sets remain disjoint.

Expected outputs:

- Lane A: stable intake runtime productization and focused tests.
- Lane B: Playback Transcode Runtime session extraction and focused tests.
- Lane C: selected artwork cache/preflight slice or a committed design/test
  artifact if persistence is not safe.
- Lane D: smoke evidence gate and focused tests/docs.

### Hour 6.5-8.0: Integration Window

- Integrate Lane D first because it should have low merge pressure.
- Integrate Lane C next if it stayed inside artwork/catalog files.
- Integrate Lane A and B separately; both are behavior-sensitive and should get
  focused nextest evidence before combined checks.
- Do not add a late fifth feature.

### Hour 8.0-9.5: Combined Gates

Minimum combined gate set:

- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo check -p nako-library -p nako-playback -p nako-transcode -p nako-server --tests`
- `cargo nextest run -p nako-library intake --no-fail-fast`
- `cargo nextest run -p nako-server watch_folder --no-fail-fast`
- `cargo nextest run -p nako-server hls --no-fail-fast`
- `cargo nextest run -p nako-server remux --no-fail-fast`
- `cargo nextest run -p nako-server catalog --no-fail-fast`
- `cargo nextest run -p nako-transcode hardware --no-fail-fast`
- `python ./.trellis/scripts/task.py validate ./.trellis/tasks/<implementation-task>`

### Hour 9.5-10.0: Closeout

- Update `.trellis/spec/` only for new durable contracts:
  watch-folder intake plan, Playback Transcode Runtime session interface,
  selected artwork cache contract, or release smoke policy.
- Update architecture maps only if a shipped lane changes capability status.
- Commit implementation and Trellis task/archive changes separately.
- Split anything unfinished into follow-on tasks instead of stretching scope.

## Alternatives

### If The User Wants More TV/Anime Metadata Value

Replace Lane C or D with Provider Review Related Hierarchy Admin Application.

Reason:

- The `nako-metadata` kernel already exists and is well guarded.
- It would make provider graph previews useful for series, season, and episode
  Hierarchy Confirmation.

Cost:

- It touches Admin API/server metadata DTOs and should not run beside another
  worker changing `crates/nako-server/src/app/metadata.rs` or generated
  contracts.

Gates:

- `cargo nextest run -p nako-metadata candidate_review_related_hierarchy --no-fail-fast`
- `cargo nextest run -p nako-server metadata_candidate --no-fail-fast`
- `cargo check -p nako-metadata -p nako-api -p nako-server --tests`

### If The User Wants Remote Storage Operator Repair

Replace Lane D with VFS Cache Repair Selected-Target Refresh.

Reason:

- Storage inspection found this is the most actionable operator-facing storage
  feature after target list/preview shipped.

Cost:

- It changes Admin storage route/DTO contracts and must own generated contract
  updates alone.
- It is not as central to the everyday media loop as Watch Folder or Playback.

Gates:

- `cargo check -p nako-api -p nako-server --tests`
- `cargo nextest run -p nako-server vfs_cache_repair --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_vfs_cache_repair --no-fail-fast`

### If The User Wants Lowest-Risk Refactor Only

Use this sequence:

1. Playback Transcode Runtime session Module.
2. Playback artifact cleanup service extraction.
3. `nako-catalog` hydration private module split.
4. API scale/cache audit tests.

This lowers feature risk but gives less visible product movement.

## Deferred Work

Do not include these in the default 10-hour goal:

- LL-HLS/CMAF.
- HEVC/AV1 executable HLS output.
- durable remote transcode workers.
- Douban TV/episode endpoint depth.
- full NFO episode/series/season export policy.
- Addon Manager process lifecycle.
- Generated Artifact intake convergence.
- Webhook scheduler parity if schema changes are required.
- Public Client endpoint discovery API.

## Automatic Follow-On

If the default campaign finishes early and the deadline has not arrived, keep
going with the next lowest-risk refactor lane instead of stopping.

### Lane E: Playback Artifact Cleanup Service Extraction

Type: refactor-only reliability work.

Owns:

- `crates/nako-server/src/app/startup.rs`
- a focused playback artifact cleanup helper module under `crates/nako-server`
- startup cleanup tests and any directly related app tests

Do not touch:

- HLS segment/session HTTP behavior
- playback planner rules
- schema, API contracts, or generated SDKs
- storage repair routes

10-hour slice:

- Extract playback artifact cleanup out of `startup.rs` into a reusable helper
  service with the same canonical-root and retention behavior.
- Keep startup behavior and tests intact while reducing procedural cleanup
  logic in the startup workflow.
- Add focused tests for root escape protection, missing root handling, and
  large directory summary behavior if they are not already covered.

Why it belongs in the automatic follow-on:

- It is a pure locality/depth improvement.
- The storage review identified it as a shallow startup-owned cleanup path.
- It avoids the contract and admin surface risk of the storage repair follow-on.

Gates:

- `cargo nextest run -p nako-server startup --no-fail-fast`
- `cargo check -p nako-server --tests`

Stop conditions:

- The refactor requires a new Admin route or schema change.
- The cleanup logic starts depending on playback session runtime behavior.
- The module begins duplicating logic already owned by another helper.

## Exact Worker Prompts

### Lane A Prompt

Implement Watch Folder Stable Intake Runtime productization for Nako. Own
`crates/nako-library/src/intake.rs`,
`crates/nako-server/src/app/watch_folder_runtime.rs`, and focused tests only.
Deepen `tick_library` into a redaction-safe intake plan around discover,
suppress, enqueue decision, and summary. Do not add OS watcher daemon behavior,
new Admin route contracts, schema changes, metadata apply behavior, or inline
scan/probe execution. Final response must include status
`DONE | DONE_WITH_CONCERNS | BLOCKED | NEEDS_CONTEXT`, changed files,
validation, concerns/follow-ups, and `WORKSTREAM_RESULT:`.

### Lane B Prompt

Implement a behavior-preserving Playback Transcode Runtime session module.
Own HLS/remux runtime session start/reuse/supersede/cancel/linkage code under
`crates/nako-server/src/app/playback/*` and focused tests. Keep HTTP handlers,
FFmpeg argv planning, pure playback decisions, schema, public DTOs, and
generated SDKs unchanged. Do not implement remote stage/artifact pressure in
this task. Final response must include status
`DONE | DONE_WITH_CONCERNS | BLOCKED | NEEDS_CONTEXT`, changed files,
validation, concerns/follow-ups, and `WORKSTREAM_RESULT:`.

### Lane C Prompt

Implement the smallest selected artwork delivery cache/preflight slice that
does not require schema or a durable binary derivative store. Own
`crates/nako-server/src/app/artwork*`, `crates/nako-server/src/http/catalog.rs`,
and selected artwork/catalog tests only. Preserve auth, Library Access,
selected artwork ETags, private cache headers, and variant invalidation. If
safe implementation is not clear, stop with a concrete derivative-cache design
and tests to add. Final response must include status
`DONE | DONE_WITH_CONCERNS | BLOCKED | NEEDS_CONTEXT`, changed files,
validation, concerns/follow-ups, and `WORKSTREAM_RESULT:`.

### Lane D Prompt

Add self-hosted playback smoke evidence around FFmpeg/ffprobe discovery,
CPU-HLS readiness, hardware report serialization, and redaction. Own only
release gate/smoke tests, `nako-transcode` hardware diagnostics tests, and
minimal docs/checklist updates if the gate command changes. Do not require GPU
devices in normal CI and do not implement HEVC/AV1 executable HLS output. Final
response must include status
`DONE | DONE_WITH_CONCERNS | BLOCKED | NEEDS_CONTEXT`, changed files,
validation, concerns/follow-ups, and `WORKSTREAM_RESULT:`.

### Lane E Prompt

Extract playback artifact cleanup from `crates/nako-server/src/app/startup.rs`
into a focused helper service without changing cleanup semantics. Own only the
startup cleanup implementation and the directly related tests. Do not touch
HTTP routes, playback planning, schema, or API contracts. Final response must
include status `DONE | DONE_WITH_CONCERNS | BLOCKED | NEEDS_CONTEXT`, changed
files, validation, concerns/follow-ups, and `WORKSTREAM_RESULT:`.

## Minimal User Input Needed

Choose one:

1. Approve the default campaign.
2. Swap Lane C/D for Provider Review Related Hierarchy Admin Application.
3. Swap Lane D for VFS Cache Repair Selected-Target Refresh.
4. Choose the lowest-risk refactor-only variant.
