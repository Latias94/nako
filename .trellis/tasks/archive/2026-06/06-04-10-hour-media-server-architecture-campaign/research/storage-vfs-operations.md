# Storage / VFS / Operations Architecture Campaign Research

Date: 2026-06-04
Research lane: Storage / VFS / Operations
Mode: read-only inspection

## Scope

This note inspects the Storage/VFS/Operations slice for a hypothetical 10-hour
Nako self-hosted media-server improvement campaign. It focuses on work that can
increase operator value and runtime reliability without reopening Extism,
Addon ABI, public API, or schema decisions.

Relevant authority:

- `CONTEXT.md`: Media Library, Media Source, Source Locator, Source Fingerprint,
  Playback Source Selection, Nako-Managed Artifact, Library File Write.
- `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md:35`: remote storage
  belongs to the `storage-vfs` lane.
- `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md:54`: probe and FFmpeg
  workflows require explicit staging when a backend cannot provide a local path.
- `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md:63`: remote Direct
  Play should prefer range streaming through VFS.
- `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md:47`:
  staging manifest and cleanup boundary are required before expanding remote
  transcode use.
- `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md:53`:
  playback-facing storage errors should become typed and redaction-safe.
- `docs/adr/0053-application-control-plane-boundary.md:65`: long-running or
  important background work belongs in durable jobs or supervised runtimes.
- `docs/adr/0053-application-control-plane-boundary.md:73`: diagnostics must be
  operator-useful and redacted.
- `docs/architecture/LANES.md:316`: `storage-vfs` owns source locators, source
  identity, VFS backends, remote storage behavior, staging/cache diagnostics,
  and storage failure classification.
- `docs/architecture/LANES.md:329`: storage-vfs shares library scan/probe,
  playback input staging, metadata/NFO sidecar write policy, and database source
  identity projections.

## Current Shape

- VFS contracts are already deep enough for staged work:
  `crates/nako-vfs/src/lib.rs:748` defines `StorageBackend`;
  `crates/nako-vfs/src/lib.rs:777` and `:784` define optional in-process and
  streaming range reads; `:867` defines local staging.
- WebDAV already supports streaming and staging:
  `crates/nako-vfs/src/webdav.rs:360` materializes `read_range`,
  `:393` exposes `stream_range`, and `:441` stages a remote object to a
  deterministic local path.
- Direct Play is already memory-bounded for remote bodies:
  `crates/nako-server/src/app/playback/direct.rs:119` uses
  `backend.stream_range`, and `crates/nako-server/src/app/playback/mod.rs:880`
  acquires a remote stream permit for non-local sources.
- Staging manifest persistence and budget reservation exist:
  `crates/nako-server/src/app/staging.rs:153` wraps a backend in
  `ManifestRecordingStorageBackend`; `:344` acquires a stage permit before
  staging; SQLite and PostgreSQL both implement
  `reserve_staging_manifest_record` at
  `crates/nako-db/src/sqlite/staging.rs:70` and
  `crates/nako-db/src/postgres/vfs_staging.rs:314`.
- FFmpeg input lease scope has just been deepened:
  `crates/nako-server/src/app/playback/input.rs:56` exposes
  `with_source_input`; `:68` exposes a prepared source input scope for HLS
  background start; `:153` centralizes release behavior.
- Storage backend health and circuit breaker behavior is already wrapped around
  all backend operations:
  `crates/nako-server/src/app/storage.rs:1474` implements `StorageBackend` for
  `LibraryStorageBackend`; `:1645` checks durable/process-local backoff.
- Admin diagnostics exist but are not fully actionable:
  `crates/nako-server/src/http/admin.rs:1795` exposes storage staging
  diagnostics; `:1893` lists VFS cache repair targets; `:1912` previews one
  target; `:1878` can only refresh the latest repair target.
- Startup cleanup is present but still procedural in the startup workflow:
  `crates/nako-server/src/app/startup.rs:199` cleans staging inputs;
  `:226` cleans playback artifacts; `:304` and `:385` perform artifact cleanup
  traversal.

## Ranked Opportunities

