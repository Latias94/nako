# Playback Runtime, Transcode, Streaming, And VFS Playback Seam Review

## Scope

This review focuses on the Playback Runtime data flow:

```text
Media Source / Source Locator
  -> VFS storage capability and FFmpeg input staging
  -> Playback Runtime planning
  -> Remux or Playback Transcode execution
  -> HLS/Remux/Direct Play media transport
```

The current architecture is directionally strong: `nako-playback` is a pure
planner Module, `nako-transcode` owns FFmpeg command planning and transcode
artifact modeling, `nako-streaming` owns pure byte-range response planning, and
`nako-server` owns orchestration, persistence, auth, tickets, and HTTP
Adapters. The main refactor opportunities are not crate splits. They are
places where server-side Modules remain shallow: callers still need to know too
much about request identity, active/latest session lookup, VFS staging leases,
resource admission, artifact readiness, and response rendering.

## Top Opportunities

### 1. Deepen the Playback Transcode Runtime session Module

**Files**

- `crates/nako-server/src/app/playback/hls_flow.rs:404`
- `crates/nako-server/src/app/playback/hls_flow.rs:461`
- `crates/nako-server/src/app/playback/hls_flow.rs:557`
- `crates/nako-server/src/app/playback/hls.rs:143`
- `crates/nako-server/src/app/playback/hls.rs:236`
- `crates/nako-server/src/app/playback/hls.rs:377`
- `crates/nako-server/src/app/playback/remux_flow.rs:198`
- `crates/nako-server/src/app/playback/remux_flow.rs:322`
- `crates/nako-server/src/app/playback/remux_flow.rs:365`
- `crates/nako-server/src/app/playback/remux_flow.rs:430`
- `crates/nako-server/src/app/playback/remux.rs:51`
- `crates/nako-server/src/app/playback/remux.rs:107`
- `crates/nako-server/src/app/playback/remux.rs:192`

**Problem**

HLS and Remux have good local Modules, but the overall Interface is still
shallow. The caller and callee both understand active/latest
`TranscodeSessionRecord` lookup, request-key reuse, path existence checks,
resource admission policy, in-flight guards, timeout loops, cancellation, and
session state transitions.

HLS shows the clearest leakage: `hls_flow.rs` performs pre-staging admission and
ready waiting, then `hls.rs` repeats reserve/reuse/supersede/session creation
logic before executing FFmpeg. Remux has the same shape spread across
`remux_flow.rs` and `remux.rs`, with duplicate wait loops for "started" versus
"output ready".

The current Seam has less Depth than the implementation already deserves. A
caller cannot simply ask, "start or reuse this Playback Transcode artifact and
give me a ready transport target." It must participate in the runtime state
machine.

**Proposed refactor**

Create a server-owned `PlaybackTranscodeRuntime` Module that owns the shared
runtime session state machine for Remux and HLS:

- active/latest session lookup;
- request-key reuse;
- in-flight exclusion;
- output/playlist existence validation;
- supersede admission for HLS;
- `PlaybackResourceDemand` acquisition policy;
- persisted session creation and state transition;
- bounded wait for "session exposed" and "artifact ready".

Keep Remux and HLS as mode-specific Adapters behind this Module:

- Remux Adapter: output path, `TranscodeSessionKind::Remux`, readiness
  predicate, execution plan request, cancelled output semantics.
- HLS Adapter: `HlsOutputLayout`, `TranscodeSessionKind::HlsTranscode`,
  artifact readiness predicate, supersede policy, execution plan request.

Do not move repository persistence into `nako-transcode`. The server owns the
runtime session store because the spec says transcode persistence is an Adapter
boundary. `nako-transcode` should stay focused on FFmpeg planning, execution
requests, engine Adapters, Hardware Capability Report, and artifact model
types.

**Deletion/deepening angle**

This should delete or collapse:

- `prepare_hls_source_before_input_staging`;
- duplicated active/latest lookup blocks in `hls_flow.rs` and `hls.rs`;
- separate Remux "wait for started" and "wait for output" loops where one
  generic ready-state waiter can handle both;
- local `HlsRequestAdmission` and `RemuxRequestAdmission` duplication, replaced
  by a typed shared admission outcome.

The deepened Interface becomes smaller: the flow layer supplies a planned
artifact request, and the runtime Module returns `ReuseExisting`, `Started`, or
`ReadyArtifact` with redaction-safe failure behavior.

**Test impact**

