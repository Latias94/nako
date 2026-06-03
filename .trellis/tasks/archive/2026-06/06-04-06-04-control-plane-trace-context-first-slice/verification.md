# Verification

Final gate evidence for control-plane trace context first slice.

## Commands

* `cargo fmt --all`
  - Pass.
* `cargo check -p nako-server --tests`
  - Pass.
* `cargo nextest run -p nako-server trace_context health_and_libraries_routes_work bearer_auth_protects_non_health_routes_and_keeps_health_public network_boundary_enforces_origin_policy_and_preserves_auth_order --no-fail-fast`
  - Pass: 8 tests run, 8 passed.
* `cargo fmt --all -- --check`
  - Pass.
* `git diff --check`
  - Pass; only Git CRLF conversion warnings were printed.

## Review Notes

* The HTTP trace context slice is server-only. It does not change API DTOs,
  generated contracts, schema, durable jobs, VFS, FFmpeg, addon, or playback
  runtime behavior.
* Root middleware order keeps request ID annotation outside auth and network
  short-circuit paths, so `/health`, protected `401`, and CORS preflight
  responses all carry `x-request-id`.
* Unsafe inbound request IDs are replaced rather than echoed. Accepted inbound
  IDs use a bounded safe alphabet and lowercase normalization.
* CORS preflight now allows `x-request-id` so browser clients can provide the
  same safe correlation header.
