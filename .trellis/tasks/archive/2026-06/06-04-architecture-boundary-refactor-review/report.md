# Architecture Boundary Refactor Review Report

## Inputs

Research artifacts:

* `research/control-plane-api-db-boundaries.md`
* `research/playback-runtime-boundaries.md`
* `research/addon-automation-metadata-boundaries.md`
* `research/local-workspace-hotspots.md`

Review stance:

* Prefer deep Modules over shallow pass-through helpers.
* Prefer deleting accidental complexity before adding new seams.
* Keep ADRs authoritative; flag conflicts instead of relitigating them silently.
* No implementation in this task.

## Consolidated Ranking

### 1. VFS cache repair target query Module

Files:

* `crates/nako-server/src/app/storage.rs`
* `crates/nako-core/src/repository/vfs.rs`
* `crates/nako-db/src/sqlite/vfs_cache.rs`
* `crates/nako-db/src/postgres/vfs_staging.rs`

Problem:

The Admin API needs unresolved, redaction-safe VFS cache repair targets, but the
current app-service code starts from raw cache failures and owns paging,
resolved filtering, HMAC target refs, repair diagnostics, and preview lookup.
That is a shallow Interface: callers ask for failures but need repair targets.

Refactor direction:

Create an internal `vfs_cache_repair` or `VfsCacheRepairTargetQuery` Module that
owns target paging, resolved filtering, HMAC refs, preview resolution, and
action-plan mapping. Keep DB rows as raw failure records for the first slice.

Why it ranks high:

Fresh pattern, small blast radius, direct follow-on from recent work, strong
redaction value. This prevents future storage repair operations from copying the
same raw-failure orchestration.

Workflow scale:

Small-to-medium Trellis implementation task.

### 2. FFmpeg input staging lease scoped Interface

Files:

* `crates/nako-server/src/app/playback/input.rs`
* `crates/nako-server/src/app/playback/hls_flow.rs`
* `crates/nako-server/src/app/playback/remux_flow.rs`

Problem:

`FfmpegInputService` hides local vs VFS-staged inputs, but callers must remember
to release staged inputs manually on every success/error path. That leaks the
Implementation and weakens locality for remote staging cleanup bugs.

Refactor direction:

Deepen the input Module with a scoped async Interface such as
`with_source_input(...)`, or a guard that requires explicit async finish. Avoid
fire-and-forget release unless a separate runtime-supervision design accepts the
risk.

Why it ranks high:

Small, local, easy to test, and likely to prevent real resource leaks in remote
playback paths.

Workflow scale:

Small fearless-refactor task.

### 3. Addon outbound call engine in `nako-addon-client`

Files:

* `crates/nako-addon-client/src/lib.rs`
* server addon runtime callers under `crates/nako-server/src/app/addons/*`

Problem:

Addon Sidecar outbound calls repeat auth headers, protocol headers, timeout
defaults, retry classification, attempt metadata, envelope parsing, schema
hooks, and redaction policy across resource, task, event, subtitle, side-effect,
and runtime calls.

Refactor direction:

Introduce an internal call engine in `nako-addon-client` with stable public
functions preserved. Split transport/call/resource/task/event/runtime/error
modules around one invariant: no token material or unsafe payload text leaks,
and every Addon Protocol call gets consistent headers, retries, and schema
validation.

Why it ranks high:

High deletion potential, internal surface, strong tests, no schema/API change.
Good first medium refactor outside the recent storage lane.

Workflow scale:

Medium fearless-refactor task.

### 4. Host-owned Library File Write Module

Files:

* `crates/nako-server/src/app/addons/library_file_write.rs`
* `crates/nako-server/src/app/addons/subtitles.rs`
* `crates/nako-server/src/app/subtitle_sidecar.rs`
* `crates/nako-server/src/app/nfo.rs`

Problem:

The current Library File Write runtime is host-owned Nako behavior but lives
under `app/addons/`. It is already used by Addon Side Effects and subtitle
import. The placement makes future NFO, subtitle, artwork sidecar, and Addon
write flows look addon-owned when the domain says Nako owns Library File Write.

Refactor direction:

Extract `LibraryFileWriteService` to a host-owned server app Module. Keep small
Addon and subtitle adapters that translate their payloads into host-owned
commands.