- Preserve existing HTTP route tests in `crates/nako-server/src/http/tests/playback.rs`.
- Add focused app-runtime tests for the new Module:
  - finished artifact reuse;
  - active duplicate rejection or waiting behavior;
  - HLS supersede path;
  - resource permit release on success and failure;
  - timeout mapping.
- Existing `nako-transcode` command planning tests should not need changes.

**Risk/ADR conflicts**

No ADR conflict if this stays server-owned. It aligns with ADR 0053 because
important background work remains inside explicit runtime supervision and
resource policy. It also respects ADR 0052 if FFmpeg command planning and
engine execution stay in `nako-transcode`.

The risk is over-generalizing Remux and HLS too early. Avoid a generic
"transcode everything" abstraction. The shared Module should own only the
runtime session lifecycle; mode-specific Adapters should keep real differences.

**Suggested workflow scale**

Medium workstream. This is the highest-leverage refactor and should be planned
as a focused `fearless-refactor` task before more HLS seek, ABR, or remote
worker work lands.

### 2. Deepen Playback Source Context construction and playback-to-transcode mapping

**Files**

- `crates/nako-server/src/app/playback/mod.rs:671`
- `crates/nako-server/src/app/playback/mod.rs:873`
- `crates/nako-server/src/app/playback/hls_flow.rs:283`
- `crates/nako-server/src/app/playback/remux_flow.rs:485`
- `crates/nako-server/src/app/playback/selection.rs:27`
- `crates/nako-server/src/app/playback/selection.rs:88`
- `crates/nako-server/src/playback_mapping.rs:19`
- `crates/nako-playback/src/lib.rs:214`
- `crates/nako-transcode/src/pipeline.rs`

**Problem**

HLS and Remux each build a similar source context:

- load `MediaSource`;
- load `MediaProbeResult`;
- resolve VFS `StorageUri` and backend;
- derive `PlaybackSelectionContext`;
- build `PlaybackTargetProfile`;
- resolve `EffectivePlaybackPolicy`;
- run `PlaybackPlanner`;
- ensure the decision is allowed;
- convert playback-owned facts into transcode-owned facts;
- bind request identity to the Source Locator / Source Variant facts;
- calculate staging layout.

This is a shallow Interface because the flow Modules still know the full
cross-layer data flow. It also concentrates a lot of playback-to-transcode
mapping in server glue. The mapping is not wrong, but the Locality is weak:
Color Pipeline Requirement, audio output, HLS output, track selection, and
selected stream facts are split across `selection.rs`, `playback_mapping.rs`,
and each mode flow.

**Proposed refactor**

Introduce a server-owned `PlaybackSourceContextBuilder` or
`PlaybackRuntimePlanContext` Module with mode-specific constructors:

- `for_decision(source_id, principal, client)`;
- `for_remux(source_id, client, requested_container, effective_policy)`;
- `for_hls(source_id, client, preferences, generation, effective_policy)`;
- `for_direct(source_id, range_request)`.

This Module should return a typed context that already contains:

- source and probe;
- VFS uri/backend and remote-input flag;
- effective policy and `PlaybackDecision`;
- `PlaybackTargetProfile`;
- mode-specific runtime request input such as `HlsRuntimePlanRequest` or
  `PlaybackRemuxProfileRequest`.

Then deepen `playback_mapping.rs` into the single Adapter from
`nako-playback` value types to `nako-transcode` value types. Today `selection.rs`
owns part of that Adapter by rebuilding `MediaStreamInfo` from
`TranscodeRequirementStream`; that should move into the mapping Module or into
a transcode-owned request builder that accepts playback requirements without
server reassembling every stream fact.

**Deletion/deepening angle**

This can delete duplicated `hls_source_context` and `remux_source_context`
setup code, and remove mode flows' need to know how Playback Runtime facts
become Playback Transcode facts.

The deepened Interface gives callers Leverage: mode flows ask for a ready
context, not for every upstream fact and mapping step.

**Test impact**

- Keep `hls_runtime_plan_request_uses_transcode_requirement_stream_facts`.
- Add equivalent Remux profile identity coverage.
- Add one app-context test proving Direct Play, Remux, and HLS all derive
  remote VFS facts through the same context Module.
- Existing playback planner tests in `nako-playback` should remain stable.

**Risk/ADR conflicts**

No ADR conflict. This strengthens ADR 0038/0044/0045 by making the
playback-to-transcode Interface more explicit. The main risk is accidentally
adding a dependency from `nako-playback` to `nako-transcode`; avoid that unless
a future ADR blesses it. The safer first step is server-local deepening.

**Suggested workflow scale**