### 1. VFS Cache Repair Selected-Target Actions

Problem:
Admin can list and preview repair targets, but selected-target execution is
explicitly not available yet. The only mutation is latest-failure refresh.

Evidence:

- `docs/architecture/STORAGE_VFS.md:101`: target-scoped preview is intentionally
  non-mutating.
- `docs/architecture/STORAGE_VFS.md:110`: selected-target refresh execution is
  the named next lane.
- `crates/nako-server/src/app/storage.rs:459`: target list API service.
- `crates/nako-server/src/app/storage.rs:504`: target preview API service.
- `crates/nako-server/src/app/storage.rs:549`: latest-only refresh action.
- `crates/nako-server/src/http/admin.rs:1878`: latest-only POST route.
- `crates/nako-server/src/http/tests/system.rs:5621`: system test proves
  targets list/preview are redacted and non-mutating.

User-visible value:
Operators can repair the exact stale or failed VFS cache target they selected,
instead of being limited to whichever failure is currently latest.

Risk:
Medium. Must preserve target_ref opacity, backend authority matching,
redaction, and non-ambiguous backend selection. Avoid broad purge/delete in the
first implementation.

Parallelizability:
Medium. Design and tests can be prepared in parallel with UI/API contract
review, but mutation code should be serial because it touches route shape,
admin DTO, and storage authority.

Serial dependencies:
Must land before durable repair queues or multi-action remediation. The target
execution route should still reuse `backend_for_vfs_cache_failure_authority`
logic instead of inventing a second selector.

Likely tests/gates:

- App test: target-scoped refresh uses the matching authority and backend.
- App test: mismatched/ambiguous target_ref does not call backend.
- HTTP test: selected target POST redacts URI/path/token and resolves preview.
- Contract test/update if Admin DTO/route inventory changes.
- `cargo check -p nako-api -p nako-server --tests`
- `cargo nextest run -p nako-server vfs_cache_repair --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_vfs_cache_repair --no-fail-fast`

### 2. Staging Cleanup Module And Operator Action Boundary

Problem:
Staging cleanup exists, but the execution is a startup helper rather than a
deep module with an explicit operator action interface, preview, and bounded
manual trigger.

Evidence:

- `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md:47`:
  staging manifest and cleanup boundary are required.
- `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md:50`:
  cleanup must enforce disk budget and run on startup/background task.
- `docs/architecture/STORAGE_VFS.md:156`: staging manifests should be
  authoritative and startup cleanup should run.
- `crates/nako-server/src/app/staging.rs:75`: cleanup loops over cleanup
  candidates and deletes files.
- `crates/nako-server/src/app/startup.rs:199`: startup calls cleanup directly.
- `crates/nako-server/src/http/admin.rs:1795`: Admin staging diagnostics expose
  cleanup pressure but no cleanup action.
- `crates/nako-db/src/sqlite/staging.rs:431` and
  `crates/nako-db/src/postgres/vfs_staging.rs:673`: cleanup candidates are
  repository-backed.

User-visible value:
Operators can clear expired staging pressure without restarting the server,
and diagnostics can explain what would be deleted before mutation.

Risk:
Medium. File deletion must stay under recorded staging paths and should not
delete actively leased inputs. Do not add broad manual purge without preview.

Parallelizability:
High for read-only preview and test fixture work; medium for mutation route.

Serial dependencies:
Should follow or share design vocabulary with selected-target VFS cache repair:
both are storage operator actions and should present similar readiness/boundary
semantics.

Likely tests/gates:

- App test: preview counts expired/unleased candidates without deleting.
- App test: execute cleanup expires, deletes, marks deleted, skips active
  leases.
- HTTP test: Admin cleanup action redacts local paths.
- PostgreSQL harness if repository query shape changes.
- `cargo check -p nako-core -p nako-db -p nako-server --tests`
- `cargo nextest run -p nako-server staging --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_storage_staging --no-fail-fast`

### 3. Playback Artifact Cleanup Service Extraction

