# Control-plane trace context first slice

## Goal

Add the first bounded HTTP trace-context seam so every server request has a
typed, redaction-safe request ID that can be returned to clients, attached to
request extensions, and used by future playback, job, VFS, FFmpeg, and addon
diagnostics.

## What I already know

* Parent queue `06-03-long-horizon-architecture-queue` lists Unified Trace
  Context as the remaining high-leverage control-plane follow-on after the
  completed playback, storage, HLS, and watch-folder slices.
* ADR 0053 says request identity and trace context should propagate across HTTP
  handlers, jobs, VFS calls, FFmpeg processes, addon calls, webhook delivery,
  and public or admin responses where useful.
* `docs/architecture/CONTROL_PLANE.md` scopes the full lane broadly, so this
  child task must stay deliberately small.
* The root router is assembled in `crates/nako-server/src/http.rs`; all
  responses already pass through the top-level API version header middleware.
* Existing HTTP tests use Axum routers plus `tower::ServiceExt`, so request-ID
  behavior can be covered without a live server.
* `nako-server` already depends on `uuid`, and existing code uses
  `uuid::Uuid::new_v4().simple()` for safe opaque IDs.

## Assumptions

* `x-request-id` is the right initial wire header because it is common and can
  be added without changing `nako-api` DTOs or generated contracts.
* The first slice should accept only a strict safe identifier alphabet from
  inbound headers and generate a new ID when the header is absent or invalid.
* Response bodies should not include the request ID yet; the response header is
  sufficient for this HTTP-boundary slice.

## Requirements

* Add a focused `nako-server` HTTP trace context module.
* Define a typed request context value that can be inserted into request
  extensions and extracted by future handlers.
* For every request handled by the root router, derive a safe request ID from
  `x-request-id` when valid, otherwise generate a new opaque ID.
* Add `x-request-id` to every response after the trace context middleware runs,
  including auth rejection and network/preflight responses.
* Include `x-request-id` in CORS preflight allow headers so browser clients can
  provide a safe request ID.
* Keep the ID redaction-safe:
  * no raw URLs, local paths, query strings, whitespace, slashes, commas,
    semicolons, control characters, or token-like oversized values;
  * bounded length;
  * stable lowercase normalization when accepting an inbound ID.
* Do not change auth behavior, route shapes, API DTOs, generated contracts,
  database schema, durable job records, playback behavior, VFS behavior, or
  FFmpeg command planning.
* Add focused server tests for generated IDs, accepted inbound IDs, invalid
  inbound header replacement, and propagation to protected-route rejections.

## Acceptance Criteria

* [x] Root server router returns `x-request-id` on `/health`.
* [x] A valid inbound `x-request-id` is normalized, stored in the typed context,
  and echoed in the response header.
* [x] An unsafe inbound request ID is replaced with a generated safe ID and is
  not echoed.
* [x] Protected-route `401` responses still include both `x-nako-api-version`
  and `x-request-id`.
* [x] Focused `nako-server` tests pass.
* [x] `cargo fmt --all -- --check`, `cargo check -p nako-server --tests`,
  focused `cargo nextest`, and `git diff --check` pass.

## Definition of Done

* Code and tests are committed with a Conventional Commit message.
* Verification evidence is persisted in this task directory.
* If the middleware establishes a reusable convention, update the relevant
  server spec and control-plane architecture map.
* Task is archived and the developer journal is recorded.

## Out of Scope

* No OpenTelemetry/exporter integration.
* No Admin incident bundle or recent-failure API.
* No trace persistence in job rows or database schema.
* No VFS, FFmpeg, addon, webhook, or playback session propagation in this
  slice.
* No public client protocol or Admin contract changes.
* No user-visible response body changes.

## Technical Approach

* Add `crates/nako-server/src/http/trace_context.rs`.
* Add a small typed context such as `HttpTraceContext` with a safe
  `request_id` accessor and an `X_REQUEST_ID_HEADER` constant.
* Implement an Axum `from_fn` middleware that:
  * reads `x-request-id`;
  * validates and normalizes the value;
  * generates `req_<uuid-simple>` or similar when absent/invalid;
  * inserts the context into request extensions;
  * runs the next service; and
  * writes the safe request ID to response headers.
* Mount the middleware in the top-level router assembly so it covers public,
  unauthenticated sensitive, protected, and addon runtime routes without
  changing route groups.
* Add tests under `crates/nako-server/src/http/tests/system.rs` or the HTTP
  router unit test module, following the existing `tower::ServiceExt` pattern.

## Research References

* [`research/current-http-trace-context.md`](research/current-http-trace-context.md)
  - existing HTTP router shape, ADR constraints, and recommended bounded seam.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-server/backend/index.md`
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
  * `.trellis/spec/nako-server/backend/logging-guidelines.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
  * `.trellis/spec/nako-server/backend/directory-structure.md`
  * `.trellis/spec/guides/cross-layer-thinking-guide.md`
* Relevant docs:
  * `docs/architecture/CONTROL_PLANE.md`
  * `docs/adr/0053-application-control-plane-boundary.md`
* Likely write scope:
  * `crates/nako-server/src/http.rs`
  * `crates/nako-server/src/http/network.rs`
  * `crates/nako-server/src/http/trace_context.rs`
  * `crates/nako-server/src/http/tests/system.rs` or
    `crates/nako-server/src/http/tests/mod.rs`