Small-to-medium refactor. It can precede opportunity 1 because it reduces the
noise in HLS/Remux flow code.

### 3. Make FFmpeg input staging lease handling a deep Module

**Files**

- `crates/nako-server/src/app/playback/input.rs:16`
- `crates/nako-server/src/app/playback/input.rs:51`
- `crates/nako-server/src/app/playback/input.rs:108`
- `crates/nako-server/src/app/playback/hls_flow.rs:361`
- `crates/nako-server/src/app/playback/remux_flow.rs:280`
- `crates/nako-vfs/src/lib.rs:867`
- `crates/nako-vfs/src/webdav.rs:441`

**Problem**

`FfmpegInputService` already hides whether a Media Source can be used through a
local path hint or must be staged from VFS. That is the right Seam. But the
current Interface returns `FfmpegSourceInput` and requires callers to remember
to call `release_source_input` after the FFmpeg workflow finishes or errors.

Both HLS and Remux have manual success/error release blocks. This gives callers
too much Implementation knowledge: they need to know that a source input may
own a staging lease, and they must preserve release semantics around every
future path. That weakens Locality for remote staging bugs.

**Proposed refactor**

Deepen `FfmpegInputService` with a scoped execution Interface, for example:

```text
with_source_input(source, uri, backend, async |input| -> Result<T>)
```

or a `FfmpegSourceInputLease` that exposes only the local input path and owns a
single explicit `finish` operation. Because async release cannot safely happen
in plain `Drop`, prefer a scoped async helper or a guard that schedules bounded
release through `RuntimeSupervisor` only if a future design accepts that trade.

The `HlsAppService` and `RemuxAppService` should receive an input path only
inside the scope, while the lease release invariant stays local to the input
Module.

**Deletion/deepening angle**

Delete the repeated release match blocks in `hls_flow.rs` and `remux_flow.rs`.
The input Module becomes deep: callers get a local FFmpeg path, while VFS
staging, manifest recording, staging lease acquisition, and release stay behind
one Interface.

**Test impact**

- Add unit tests with a fake staging manifest repository:
  - local-path source does not acquire a lease;
  - remote-staged source releases on success;
  - remote-staged source releases on error;
  - cancellation/error path does not silently leak a lease.
- Existing VFS staging tests should remain unchanged.

**Risk/ADR conflicts**

No ADR conflict. This aligns with ADR 0017's staging-manifest requirement and
ADR 0053's control-plane rule for supervised runtime work. The risk is using a
fire-and-forget release path and hiding release failures. Prefer explicit async
scope first.

**Suggested workflow scale**

Small refactor. Good candidate for an immediate fearless cleanup after this
review because the change is local and has clear tests.

### 4. Introduce a server-side Playback media transport Adapter

**Files**

- `crates/nako-streaming/src/direct.rs:49`
- `crates/nako-server/src/app/playback/direct.rs:94`
- `crates/nako-server/src/app/playback/mod.rs:912`
- `crates/nako-server/src/app/playback/remux_flow.rs:543`
- `crates/nako-server/src/app/playback/hls_artifact.rs:100`
- `crates/nako-server/src/http/playback.rs:1225`
- `crates/nako-server/src/http/playback.rs:1240`
- `crates/nako-server/src/http/playback.rs:1280`
- `crates/nako-server/src/http/playback.rs:1310`
- `crates/nako-server/src/http/playback.rs:1351`

**Problem**

`nako-streaming` is usefully pure: it parses and resolves ranges and returns a
`DirectPlayResponsePlan`. The shallow part is the HTTP Adapter in
`http/playback.rs`. The route layer repeatedly handles:

- HTTP range parsing;
- `RangeNotSatisfiable` response shape;
- local file seeking;
- VFS stream body conversion;
- `PlaybackSessionId` response header;
- no-store cache headers for Direct Play/Remux;
- HLS artifact cache headers;
- path/uri display strings for storage errors.

This logic is not auth policy, and it is not route inventory. It is media-byte
transport rendering. Keeping it in route functions makes the route Interface
wide and makes future cache/ETag work harder.

**Proposed refactor**

Add a server HTTP-layer `PlaybackMediaTransportAdapter` Module that converts
typed app outputs into `axum::Response`:

- `DirectPlaySourceBody + DirectPlayResponsePlan`;
- `RemuxPlaybackStreamOutput`;
- `HlsSegmentPlan`;
- empty/preflight response plans.

Do not move Axum types into `nako-streaming`; that crate should keep its current
pure Interface. The Adapter lives in `nako-server/src/http/playback_transport.rs`
or similar and is the only Module that knows how a range plan becomes headers
and body bytes.