Problem:
Playback artifact cleanup is implemented inside startup and uses synchronous
filesystem traversal/removal from an async workflow. That is workable for small
trees but shallow: startup owns too much artifact policy and cleanup cannot be
reused by Admin diagnostics or future background cleanup.

Evidence:

- `docs/architecture/CONTROL_PLANE.md:270`: HLS artifacts are session/ticket
  scoped and cache behavior remains conservative.
- `docs/architecture/STORAGE_VFS.md:27`: playback artifact I/O pressure is a
  named follow-on.
- `crates/nako-server/src/app/startup.rs:226`: startup triggers playback
  artifact cleanup.
- `crates/nako-server/src/app/startup.rs:256`: cleanup scans terminal transcode
  sessions.
- `crates/nako-server/src/app/startup.rs:304`: per-session artifact cleanup.
- `crates/nako-server/src/app/startup.rs:346`: deletes artifact directories
  synchronously.
- `crates/nako-server/src/app/startup.rs:385`: recursively summarizes artifact
  paths synchronously.
- `crates/nako-server/src/app/tests/startup.rs:2781`: startup cleanup behavior
  is already covered.

User-visible value:
Faster, safer startup on hosts with large HLS/remux artifact directories, and a
clean path toward Admin-triggered playback artifact cleanup.

Risk:
Low to medium. The behavior is local and test-covered, but cleanup security
must preserve canonical-root checks.

Parallelizability:
High. One worker can extract the service and keep current startup behavior;
another can inspect future Admin/read-only diagnostics without touching the
same files.

Serial dependencies:
Extract service before adding any route or background task. Keep public API
unchanged in the first slice.

Likely tests/gates:

- Existing startup cleanup tests unchanged.
- New service tests for root escape, missing root, large directory summary, and
  retention cutoff.
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server startup --no-fail-fast`

### 4. Remote Stage Pressure Enforcement For Playback Start

Problem:
Remote stage capacity is partly modelled twice: playback resource demand treats
`remote_stage` as host-owned capacity, while actual stage concurrency is
enforced by `LibraryStorageBackend` stage permits during staging. Staging
manifest pressure also blocks library scan, but playback HLS/remux start does
not get an explicit preflight staging-pressure decision.

Evidence:

- `crates/nako-server/src/app/playback/resource.rs:92`: Remux demand records
  remote stage as host-owned when input is remote.
- `crates/nako-server/src/app/playback/resource.rs:108`: HLS demand records
  remote stage as host-owned.
- `crates/nako-server/src/app/storage.rs:1210`: `LibraryStorageBackend` owns
  per-library stage permits.
- `crates/nako-server/src/app/storage.rs:1256`: library scan only checks
  backend backoff.
- `crates/nako-server/src/app/storage.rs:774`: scan admission also checks
  staging pressure.
- `crates/nako-server/src/app/storage.rs:1007`: policy status is computed from
  configured bytes and manifest usage.
- `crates/nako-server/src/app/staging.rs:344`: actual stage permit is acquired
  inside the manifest recording backend.

User-visible value:
Playback start can fail fast with a clear redacted "remote stage pressure" or
"stage capacity busy" reason instead of discovering it deep inside staging.

Risk:
Medium. Must avoid double-acquiring permits or changing HLS synchronous staging
semantics. Do not block local playback.

Parallelizability:
Medium. One worker can map demand/diagnostics, but implementation should be
serial with playback-transcode because it touches HLS/remux start behavior.

Serial dependencies:
Should come after the FFmpeg input lease scope refactor that just landed, and
before broader playback artifact/source-read pressure work.

Likely tests/gates:

- App tests for remote remux/HLS rejection on exhausted staging bytes.
- App tests for local remux/HLS unaffected.
- HTTP tests for redaction in error response.
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server ffmpeg_input --no-fail-fast`
- `cargo nextest run -p nako-server remux --no-fail-fast`
- `cargo nextest run -p nako-server hls_source --no-fail-fast`

### 5. Storage Diagnostics Internal Split

Problem:
`crates/nako-server/src/app/storage.rs` is a wide module that owns diagnostics,
target_ref HMAC, staging pressure summaries, repair action planning, registry,
health, and backend wrapper behavior. The module has depth in places, but too
many unrelated interfaces live in one file.

