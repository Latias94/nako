# Evidence

## Verification Commands

* PASS: `cargo fmt --all`
* PASS: `cargo check -p nako-server --tests`
* PASS:
  `cargo nextest run -p nako-server hls_playlist_and_segment_routes_work --no-fail-fast`
* PASS: `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
* PASS: `cargo fmt --all -- --check`
* PASS:
  `python .\\.trellis\\scripts\\task.py validate 06-04-06-04-hls-artifact-cache-control-headers-first-slice`
* PASS: `git diff --check`

`git diff --check` printed only Git line-ending warnings for tracked files on
Windows and no whitespace errors.

## Scope Verification

* HLS playlist responses now assert `Cache-Control: no-store`.
* HLS segment responses now assert `Cache-Control: no-store`.
* Direct Play and Remux cache behavior was not changed.
* No DTO, generated contract, schema, planner, or FFmpeg command planning
  changes were made.
