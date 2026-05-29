# HLS Progressive Runtime Boundary — Evidence And Gates

Status: Completed
Last updated: 2026-05-29

## Smallest Current Repro

The current runtime proof now spans `nako-transcode` and `nako-server`: HLS
output publication is explicit, and playlist-facing server paths can return
after playlist readiness while the transcode session remains running.

```bash
cargo nextest run -p nako-transcode hls_runner --no-fail-fast
```

## Gate Set

### Planning Gate

```bash
python3 -m json.tool docs/workstreams/hls-progressive-runtime-boundary/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-progressive-runtime-boundary docs/workstreams/README.md docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md
```

Proves the workstream docs are syntactically valid and do not introduce
whitespace artifacts.

### HLS Runtime Iteration Gate

```bash
cargo nextest run -p nako-transcode hls_runner --no-fail-fast
cargo nextest run -p nako-transcode hls --no-fail-fast
```

Proves FFmpeg HLS runner behavior, command planning, artifact manifest rules,
and request-variant identity.

### Server HLS Gate

```bash
cargo nextest run -p nako-server hls_source --no-fail-fast
cargo nextest run -p nako-server hls_segment --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
```

Proves server playlist readiness, segment serving, artifact reconstruction,
ticket rewrite coverage, and HLS playback orchestration.

### Playback And Renderer Gate

```bash
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-server renderer --no-fail-fast
```

Proves existing playback sessions, browser tickets, and renderer transport
contracts still work.

### Final Closeout Gate

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Use a broader workspace gate only if the final implementation changes shared
protocol, database, or API contracts outside the focused playback/transcode
surface.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or link to a review
note.

## Evidence Log

| Date | Task | Evidence | Status | Notes |
| --- | --- | --- | --- | --- |
| 2026-05-29 | HPRB-010 | Workstream opened | Passed | `python3 -m json.tool docs/workstreams/hls-progressive-runtime-boundary/WORKSTREAM.json`; `git diff --check -- docs/workstreams/hls-progressive-runtime-boundary docs/workstreams/README.md docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md`. |
| 2026-05-29 | HPRB-020 | HLS output publication policy proof | Passed | `cargo nextest run -p nako-transcode hls_runner --no-fail-fast`; `cargo nextest run -p nako-transcode hls --no-fail-fast`. |
| 2026-05-29 | HPRB-020 | Closeout verification | Passed | `python3 -m json.tool docs/workstreams/hls-progressive-runtime-boundary/WORKSTREAM.json`; `cargo fmt --all -- --check`; `git diff --check`. Review found no blocking issues; HPRB-030 must preserve manifest-backed serving when using `ServeWhileRunning`. |
| 2026-05-29 | HPRB-030 | Progressive HLS server proof | Passed | `cargo nextest run -p nako-server hls_playlist --no-fail-fast`; `cargo nextest run -p nako-server hls_source --no-fail-fast`; `cargo nextest run -p nako-server hls_segment --no-fail-fast`; `cargo nextest run -p nako-server hls --no-fail-fast`. Proves playlist readiness returns while the session is `Running`, generated segments are served, missing running segments conflict as not ready, and cancellation reaches `Cancelled`. |
| 2026-05-29 | HPRB-030 | Closeout verification | Passed | `python3 -m json.tool docs/workstreams/hls-progressive-runtime-boundary/WORKSTREAM.json`; `cargo fmt --all -- --check`; `git diff --check`. Review found and fixed the pre-session input preparation risk by preparing FFmpeg input before background HLS spawn. |
| 2026-05-29 | HPRB-040 | Typed HLS artifact reconstruction | Passed | `cargo nextest run -p nako-transcode hls_request_variant --no-fail-fast`; `cargo nextest run -p nako-server hls_artifact --no-fail-fast`. Proves `nako-transcode::HlsArtifactSpec` reconstructs single-variant fMP4 and adaptive fMP4 manifests from persisted request identity, including request-variant ladder/media rendition data, and server artifact serving delegates to that typed boundary. |
| 2026-05-29 | HPRB-040 | Closeout verification | Passed | `python3 -m json.tool docs/workstreams/hls-progressive-runtime-boundary/WORKSTREAM.json`; `cargo fmt --all -- --check`; `git diff --check`. Review found no blocking issues after adding single-variant fMP4 reconstruction coverage. |
| 2026-05-29 | HPRB-050 | Manifest-aware playlist authoring and auth decoration | Passed | `cargo nextest run -p nako-server hls --no-fail-fast`; `cargo nextest run -p nako-server renderer --no-fail-fast`. Proves browser ticket and renderer ticket query decoration now flow through `author_hls_session_playlist` with manifest-backed route binding instead of HTTP-local playlist rewrite passes. |
| 2026-05-29 | HPRB-050 | Closeout verification | Passed | `python3 -m json.tool docs/workstreams/hls-progressive-runtime-boundary/WORKSTREAM.json`; `cargo fmt --all -- --check`; `git diff --check`. Review found no blocking issues; HPRB-060 should run the final broader closeout gate and split any LL-HLS/DASH/DRM/resource-scheduler follow-ons. |
| 2026-05-29 | HPRB-060 | Final HLS closeout gate | Passed | `cargo nextest run -p nako-transcode hls --no-fail-fast`; `cargo nextest run -p nako-server hls --no-fail-fast`; `cargo nextest run -p nako-server playback --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. Closeout initially exposed Windows/concurrent test races and a real readiness gap where a running playlist path could exist before it contained a media or variant URI. The final implementation requires a non-comment URI line before treating a running playlist as ready, keeps running missing segments as bounded not-ready conflicts, and aligns HTTP tests with progressive serving semantics. |
| 2026-05-29 | HPRB-060 | Follow-on split | Passed | LL-HLS, DASH/CMAF, DRM/key delivery, remote transcode workers, selected-main-audio cleanup, and the playback runtime resource scheduler remain out of this lane and are recorded as proposed follow-ons in the architecture/workstream closeout docs. |
| 2026-05-29 | HPRB-060 | Review and verification | Passed | Workstream compliance and code-quality review found no blocking issues. Fresh verification covered the final closeout gate and JSON syntax. `cargo nextest run -p nako-server renderer --no-fail-fast` was not rerun in HPRB-060 because this closeout changed HLS readiness/tests/docs only and the final closeout gate narrowed coverage to `playback`; HPRB-050 already carried renderer evidence for auth decoration. |

## Evidence Anchors

- `docs/workstreams/hls-progressive-runtime-boundary/DESIGN.md`
- `docs/workstreams/hls-progressive-runtime-boundary/TODO.md`
- `docs/workstreams/hls-progressive-runtime-boundary/MILESTONES.md`
- `docs/workstreams/hls-progressive-runtime-boundary/CLOSEOUT.md`
- `crates/nako-transcode/src/hls.rs`
- `crates/nako-server/src/app/playback/hls.rs`
- `crates/nako-server/src/app/playback/hls_artifact.rs`
- `crates/nako-server/src/app/playback/playlist.rs`

Fresh verification is required before marking a task, Codex goal, or lane
complete.
