# Playback Policy And Renderer Targets

Status: Completed
Last updated: 2026-05-27

## Why This Lane Exists

Nako now has Playback Sessions, browser playback tickets, a Playback Planner,
runtime/transcode policy seams, and real User/Role/Library Access. The missing
piece is an explicit answer to this question:

> Given this user, source, network context, and renderer target, which playback
> behaviors are allowed?

Today playback routes mostly require `RequiredLibraryAccess::Play`. That is too
coarse for Jellyfin-class user needs such as disabling video transcode for a
viewer, allowing local direct play but denying remote playback, limiting remote
bitrate, or deciding whether one user may control another user's target
session.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- `docs/adr/0036-short-lived-browser-playback-tickets.md`
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
- `docs/adr/0039-playback-policy-and-renderer-target-boundary.md`
- `docs/workstreams/identity-and-library-access-contract/`
- `docs/workstreams/browser-playback-auth-transport/`
- `docs/workstreams/playback-transcode-policy-deepening/`
- `crates/nako-core/src/identity.rs`
- `crates/nako-core/src/session.rs`
- `crates/nako-playback/src/lib.rs`
- `crates/nako-server/src/http/access.rs`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-server/src/app/playback/`

## Jellyfin-Class Feature Pressure

The reference point is behavior and architecture pressure, not source-code
reuse.

Observed pressure from `repo-ref/jellyfin`:

| Jellyfin pressure | Nako boundary |
| --- | --- |
| `UserPolicy` distinguishes media playback, remux, audio transcode, video transcode, remote access, remote control, shared-device control, device allow-lists, active sessions, and bitrate limits. | `PlaybackPermissionPolicy` layered after Library Access. |
| `DeviceProfile` carries direct-play profiles, transcoding profiles, container/codec conditions, subtitle profiles, and bitrate limits. | `PlaybackTarget` plus `ClientPlaybackCapabilities`, not a copied DLNA model. |
| `PlaybackInfo` returns candidate media sources, a play session id, device-specific stream data, selected play method, and error codes. | Playback planner/app service returns safe decision, denial reasons, session identity, and transport adapter output. |
| `SessionInfoDto` includes capabilities, now-playing state, supported commands, and remote-control flags. | Renderer Session read models and command capability records. |
| `SessionController` sends browse/play/playstate/system commands to target sessions after session lookup and permission checks. | Future renderer-control app service with typed commands and target authorization. |
| Cast receiver applications are configured separately from normal playback planning. | Casting is a renderer adapter lane, not a planner mode. |

## Problem

Current gaps:

- playback authorization is too coarse: `Browse`, `Play`, and `Manage` do not
  express transcode/remux/direct/remote/cast permissions;
- `ClientPlaybackCapabilities` only describes container/codecs/direct-play
  support, not where playback will happen or which transport auth is required;
- planner decisions do not receive an effective playback policy, so denial
  reasons cannot distinguish "client cannot play this" from "user is not
  allowed to use this mode";
- Playback Session stores client capabilities JSON, but not a target kind,
  network scope, or renderer session relationship;
- Admin diagnostics can show runtime/transcode readiness, but not effective
  user playback policy;
- casting would currently have to tunnel through ad hoc browser ticket or route
  logic because no renderer target boundary exists.

## Target State

When this lane closes:

- Nako has explicit playback policy records for direct play, remux, audio
  transcode, video transcode, remote playback, remote control, cast, and
  bitrate constraints.
- Effective playback policy is resolved server-side from authenticated
  principal, Role, Library Access, optional user/role policy rows, source
  facts, and network scope.
- `nako-playback` planner receives policy and target capabilities as pure
  inputs and returns allow/deny decisions with typed reasons.
- Playback routes and `PlaybackAppService` stop hard-coding mode permission
  assumptions beyond HTTP authentication and ticket validation.
- Public Client API can request a target kind/capability shape without exposing
  Admin policy internals.
- Admin API can expose redaction-safe effective-policy diagnostics.
- The target model is ready for Tauri/native desktop and mobile clients, and
  ready for the follow-on casting lane.

## In Scope

- Workstream docs and ADR 0039.
- Characterization tests for current route/app planner gaps.
- Pure playback permission policy records.
- Renderer target and control capability records.
- Planner changes for policy-aware direct/remux/transcode decisions.
- Public Client DTO additions for target/capability request and safe denial
  reasons when needed.
- Admin diagnostics/readiness for effective playback policy.
- Repository/storage for user/role playback policy rows needed by effective
  policy resolution and tests.
- Focused route/app tests proving denied modes do not start Playback Sessions
  or Transcode Sessions.

## Out Of Scope

- Chromecast, DLNA, AirPlay, or Nako-to-Nako casting implementation.
- SyncPlay/watch-party behavior.
- Frontend UI.
- Recommendation systems.
- Live TV.
- Optimized Versions.
- Adaptive bitrate HLS ladders.
- Remote/distributed transcode workers.
- Copying Jellyfin code, schemas, comments, migrations, tests, or assets.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Library Access is necessary but not sufficient for mature playback control. | High | Jellyfin separates play access from remux/transcode/remote-control permissions; Nako currently only checks `RequiredLibraryAccess::Play`. | If Library Access remains enough, this lane can close after documenting no-op policy; casting would still need target/session records. |
| The first policy can default to permissive for administrators and existing local setups. | High | Nako is pre-production and has bootstrap admin semantics. | If secure-by-default needs stricter defaults, add an Admin settings migration and readiness task before public API changes. |
| `nako-playback` is the right home for pure policy-aware planning. | High | ADR 0038 extracted playback planning from streaming/HTTP and closed the route cleanup. | If dependency cycles appear, keep records in `nako-core` and server app orchestration in `nako-server`. |
| Renderer targets should be transport/capability contexts, not playback modes. | High | Casting, browser, native desktop, and native mobile differ in auth/control but can all request direct/remux/HLS playback. | If a protocol requires special playback output, model it as target constraints feeding the planner, not a new top-level mode. |
| Casting should follow this lane, not precede it. | High | Cast devices often cannot attach bearer headers and need policy/ticket/session primitives. | If a short Nako-to-Nako proof becomes urgent, it can use the same target model behind a feature-limited adapter. |

## Architecture Direction

### Playback Permission Policy

The policy should be explicit and explainable:

```text
EffectivePlaybackPolicy
  library_access
  allow_media_playback
  allow_direct_play
  allow_remux
  allow_audio_transcode
  allow_video_transcode
  allow_remote_playback
  allow_remote_control
  allow_cast
  max_streaming_bitrate
  max_remote_bitrate
  reason
