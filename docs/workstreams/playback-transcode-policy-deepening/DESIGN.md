# Playback Transcode Policy Deepening Design

Status: Active
Last updated: 2026-05-27

## Problem

Nako now has a much better playback domain than before: Playback Session is the
user/client attempt, while Transcode Session is an optional FFmpeg artifact.
However, playback planning and transcode policy are still too shallow for the
next product stage.

Current friction:

- HTTP handlers still select direct/remux/HLS paths and know too much about URL
  shape, tickets, output containers, and legacy transcode compatibility.
- `nako-transcode` can build FFmpeg commands, but command construction is not a
  user-facing planning interface.
- hardware acceleration is represented near command planning, not as an
  explainable policy with selected decode/filter/encode stages and fallback.
- Admin diagnostics and Public Client playback decisions can drift because they
  do not yet share a single planner/policy result.
- future features such as adaptive HLS ladders, optimized versions, remote
  transcode workers, per-user transcode permission, subtitle burn-in, HDR
  tonemapping, and desktop-native player capability negotiation need a deeper
  seam than route-level branching.

The target is not to copy Jellyfin. The target is to use Jellyfin-class feature
pressure to make the Nako playback architecture deep enough before expanding
feature breadth.

## Jellyfin-Class Feature Pressure

Nako should be able to carry these mature product requirements:

| Capability pressure | Nako target seam |
| --- | --- |
| Device profiles with containers/codecs/conditions | Client Playback Capabilities plus Library Playback Policy |
| PlaybackInfo with selected play method and stream URLs | Playback Planner result plus browser ticket/native transport adapters |
| Transcode reasons such as unsupported codec/bitrate/subtitle | typed Playback Decision Reasons |
| Encoding options for hardware, tonemapping, throttling, cleanup | Admin Playback Runtime Settings |
| Transcode job pings/progress/cancel/cleanup | Playback Session plus Transcode Artifact lifecycle |
| Segment deletion and throttling | Streaming Artifact Lifecycle policy |
| User policy for remux/audio transcode/video transcode | Library Access plus Playback Permission policy |
| Subtitle extraction, burn-in, HLS subtitle delivery | Subtitle Strategy inside Playback Planner and Transcode Policy |
| Hardware decode/encode fallback | Transcode Acceleration Plan and Runtime Inventory |
| Desktop/native clients | Client Playback Capabilities and native transport, not browser-only tickets |

## Target Modules

### Playback Planner

`PlaybackPlanner` is a deep Module. Callers give it facts; it returns a stable
plan and reasons. Callers should not know how direct/remux/HLS compatibility is
derived.

Candidate Interface shape:

```rust
pub struct PlaybackPlanningCommand {
    pub principal: UserPrincipalId,
    pub source: MediaSourcePlaybackFacts,
    pub item: Option<MediaItemPlaybackFacts>,
    pub client: ClientPlaybackCapabilities,
    pub library_policy: LibraryPlaybackPolicy,
    pub runtime: PlaybackRuntimeCapabilitySnapshot,
    pub preference: PlaybackPreference,
}

pub struct PlaybackPlan {
    pub mode: PlaybackPlanMode,
    pub selected_source_id: MediaSourceId,
    pub selected_streams: SelectedMediaStreams,
    pub transport: PlaybackTransportPlan,
    pub transcode: Option<TranscodeRequirement>,
    pub reasons: Vec<PlaybackDecisionReason>,
}
```

The Interface is intentionally not FFmpeg-shaped. It is playback-shaped.

### Transcode Policy

`TranscodePolicy` turns `TranscodeRequirement` into an execution-ready
`TranscodeArtifactPlan`.

It must model separate media stages:

```rust
pub struct TranscodeAccelerationPlan {
    pub decode: AccelerationStageSelection,
    pub filter: AccelerationStageSelection,
    pub encode: AccelerationStageSelection,
    pub fallback: AccelerationFallbackPolicy,
}
```

This avoids the common media-server trap of treating hardware acceleration as a
boolean. A future HDR tone-map or subtitle burn-in can force a different filter
stage even when encode remains hardware-backed.

### Playback Runtime Inventory

`PlaybackRuntimeInventory` probes and records redaction-safe runtime evidence:

- FFmpeg path/version presence;
- available hwaccels, encoders, decoders, and filters;
- configured devices and safe readiness status;
- selected hardware backend support by codec/pixel format where available;
- capability gaps and fallback reasons.

Admin diagnostics can expose the snapshot. Public Client responses should only
receive plan reasons and safe selected mode, not raw host paths or commands.