Why it ranks high:

Clear domain correction, aligns with Addon permission ADRs, likely no public API
or schema change.

Workflow scale:

Small-to-medium fearless-refactor task.

### 5. Playback Transcode Runtime session Module

Files:

* `crates/nako-server/src/app/playback/hls_flow.rs`
* `crates/nako-server/src/app/playback/hls.rs`
* `crates/nako-server/src/app/playback/remux_flow.rs`
* `crates/nako-server/src/app/playback/remux.rs`

Problem:

HLS and Remux both know active/latest transcode session lookup, request-key
reuse, in-flight exclusion, resource admission, wait loops, session creation,
artifact readiness, and supersede/cancel semantics. The caller cannot simply
ask for "start or reuse this Playback Transcode artifact."

Refactor direction:

Create a server-owned `PlaybackTranscodeRuntime` Module that owns shared session
lifecycle and resource admission. Keep HLS and Remux as mode-specific Adapters.

Why it ranks below smaller candidates:

Highest playback leverage, but medium blast radius. Best after source-context
and input-lease cleanup reduce flow noise.

Workflow scale:

Medium workstream or carefully scoped fearless-refactor task.

### 6. Admin route inventory single source

Files:

* `crates/nako-api/src/admin_contract.rs`
* `crates/nako-server/src/http/admin.rs`
* generated Admin TS contracts

Problem:

Admin route paths live in Axum wiring and in the generated contract inventory.
Tests catch drift after the fact, but adding a route still requires remembering
route key, path, generated contract, server wiring, and executable-action
metadata separately.

Refactor direction:

Let `nako-api` own typed route metadata and path templates. `nako-server` imports
route constants for handler registration and executable-action guidance.

Workflow scale:

Medium contract-focused task.

### 7. Storage diagnostics internal Module split

Files:

* `crates/nako-server/src/app/storage.rs`

Problem:

`StorageDiagnosticsAppService` now groups Storage Backend Health, circuit
breaker reset, staging pressure, cleanup pressure, VFS cache summary, repair
plans, target refs, and refresh actions. The Module is broad and risks low
locality as storage control-plane features keep growing.

Refactor direction:

Keep `NakoApp::storage()` stable as a facade, but split Implementation into
`storage/health.rs`, `storage/staging_pressure.rs`, and
`storage/vfs_cache_repair.rs`.

Workflow scale:

Medium internal-module refactor.

### 8. Generated Artifact Acceptance Workflow deepening

Files:

* `crates/nako-server/src/app/automation.rs`
* `crates/nako-server/src/app/metadata_application.rs`
* `crates/nako-core/src/repository/automation.rs`
* `crates/nako-metadata`
* SQLite/Postgres automation adapters

Problem:

Automation Provider configuration, Generated Artifact proposals, metadata apply
planning, recovery, bulk batches, durable execution, and outcome persistence are
too concentrated in Automation repository/app-service Interfaces. Automation
Provider and Acceptance Workflow are related but not the same Module.

Refactor direction:

Split provider configuration from Generated Artifact Acceptance Workflow. Move
metadata authority application closer to `nako-metadata`; split repository
traits and app-service Modules only in a dedicated lane.

Workflow scale:

Large fearless-refactor workstream. High value, high blast radius.

## Other Valid Opportunities

* Playback Source Context / playback-to-transcode mapping deepening.
* Playback media transport Adapter for HTTP byte response rendering.
* HLS Artifact Authority around request-key parsing and artifact readiness.
* Addon event delivery runtime around the durable outbox seam.
* Admin TypeScript contract fragment split by Admin module.
* Scan metadata acquisition adjunct Interface.

## Recommended Next Move

Choose one first candidate based on appetite:

1. **Small and safe**: FFmpeg input staging lease scoped Interface.
2. **Recent storage follow-up**: VFS cache repair target query Module.
3. **High deletion, still bounded**: Addon outbound call engine.
4. **Domain correction**: host-owned Library File Write Module.
5. **High playback leverage**: Playback Transcode Runtime session Module.

For any selected candidate, the next step is to run `fearless-refactor` and turn
it into a concrete refactor brief with scope, deletion plan, tests, and risk
plan before implementation.