Evidence:

- `crates/nako-server/src/app/storage.rs:23`: storage diagnostics import set is
  already broad across admin DTOs, VFS, DB, crypto, staging, and health.
- `crates/nako-server/src/app/storage.rs:40`: `StorageDiagnosticsAppService`.
- `crates/nako-server/src/app/storage.rs:459`: VFS target list logic.
- `crates/nako-server/src/app/storage.rs:722`: staging budget policy surface.
- `crates/nako-server/src/app/storage.rs:749`: `StorageBackendRegistry`.
- `crates/nako-server/src/app/storage.rs:1196`: `LibraryStorageBackend`.

User-visible value:
Mostly indirect: faster, safer future improvements to storage repair, staging
pressure, and backend health. This is a reliability enabler, not a feature by
itself.

Risk:
Low if it is a pure move/refactor with tests unchanged. Higher if behavior is
changed in the same pass.

Parallelizability:
Low during edits because many storage tasks touch the same file. High as a
preparatory task before assigning parallel implementation lanes.

Serial dependencies:
Best as a first or last cleanup commit around storage work, not concurrent with
selected-target repair mutation.

Likely tests/gates:

- No new behavior tests required for pure move.
- `cargo check -p nako-server --tests`
- Focused existing storage/system tests.

### 6. PostgreSQL Runtime Harness For Staging And VFS Cache Repair

Problem:
SQLite and PostgreSQL implementations are paired for staging and health, but
many app tests use SQLite in-memory. Storage/VFS follow-ons that rely on leases,
cleanup candidates, and repair failure authority should keep PostgreSQL parity
visible.

Evidence:

- `docs/architecture/STORAGE_VFS.md:27`: PostgreSQL runtime harness work is a
  named follow-on.
- `docs/architecture/OPERATIONS_RELEASE.md:26`: PostgreSQL contract harness is
  a shipped foundation.
- `crates/nako-db/src/sqlite/staging.rs:70`: SQLite reservation logic.
- `crates/nako-db/src/postgres/vfs_staging.rs:314`: PostgreSQL reservation
  logic with `FOR UPDATE`.
- `crates/nako-db/src/sqlite/vfs_health.rs:25` and
  `crates/nako-db/src/postgres/vfs_health.rs:28`: paired health repositories.

User-visible value:
Reduces production risk for users running PostgreSQL, especially around remote
staging cleanup and repair actions.

Risk:
Low to medium. Mostly gate/harness work, but may reveal adapter divergence.

Parallelizability:
High as a validation lane. It should not edit the same server files as
feature workers unless a parity bug is found.

Serial dependencies:
Run after any repository contract changes; can run concurrently with pure app
module refactors.

Likely tests/gates:

- `scripts/postgres-contract-harness.*` focused on staging/VFS if available.
- `cargo nextest run -p nako-db staging --no-fail-fast`
- `cargo nextest run -p nako-db vfs --no-fail-fast`

### 7. OS-Mount Blocking Local Backend Audit

Problem:
The architecture warns that SMB/NFS/rclone mounts may look local but behave
like remote services. Local filesystem adapters and startup cleanup still use
local filesystem operations directly in several places. A first implementation
slice should be an audit and bounded-wrapper plan, not a broad rewrite.

Evidence:

- `docs/architecture/STORAGE_VFS.md:126`: OS mounts can block like local files.
- `docs/architecture/STORAGE_VFS.md:131`: mitigation is bounded permits and
  timeout wrappers around probe/stage workflows.
- `crates/nako-vfs/src/local.rs:179`: local `read_range`.
- `crates/nako-vfs/src/local.rs:308`: local staging.
- `crates/nako-server/src/app/startup.rs:385`: synchronous artifact traversal.
- `crates/nako-server/src/app/playback/input.rs:164`: local path metadata can
  be read when metadata length is missing.

User-visible value:
Reduces hangs for NAS-mounted libraries and makes remote-like local storage
failures diagnosable.

Risk:
Medium to high if implemented broadly. Keep first slice to audit + one bounded
critical path.