### Transcode Engine Adapter

`TranscodeEngineAdapter` executes plans, not policy:

```rust
pub trait TranscodeEngineAdapter {
    async fn start(&self, plan: TranscodeArtifactPlan) -> Result<TranscodeArtifactStart>;
    async fn cancel(&self, artifact_id: TranscodeSessionId) -> Result<TranscodeArtifactStop>;
}
```

The first implementation stays FFmpeg CLI. This keeps deployment simple and
preserves the existing `nako-transcode` investment while allowing future remote
workers or optimized-version adapters.

### Streaming Artifact Lifecycle

HLS and remux artifacts need explicit lifecycle semantics:

- active playback lease;
- segment retention/deletion;
- output cleanup after cancel/failure/end;
- throttling/backpressure;
- disk budget;
- redacted progress and failure evidence.

This is separate from Playback Session state. A Playback Session may be direct
play and have no transcode artifact at all.

## Crate Direction

- `nako-core`: pure playback policy records, IDs, enum vocabulary, repository
  traits where needed by more than one crate.
- `nako-playback`: playback planning records, client capability matching,
  decision reasons, selected source records, and playback profile identity. It
  must stay playback-shaped and must not serve bytes or build FFmpeg command
  lines.
- `nako-transcode`: FFmpeg command planning, capability probing, failure
  taxonomy, and FFmpeg engine Adapter.
- `nako-streaming`: byte-range, direct body, HLS/remux serving mechanics.
- `nako-server`: planner orchestration, settings, access checks, persistence,
  and HTTP route adaptation.
- `nako-api`: Admin runtime settings and diagnostics DTOs.
- `nako-client-protocol`: Public Client playback decisions, capabilities,
  sessions, and browser-safe transport DTOs.

PTP-040 established `ClientPlaybackDecisionReason` and
`ClientPlaybackCapabilitiesDto` as Public Client protocol types. Internal
planner reasons may grow faster than the public contract; `nako-api` is the
adapter that maps internal reasons to stable, safe wire values.

PTP-050 established `TranscodeExecutionPolicy` as the policy record consumed by
HLS profile identity and FFmpeg command planning. The policy carries
decode/filter/encode acceleration stages, selected/requested fallback evidence,
output constraints, and subtitle strategy. Public Client transcode plans no
longer expose server hardware selection; that remains Admin/runtime evidence
and execution policy.

PTP-060 established `TranscodeRuntimeInventory` and `TranscodeEngineAdapter`.
The inventory is a redaction-safe runtime summary for Admin diagnostics and
future planner/runtime decisions. FFmpeg CLI remains the first implementation,
but remux/HLS runners now satisfy typed engine start/progress outcomes instead
of being route-shaped process helpers.

PTP-030 extracted `nako-playback` because the deletion test became real:
removing selection from `nako-streaming` leaves a smaller transport crate, while
server app code and public DTO adapters both consume playback planning records.
Future extractions should use the same rule: split only when deletion makes an
existing crate more coherent and at least two callers need the resulting
domain-shaped API.

## Scope

In scope:

- workstream and ADR;
- current Nako playback/transcode audit against Jellyfin-class features;
- characterization tests for existing direct/remux/HLS/session behavior;
- Playback Planner Module;
- typed Client Playback Capabilities and Playback Decision Reasons;
- typed Transcode Policy and Acceleration Plan;
- Runtime Inventory and FFmpeg engine Adapter seams;
- Admin diagnostics/settings contracts needed to prove the policy;
- route cleanup where planner/policy replaces duplicated branching.

Out of scope:

- frontend player implementation;
- desktop Tauri/native playback implementation;
- recommendation systems;
- DLNA/UPnP compatibility;
- SyncPlay/watch-party behavior;
- live TV;
- remote/distributed transcode workers beyond interface readiness;
- copying reference project code.

## Risks

- Over-copying Jellyfin's DLNA profile model would make Nako's public client
  contract too broad and too legacy-oriented.
- Under-modeling hardware acceleration as a boolean would block tonemapping,
  subtitle burn-in, per-codec support, and explainable fallback.
- Moving FFmpeg command building too high would leak host paths/commands into
  policy and Admin/Public contracts.
- New crates can become shallow pass-through modules. Keep `nako-playback`
  responsible for actual planning decisions and profile identity, not merely
  re-exporting server or transcode types.
- Route compatibility must be handled carefully so browser playback tickets and
  legacy HLS segment URLs keep working while the planner lands.
