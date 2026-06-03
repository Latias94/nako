# Playback HLS trace context propagation first slice

## Goal

Propagate the existing redaction-safe HTTP request ID into the HLS playlist
startup path so playback diagnostics and completion events can be correlated
with the request that started the work, without changing public API shape,
database schema, or FFmpeg command planning.

## What I Already Know

* `06-04-06-04-control-plane-trace-context-first-slice` shipped
  `x-request-id` normalization, response echoing, and a typed
  `HttpTraceContext` request extension.
* `docs/architecture/CONTROL_PLANE.md` still lists playback and library scan
  trace propagation as follow-ons after the HTTP boundary slice.
* HLS playlist routes are in `crates/nako-server/src/http/playback.rs`.
* HLS app orchestration is in `crates/nako-server/src/app/playback/mod.rs`,
  `hls_flow.rs`, and `hls.rs`.
* HLS and remux completion currently emit `PlaybackSessionFinished` outbox
  events through `crates/nako-server/src/app/playback/events.rs`.

## Assumptions

* This slice should create an app-level trace context seam that can later be
  reused by remux, direct play, jobs, VFS, FFmpeg, and addon paths.
* The first executable propagation target should be HLS playlist startup only,
  because it is a high-value path with existing session/event evidence.
* Outbox event payload metadata is an acceptable first diagnostic surface
  because it avoids schema changes and stays internal/redaction-safe.

## Requirements

* Expose the HTTP trace context to playback HTTP handlers without weakening the
  existing request-ID validation rules.
* Add a small playback/app trace context type that carries only safe request
  identity.
* Thread that context through HLS playlist source/session request structs and
  HLS runtime orchestration.
* Include the request ID in HLS `PlaybackSessionFinished` outbox payloads when
  the HLS work was started from a traced HTTP request.
* Preserve existing remux completion event payloads unless a trace context is
  explicitly passed.
* Keep all payloads redaction-safe: no paths, URLs, tickets, bearer tokens,
  source locators, raw FFmpeg argv, or provider payloads.
* Add focused tests that prove a safe inbound `x-request-id` reaches the HLS
  completion event payload.

## Acceptance Criteria

* [x] `HttpTraceContext` can be extracted by playback HTTP handlers.
* [x] `HlsPlaylistPlaybackRequest` and `HlsPlaylistSessionRequest` can carry a
  redaction-safe trace context.
* [x] HLS completion outbox events include `request_id` when trace context is
  present.
* [x] Existing playback behavior and response contracts remain unchanged.
* [x] Focused `nako-server` HLS/trace tests pass.
* [x] `cargo fmt --all -- --check`, `cargo check -p nako-server --tests`, and
  `git diff --check` pass.

## Definition Of Done

* Code and tests are committed with a Conventional Commit message.
* Task evidence records verification commands.
* Relevant Trellis spec or architecture docs are updated if this establishes a
  reusable trace propagation convention.
* The task is archived and the developer journal is recorded.

## Out Of Scope

* No database schema migration or new transcode session column.
* No public API/Admin API DTO or generated contract changes.
* No OpenTelemetry exporter, tracing subscriber, incident bundle, or metrics
  backend.
* No VFS, FFmpeg argv, addon, durable job, remux, direct play, or library scan
  propagation except where required to keep shared helper signatures coherent.
* No exposure of request IDs in response bodies.

## Technical Approach

Introduce an app-layer `PlaybackTraceContext` with a request ID constructor and
accessor. Make the HTTP `HttpTraceContext` visible inside the `http` module and
convert it at HLS playlist handler boundaries. Carry the trace context through
`HlsPlaylistPlaybackRequest` / `HlsPlaylistSessionRequest`, then into
`HlsSourceRequest`, `HlsAppService::run`, and HLS completion event recording.
Update event payload construction to include `request_id` only when supplied.

## Research References

* Pending: `research/current-hls-trace-context-state.md`

## Technical Notes

Likely specs:

* `.trellis/spec/nako-server/backend/index.md`
* `.trellis/spec/nako-server/backend/http-api-patterns.md`
* `.trellis/spec/nako-server/backend/logging-guidelines.md`
* `.trellis/spec/nako-server/backend/quality-guidelines.md`
* `.trellis/spec/guides/cross-layer-thinking-guide.md`

Likely write scope:

* `crates/nako-server/src/http/trace_context.rs`
* `crates/nako-server/src/http/playback.rs`
* `crates/nako-server/src/app/playback/mod.rs`
* `crates/nako-server/src/app/playback/hls_flow.rs`
* `crates/nako-server/src/app/playback/hls.rs`
* `crates/nako-server/src/app/playback/events.rs`
* focused `nako-server` tests

## Verification

* PASS: `cargo fmt --all -- --check`
* PASS: `cargo check -p nako-server --tests`
* PASS: `git diff --check`
* PASS: `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
* PASS: `cargo nextest run -p nako-server http_trace_context --no-fail-fast`
* PASS:
  `cargo nextest run -p nako-server hls_playlist_completion_event_includes_trace_request_id --no-fail-fast`

## Spec Update

* Updated `.trellis/spec/nako-server/backend/http-api-patterns.md` with the
  app-layer trace context propagation convention and HLS outbox payload test
  requirement.
* Updated `docs/architecture/CONTROL_PLANE.md` to record HLS playlist-to-event
  request ID propagation and keep broader job/VFS/FFmpeg/addon/library scan
  propagation as follow-ons.
