# Transcode Runtime Workstream

## Purpose

This workstream turns Nako playback and transcode from the current remux/HLS
MVP into a product-grade runtime boundary. It owns playback service
decomposition, FFmpeg-backed hardware capability probing, selected
acceleration policy, resource budgets, persisted session lifecycle, and stable
client-facing playback contracts.

## Status

Completed through M26.

M25 built on the completed M24 server architecture cleanup. M26 then stabilized
the playback/session HTTP contract for future clients. The goal is not to add a
flashy streaming feature first; the goal is to make the runtime shape clean
enough that adaptive HLS, future subtitles, client playback controls, and
hardware-specific behavior do not re-entangle `nako-server`, `nako-transcode`,
`nako-streaming`, and storage code.

Top-level tracking:

- [Goal map](../../GOALS.md)
- [Roadmap](../../ROADMAP.md)
- [Milestones](MILESTONES.md)
- [TODO](TODO.md)
- [Phase 25.1 runtime productization slice](PHASE25_1_RUNTIME_PRODUCTIZATION.md)
- [Phase 26.0 playback client contract](PHASE26_0_PLAYBACK_CLIENT_CONTRACT.md)

## Goals

- Split the large playback application service into focused direct-play,
  remux, HLS, staging, and runtime orchestration modules.
- Move hardware capability probing from policy-only CPU defaults to an
  FFmpeg-backed detector when hardware acceleration is configured.
- Make VAAPI, NVENC, and QuickSync selection, fallback, and resource budget
  behavior explicit service contracts.
- Keep `nako-transcode` responsible for FFmpeg command planning and process
  execution primitives, not HTTP or storage concerns.
- Keep `nako-server::app` responsible for application orchestration, persisted
  sessions, storage staging, and API-safe errors.
- Define the playback session lifecycle and error model that future Flutter or
  web clients can depend on.

## Non-Goals

- No adaptive bitrate HLS ladder in the first slice.
- No client UI implementation.
- No direct FFmpeg remote credential input until a separate storage security
  design is accepted.
- No in-process plugin ABI or external worker process model.
- No distributed transcode queue; Nako remains a single-process modular
  monolith for this phase.

## Boundary Rules

- `nako-transcode` owns FFmpeg plans, hardware capability reports, runtime
  guards, process runners, temporary output promotion, and command-level
  errors.
- `nako-streaming` owns playback decisions and client capability matching.
- `nako-server::app` owns source lookup, storage staging, persisted session
  coordination, runtime service composition, and API-safe error categories.
- `nako-server::http` owns request extraction, headers, body streaming, and
  response translation only.
- Storage credentials and source locators must not be embedded into FFmpeg
  command lines or logs unless a dedicated remote-input security design
  explicitly allows it.

## Related Workstreams

- [server-architecture-hardening](../server-architecture-hardening/README.md):
  M24 cleaned the app service and runtime supervisor boundaries that M25 should
  build on.
- [playback-streaming](../playback-streaming/README.md): M7 remote playback,
  staging, and resource budget foundation.
- [runtime-foundation](../runtime-foundation/README.md): M15 hardware selection
  policy and cross-cutting runtime constraints.
- [server-foundation](../server-foundation/README.md): historical playback,
  remux, HLS, and hardware acceleration MVP notes.
