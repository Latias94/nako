# Playback Transcode Ops Hardening Design

Status: Complete
Last updated: 2026-05-22

## Why This Lane Exists

Nako already has the core **Playback Runtime** building blocks:

- direct play, remux, and HLS orchestration;
- FFmpeg-backed hardware capability probing for VAAPI, NVENC, and Quick Sync;
- hardware acceleration policy and CPU fallback behavior;
- transcode and remote playback resource budgets;
- persisted playback transcode sessions;
- cancellation and session inspection;
- read-only Admin playback runtime diagnostics.

Those pieces are enough for a functional server, but they are not yet enough
for a self-hosted operator to confidently answer "why is playback degraded or
failing?" without inspecting local config, paths, logs, process commands, or
secrets.

This lane hardens supportability after the high-risk metadata, NFO, and
Library File Write boundaries are already proven.

## Relevant Authority

- ADRs:
  - `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
  - `docs/adr/0021-video-first-media-server-domain-model.md`
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/workstreams/transcode-runtime/README.md`
  - `docs/workstreams/playback-streaming/README.md`
  - `docs/workstreams/admin-playback-runtime-diagnostics/DESIGN.md`
  - `docs/workstreams/admin-playback-session-read-model/README.md`
- Related code:
  - `crates/nako-transcode`
  - `crates/nako-streaming`
  - `crates/nako-server/src/app/playback`
  - `crates/nako-server/src/http/admin.rs`
  - `crates/nako-api/src/admin.rs`

## Problem

The current implementation has useful runtime facts, but the facts are not yet
shaped as a complete operator contract:

- readiness is spread across FFmpeg probe status, selected hardware
  acceleration, budgets, staging config, and storage diagnostics;
- fallback reasons are mostly human strings instead of stable categories that
  can be tested, documented, and consumed by Admin UI/support tooling;
- playback transcode request/profile facts can be constructed before every
  impossible or unsafe combination is rejected explicitly;
- session failures are not yet grouped into a support-first taxonomy across
  probe, plan, staging, budget, runner, timeout, cancellation, and hardware
  fallback boundaries;
- support evidence is available in several places, but there is no bounded,
  redacted read model for "what should I send when this playback fails?"

## Target State

When this lane closes:

- Admin runtime diagnostics expose a stable readiness state and safe reason
  categories for FFmpeg, hardware acceleration, budget, staging, and remote
  playback prerequisites.
- Hardware fallback behavior is explainable through typed, redaction-safe
  reason codes while preserving operator-readable messages.
- Playback transcode request/profile validation rejects impossible or unsafe
  combinations before Nako creates or starts a session.
- Session failures use a stable support taxonomy that separates probe,
  planning, staging, budget, runner, timeout, cancellation, and hardware
  fallback classes.
- An Admin-only support evidence read model can collect bounded playback
  runtime/session evidence without exposing raw Source Locators, local paths,
  FFmpeg command lines, output paths, stderr payloads, secrets, or credentials.
- Public Client API behavior remains stable unless a separate workstream
  explicitly accepts a client-contract change.

## In Scope

- Playback runtime readiness contract.
- FFmpeg/hardware capability evidence classification.
- Hardware fallback reason codes and redacted diagnostics.
- Transcode request/profile validation before session creation or execution.
- Playback session failure taxonomy and redacted admin read models.
- Focused tests in `nako-transcode`, `nako-streaming`, `nako-api`, and
  `nako-server`.
- Workstream and umbrella roadmap closeout updates.

## Out Of Scope

