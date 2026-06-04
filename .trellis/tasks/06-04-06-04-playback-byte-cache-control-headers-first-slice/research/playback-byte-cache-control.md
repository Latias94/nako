# Research: playback byte cache-control headers

- Query: Add the narrowest cache-control baseline for Direct Play and Remux
  media byte responses.
- Scope: internal code/docs only.
- Date: 2026-06-04.

## Findings

* `crates/nako-server/src/http/playback.rs` has one shared helper,
  `apply_direct_play_headers`, for Direct Play and Remux byte response headers.
* The helper covers streaming responses, empty HEAD/preflight responses, and
  range-not-satisfiable responses.
* HLS uses a separate `apply_hls_artifact_cache_headers` helper and already has
  `Cache-Control: no-store`.
* Selected artwork lives in `http/catalog.rs` and now has private cache headers
  plus exact ETag revalidation. It should not share playback byte semantics.
* `crates/nako-server/src/http/tests/playback.rs` already has focused tests for
  Direct Play HEAD/range and Remux GET/HEAD behavior.

## Recommendation

Add `Cache-Control: no-store` to `apply_direct_play_headers` and assert it in
existing focused Direct Play and Remux route tests. This is a dedicated playback
byte cache-contract task, so it is intentionally separate from the prior HLS
HLS-only guardrail.

## Guardrails

* Keep HLS and selected artwork helpers separate.
* Do not add ETags or conditional GET for Direct Play/Remux in this slice.
* Preserve range, content length, content range, status, and playback session
  headers.
