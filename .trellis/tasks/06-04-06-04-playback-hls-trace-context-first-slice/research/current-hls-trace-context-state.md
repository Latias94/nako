# Current HLS Trace Context State

## Existing HTTP Trace Context

`crates/nako-server/src/http/trace_context.rs` defines:

* `X_REQUEST_ID_HEADER`
* `HttpTraceContext`
* `attach_http_trace_context`

The middleware normalizes a safe inbound `x-request-id`, generates a safe
`req_<uuid>` when missing or unsafe, inserts `HttpTraceContext` into request
extensions, and writes `x-request-id` onto the response.

Current visibility is `pub(super)`, which makes it usable from sibling modules
inside `http`, including `http/playback.rs`, but not from `app`. That is the
right boundary for this task: convert HTTP trace context into an app-layer
playback trace context at the route handler edge.

## HLS HTTP Handler Entry Points

`crates/nako-server/src/http/playback.rs` owns HLS source playlist handling:

* `hls_playlist_source`
* `HlsPlaybackQuery`
* conversion to `HlsPlaylistPlaybackRequest`
* conversion to `HlsPlaylistSessionRequest`

The handler currently extracts app state, optional authenticated principal,
source id, and query. It can also extract
`Extension<crate::http::trace_context::HttpTraceContext>` because the root
router already installs the trace middleware.

Recommended conversion:

* Add `Extension(http_trace): Extension<HttpTraceContext>` to
  `hls_playlist_source`.
* Convert to a playback/app trace context with only `request_id`.
* Pass cloned trace context into each HLS playlist request struct.

## HLS App Flow

`crates/nako-server/src/app/playback/mod.rs` defines:

* `HlsPlaylistPlaybackRequest`
* `HlsPlaylistSessionRequest`
* `HlsSourceRequest`
* `PlaybackAppService::hls_playlist_playback`
* `PlaybackAppService::hls_playlist_for_playback_session`
* `PlaybackAppService::hls_source`

`crates/nako-server/src/app/playback/hls_flow.rs` takes `HlsSourceRequest` and
threads the request into the HLS planner/staging/runtime path. This is the
right app-layer seam to carry a small optional trace context without touching
public DTOs, schema, or transcode command planning.

Recommended bounded app type:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackTraceContext {
    request_id: String,
}
```

Keep construction strict at the HTTP edge. The app type should not parse raw
headers or know about `x-request-id`.

## Completion Event Surface

`crates/nako-server/src/app/playback/events.rs` currently builds
`PlaybackSessionFinished` payloads with:

* `session_id`
* `source_id`
* `kind`
* `request_key`
* `state`

This helper is called from both HLS and remux:

* `crates/nako-server/src/app/playback/hls.rs`
* `crates/nako-server/src/app/playback/remux.rs`

Recommended change:

* Introduce a small `PlaybackSessionEventContext` or pass
  `Option<&PlaybackTraceContext>` to the event helper.
* Add `"request_id": <safe id>` only when trace context is present.
* Pass the HLS trace context from `HlsAppService::run_reserved`.
* Pass `None` from remux for this first slice unless remux propagation is
  explicitly widened later.

This avoids schema changes and keeps the first diagnostic correlation surface
internal.

## Test Targets

Preferred focused test:

* Add an app-level or HTTP-level server playback test that starts an HLS
  playlist request with a safe inbound `x-request-id`, waits for HLS completion
  under the existing test runner setup, and reads the persisted
  `PlaybackSessionFinished` outbox event payload to assert `request_id`.

Fallback focused test if HTTP HLS setup is too heavy:

* Unit/integration test the event helper with `Some(PlaybackTraceContext)` and
  `None`, plus a narrow handler extraction test showing the safe request ID is
  converted into `HlsPlaylistPlaybackRequest`.

Keep tests in `nako-server`; do not add generated API contract tests because no
public DTO changes are intended.

## Risks And Boundaries

* Do not add a transcode session column; that would be a separate schema task.
* Do not expose request ID in response bodies.
* Do not put raw ticket query strings, renderer ticket values, paths, source
  locators, FFmpeg argv, or provider payloads into event metadata.
* Do not make `app` depend on HTTP headers. HTTP owns extraction; app owns a
  sanitized trace context value.
* If signatures become noisy, keep propagation only through HLS playlist/source
  request structs and HLS runtime methods.

## Recommended Write Scope

* `crates/nako-server/src/http/trace_context.rs`
* `crates/nako-server/src/http/playback.rs`
* `crates/nako-server/src/app/playback/mod.rs`
* `crates/nako-server/src/app/playback/hls_flow.rs`
* `crates/nako-server/src/app/playback/hls.rs`
* `crates/nako-server/src/app/playback/events.rs`
* focused `crates/nako-server/src/app/tests` or `http/tests` coverage