Parallelizability:
High for read-only audit; low for implementation because it can affect scan,
probe, playback, and startup behavior.

Serial dependencies:
Should follow storage diagnostics split or be a dedicated design task with ADR
or spec update if it changes local backend guarantees.

Likely tests/gates:

- Targeted tests using a blocking/failing test backend where possible.
- No global timeout wrapper without deterministic tests.
- `cargo check -p nako-vfs -p nako-server --tests`

## 10-Hour Campaign Shape

Recommended mode: PLAN, not ASSIGN-to-code yet, unless at least four research
lanes report. This storage lane is implementation-ready for two narrow slices,
but the overall media-server campaign still needs Playback/Transcode,
Library/Metadata/Catalog, Addon/Automation/Control-Plane, and API/Admin/Web
parallel research before choosing the full 10-hour goal.

### First 90 Minutes: Parallel Read-Only Recon

Run four workers in parallel:

- Storage/VFS/Operations: this artifact.
- Playback/Transcode/Streaming: inspect resource admission, HLS/remux lifecycle,
  artifact I/O pressure, player-visible failure modes.
- Library/Metadata/Catalog/NFO: inspect scan scheduling, source fingerprint
  escalation, provider governance, catalog projection scale.
- Addon/Automation/Control Plane/API: inspect durable jobs, trace context,
  Admin API scale, operations surfaces.

Stop condition:
Do not start implementation if two lanes need the same API/schema/route files
or contradict ADR 0016/0017/0053.

### Hours 1.5-2.0: Planner Merge

Rank by:

- visible user value;
- risk reduction for self-hosted operation;
- bounded implementation surface;
- testability through existing app/http tests;
- no schema/API churn unless explicitly accepted.

For this lane, the top implementation candidate is selected-target VFS cache
repair. The top refactor-only candidate is playback artifact cleanup service
extraction.

### Hours 2.0-6.0: Parallel Implementation Wave 1

Safe pair if the merged plan agrees:

- Worker A: VFS cache selected-target refresh execution.
- Worker B: Playback artifact cleanup service extraction.

These touch mostly separate files:

- Worker A: `storage.rs`, `http/admin.rs`, `nako-api/src/admin/storage.rs`,
  storage/system tests.
- Worker B: startup/playback artifact cleanup module and startup tests.

Serial gate:
Worker A must own Admin DTO/route changes. No other worker should edit
Admin storage DTOs at the same time.

### Hours 6.0-8.0: Parallel Implementation Wave 2

Choose one based on remaining time:

- If VFS selected-target execution is green: add Admin/Web-ready action metadata
  and generated contract sync.
- If artifact cleanup extraction is green: add read-only cleanup preview or
  diagnostics only, not mutation.
- If playback research finds immediate reliability risk: implement remote stage
  pressure preflight for HLS/remux instead of UI/contract polish.

### Hours 8.0-10.0: Integration, Gates, Spec

Required gates:

- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo check -p nako-core -p nako-vfs -p nako-api -p nako-db -p nako-server --tests`
- `cargo nextest run -p nako-server storage --no-fail-fast`
- `cargo nextest run -p nako-server startup --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_vfs_cache_repair --no-fail-fast`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-10-hour-media-server-architecture-campaign`

Spec/doc updates likely required:

- `.trellis/spec/nako-vfs/backend/quality-guidelines.md` if repair action or
  cleanup action semantics change.
- `.trellis/spec/nako-server/backend/http-api-patterns.md` if Admin route or
  response contract changes.
- `docs/architecture/STORAGE_VFS.md` if selected-target execution or cleanup
  action ships.
- `docs/architecture/CONTROL_PLANE.md` if any background/durable cleanup queue
  is introduced.

## Recommended Next Decision

If the broader campaign research confirms no higher-value playback or library
lane, set the first 10-hour goal to:

> Ship selected-target VFS cache refresh execution and extract playback artifact
> cleanup into a reusable service, with Admin/storage diagnostics preserved and
> no schema changes.

This gives one operator-visible storage repair feature plus one reliability
refactor that prepares future cleanup/pressure work.