**Deletion/deepening angle**

Route functions shrink to auth/ticket resolution plus one Adapter call. This
improves Locality for future media-byte cache contracts, ETags, conditional
GETs, and renderer transport quirks.

**Test impact**

- Keep existing route tests.
- Add small Adapter tests for:
  - 200/206/416 status and headers;
  - no-store versus HLS artifact cache headers;
  - session header insertion;
  - local file seek and remote stream body path.

**Risk/ADR conflicts**

No ADR conflict if auth, ticket validation, and source access checks remain in
route/app code. The risk is making the Adapter too powerful and smuggling auth
decisions into transport rendering. Keep it as an HTTP Adapter only.

**Suggested workflow scale**

Small-to-medium refactor. Good follow-up before broader API cache/ETag work.

### 5. Separate HLS Artifact Authority from persisted request-key parsing

**Files**

- `crates/nako-server/src/app/playback/hls_artifact.rs:23`
- `crates/nako-server/src/app/playback/hls_artifact.rs:100`
- `crates/nako-server/src/app/playback/hls_artifact.rs:163`
- `crates/nako-server/src/app/playback/staging_policy.rs:61`
- `crates/nako-transcode/src/artifact.rs:980`
- `crates/nako-transcode/src/artifact.rs:1168`
- `crates/nako-transcode/src/artifact.rs:1330`
- `crates/nako-transcode/src/profile.rs:483`

**Problem**

HLS artifact modeling is strong in `nako-transcode`, but serving artifacts
currently depends on reconstructing `HlsArtifactSpec` from the persisted
request key. The server `HlsArtifactService` combines artifact authority,
readiness, cleanup policy, playlist rewriting, file metadata reads, and segment
planning.

This is working, but the Interface is becoming shallow as HLS grows. Future
LL-HLS/CMAF, immutable caching, token-aware cache keys, partial playlist
readiness, ABR pruning, and per-artifact I/O pressure will all need a stronger
artifact authority. Relying on opaque request-key parsing for every serve path
keeps important HLS shape information implicit.

**Proposed refactor**

Short-term: deepen server-side `HlsArtifactService` into an
`HlsArtifactAuthority` Module with explicit methods:

- `artifact_set_for_session(session)`;
- `playlist_readiness(session)`;
- `playback_playlist(session, playback_session_id, transport_query)`;
- `segment(session, segment_name)`;
- `cleanup_candidates(manifest, requested_artifact)`.

The Module should keep all request-key reconstruction and manifest validation
inside one Interface.

Long-term: consider persisting a typed `TranscodeArtifactSet` or artifact
manifest snapshot with the transcode session. That would reduce reliance on
request-key parsing, but it likely needs schema/API planning and should not be
smuggled into a small cleanup.

**Deletion/deepening angle**

The first slice mostly deepens rather than deletes. It concentrates request-key
parsing, artifact readiness, and cleanup logic into one Module so route and HLS
flow code no longer need to understand artifact reconstruction.

The larger slice can delete string-reconstruction pressure if typed artifact
state is persisted.

**Test impact**

- Existing `hls_artifact.rs` tests are good leverage and should move with the
  Module.
- Add tests for malformed/stale request keys, running playlist readiness,
  adaptive fMP4 artifact lookups, sidecar subtitle/audio artifacts, and cleanup
  candidates.
- If typed persistence is added later, add repository contract tests.

**Risk/ADR conflicts**

No conflict for the short-term server Module. A typed persisted artifact
manifest may cross schema and public diagnostics surfaces, so it should be a
separate Trellis task. It aligns with ADR 0052's media-engine boundary and ADR
0053's cache/diagnostics discipline if diagnostics stay redacted.

**Suggested workflow scale**

Small for server-local Authority deepening. Medium-to-large if typed artifact
persistence is introduced.

## Priority Ranking

1. **Playback Transcode Runtime session Module** - highest Leverage and highest
   risk reduction before future HLS seek, ABR, remote worker, and resource
   scheduling work.
2. **Playback Source Context construction and playback-to-transcode mapping** -
   good precursor to the runtime refactor because it improves Locality and
   removes cross-layer setup duplication.
3. **FFmpeg input staging lease handling** - small, concrete, and likely to
   prevent remote-staging cleanup bugs.
4. **Playback media transport Adapter** - valuable before cache/ETag and
   renderer/client transport work.
5. **HLS Artifact Authority** - important for future HLS complexity, but the
   short-term payoff is lower unless a follow-on HLS artifact/cache lane is
   already planned.
