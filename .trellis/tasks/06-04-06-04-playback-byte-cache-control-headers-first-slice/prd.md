# Playback Byte Cache-Control Headers First Slice

## Goal

Give authenticated or ticket-scoped Direct Play and Remux media byte responses
an explicit conservative cache contract without changing HLS, selected artwork,
DTOs, schemas, or byte-range behavior.

## What I Already Know

* HLS playlist and segment responses already use `Cache-Control: no-store`.
* Selected artwork image responses use `Cache-Control: private, max-age=86400`
  and exact `If-None-Match` / 304 behavior because they are selected artwork
  images with safe ETags.
* Direct Play and Remux byte responses share `apply_direct_play_headers` in
  `crates/nako-server/src/http/playback.rs`.
* `apply_direct_play_headers` already owns status, `Accept-Ranges`,
  `Content-Type`, `Content-Length`, and optional `Content-Range` for Direct
  Play, Remux, HEAD/preflight, and range-not-satisfiable responses.
* Direct Play and Remux routes are protected by API auth or short-lived browser
  playback tickets.

## Requirements

* Add `Cache-Control: no-store` to Direct Play and Remux media byte responses
  through the existing direct/remux byte response header helper.
* Preserve existing status codes, body/no-body behavior, range headers,
  playback session headers, auth, ticket validation, and content headers.
* Keep HLS and selected artwork cache helpers separate.
* Do not add ETags, conditional GET, DTO/generated contract changes, schema
  changes, or new public route behavior.
* Add focused tests for Direct Play HEAD, Direct Play range GET, Remux range
  GET, and Remux HEAD/preflight behavior.

## Acceptance Criteria

* [x] Direct Play GET/range responses include `Cache-Control: no-store`.
* [x] Direct Play HEAD/preflight responses include `Cache-Control: no-store`
  with no body.
* [x] Remux GET/range responses include `Cache-Control: no-store`.
* [x] Remux HEAD/preflight responses include `Cache-Control: no-store` with no
  body.
* [x] HLS `no-store` and selected artwork private cache behavior remain
  separate.
* [x] Focused server checks pass and evidence is recorded.

## Technical Approach

Insert `Cache-Control: no-store` in `apply_direct_play_headers`. That helper is
already shared by Direct Play and Remux streaming/preflight response assembly
and is separate from HLS and selected artwork helpers.

## Out Of Scope

* No Direct Play or Remux ETags.
* No conditional GET / 304 for media bytes.
* No immutable, shared-cache, or CDN semantics.
* No HLS or selected artwork behavior changes.
* No DTO, generated contract, OpenAPI, SDK, or schema changes.

## Verification Plan

* `cargo fmt --all -- --check`
* `cargo check -p nako-server --tests`
* `cargo nextest run -p nako-server direct_stream_head_returns_headers_without_body --no-fail-fast`
* `cargo nextest run -p nako-server direct_stream_route_records_playback_session_without_transcode_artifact --no-fail-fast`
* `cargo nextest run -p nako-server remux_stream_route_runs_and_reuses_completed_output --no-fail-fast`
* `cargo nextest run -p nako-server head_remux_stream_route_exposes_session_without_body --no-fail-fast`
* `git diff --check`

## Verification Evidence

* PASS: `cargo fmt --all -- --check`
* PASS: `cargo check -p nako-server --tests`
* PASS: `cargo nextest run -p nako-server direct_stream_head_returns_headers_without_body --no-fail-fast`
* PASS: `cargo nextest run -p nako-server direct_stream_route_records_playback_session_without_transcode_artifact --no-fail-fast`
* PASS: `cargo nextest run -p nako-server remux_stream_route_runs_and_reuses_completed_output --no-fail-fast`
* PASS: `cargo nextest run -p nako-server head_remux_stream_route_exposes_session_without_body --no-fail-fast`
* PASS: `git diff --check`
* PASS: `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-06-04-playback-byte-cache-control-headers-first-slice`

## Spec Update

Updated `.trellis/spec/nako-server/backend/http-api-patterns.md` with the
dedicated Direct Play/Remux playback byte cache-control contract and adjusted
the HLS cache-control guardrail to point at the dedicated byte-route helper.
Updated `docs/architecture/CONTROL_PLANE.md` and
`docs/architecture/PLAYBACK.md` to record the Direct Play/Remux `no-store`
baseline.