- Adaptive bitrate HLS ladders.
- Durable **Optimized Version** assets.
- Distributed transcode workers or queues.
- Direct FFmpeg remote URL/credential input.
- New downloader, watch-folder, torrent, Usenet, or acquisition protocols.
- Metadata, NFO, sidecar, or Library File Write mutation.
- Addon playback replacement or in-process plugin behavior.
- Public Client API changes unless split into a dedicated client-contract lane.
- Admin web console UI implementation beyond documenting the read model.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The existing Admin playback runtime route is the right home for server-wide readiness. | High | `admin-playback-runtime-diagnostics` closeout and `GET /admin/v1/playback/runtime` tests | Split a narrower Admin diagnostics follow-on instead of expanding Public Client API. |
| `nako-transcode` should own typed hardware/fallback and request validation concepts. | High | `transcode-runtime` boundary rules and existing `HardwareAccelerationReport` / `TranscodeProfile` code | If validation needs app context, keep pure validation in `nako-transcode` and compose context in `nako-server::app::playback`. |
| A first support evidence bundle can be a bounded read model, not a persisted artifact. | Medium | Existing session rows, admin session list, runtime diagnostics, and staging diagnostics | If operators need retention/export, split a persistence/export follow-on. |
| Public Client API can remain unchanged for this lane. | Medium | This work is operational and Admin-only by default | If client-visible errors must change, split a Public Client API contract task before implementation. |

## Architecture Direction

Keep ownership narrow:

- `nako-transcode` owns FFmpeg command plans, hardware capability reports,
  selected acceleration, fallback reason codes, and transcode request/profile
  validation that does not require repository or storage context.
- `nako-streaming` owns playback decisions and client capability matching. It
  may expose selection prerequisites, but it must not own FFmpeg execution or
  Admin support surfaces.
- `nako-server::app::playback` owns source lookup, staging, session lifecycle,
  runtime composition, cancellation, and translating runtime events into
  redacted support evidence.
- `nako-api::admin` owns Admin DTO shapes. It must not leak into
  `nako-client-protocol` or Public Client API unless a separate contract lane
  accepts that change.
- `nako-server::http::admin` translates app snapshots into Admin responses and
  enforces route-level redaction/auth behavior.

Redaction is not optional. The support surface may expose categories, counts,
booleans, enum values, safe encoder names, API versions, timeout/budget values,
and stable IDs. It must not expose raw local paths, raw Source Locators,
FFmpeg paths, command argv, output paths, raw stderr, storage credentials,
provider secrets, or private environment variable values.

## Task Shape

This lane is split into vertical slices:

1. readiness contract;
2. validation and fallback reason hardening;
3. session failure taxonomy;
4. support evidence read model;
5. closeout and parent re-score.

Each task must leave a fresh evidence entry before the next task is accepted.

## Closeout Summary — 2026-05-22

The lane closed with the intended operator-supportability contract:

- `GET /admin/v1/playback/runtime` exposes stable readiness checks for FFmpeg
  probe state, hardware capability, fallback selection, transcode budget,
  remote playback budget, and staging prerequisites.
- Playback transcode profile and plan construction reject unsafe or impossible
  combinations before session identity, staging, or FFmpeg execution.
- Transcode session failures are categorized by support boundary while public
  client responses keep their coarse compatibility contract.
- `GET /admin/v1/playback/support` provides a bounded Admin-only evidence read
  model for runtime/session/source context without raw Source Locators, local
  paths, FFmpeg paths, command argv, output paths, raw stderr, fingerprints,
  secrets, or credentials.
- Admin TypeScript contract, typed client, mock data, and tests were synced for
  the Admin API surface only.
- Public Client API and `nako-client-protocol` remained unchanged.

Residual work is intentionally split:

- downloadable support bundles, retention, and Admin UI workflows;
- adaptive bitrate ladders and durable Optimized Versions;
- direct remote FFmpeg credentials or distributed transcode workers;
- downloader/watch-folder, network access, AI, and Addon runtime lanes.

## Closeout Condition

This lane can close when:

- PTOH tasks are complete with fresh command evidence;
- Admin playback runtime/support diagnostics are redaction-safe;
- public client and generated SDK boundaries are explicitly unchanged or split;
- docs reflect shipped behavior;
- downloads/watch-folder, network, AI, and addon runtime are re-scored in the
  parent umbrella.