```

`LibraryAccessLevel::Play` remains required before media playback. The
playback policy narrows what kind of playback is allowed.

Policy evaluation belongs in server app composition because it needs identity,
role/access repositories, source facts, Admin settings, and network context.
The planner should receive the already-resolved policy.

### Renderer Target

The first target record should describe stable capabilities and transport
requirements:

```text
PlaybackTarget
  id
  kind
  display_name
  network_scope
  transport_auth
  client_capabilities
  control_capabilities
```

Target kinds:

- `browser`
- `native_desktop`
- `native_mobile`
- `nako_remote_client`
- `chromecast`
- `dlna_renderer`
- `airplay`

Only browser/native targets need to be executable in this lane. Cast target
kinds are allowed as vocabulary so API and policy do not need to churn later.

### Planner Boundary

`PlaybackPlanningRequest` should grow from:

```text
source + probe + client + context
```

to:

```text
source + probe + client/target + effective_policy + context
```

The planner should return:

- selected source;
- allowed playback mode;
- execution plan;
- denial reason when blocked;
- safe decision reasons suitable for Public Client mapping.

When a policy denies a required mode, the route/app service must stop before
creating Playback Session or Transcode Session records.

### API Boundaries

Public Client API owns:

- target kind/capability request DTOs;
- safe decision/denial reasons;
- current user's target/session-safe state.

Admin API owns:

- effective playback policy diagnostics;
- policy storage/readiness if this lane persists user/role playback policies;
- runtime diagnostics that combine policy state with existing hardware and
  artifact lifecycle evidence.

Public DTOs must not expose:

- raw policy rows;
- role assignment internals;
- local paths or Source Locators;
- FFmpeg command strings;
- ticket secrets;
- renderer device secrets.

## Closeout Condition

This lane closed on 2026-05-27 after:

- ADR 0039 and workstream docs match shipped behavior;
- policy and target records exist at the right crate boundary;
- planner/app service enforce policy before direct/remux/HLS starts;
- public and admin DTOs expose only safe policy/target facts;
- focused tests prove mode denial, no artifact/session creation on denial, and
  unchanged compatible playback behavior;
- follow-on casting work is activated through `casting-renderer-runtime`.

Follow-on policy editing, bitrate-limit enforcement, and non-Nako casting
protocol adapters remain outside this lane.
