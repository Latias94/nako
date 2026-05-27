# Source-Aware Transcode Runtime - TODO

Status: Completed
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

- [x] SATR-010 [owner=planner] [deps=none] [scope=docs/workstreams/source-aware-transcode-runtime,docs/adr]
  Goal: Open the source-aware transcode runtime lane, record the ADR, and freeze
  boundaries, non-goals, task order, and validation gates.
  Validation: `python -m json.tool docs/workstreams/source-aware-transcode-runtime/WORKSTREAM.json`
  Evidence: `docs/workstreams/source-aware-transcode-runtime/DESIGN.md`, `docs/adr/0049-source-aware-transcode-runtime.md`
  Handoff: Planner owns this before execution starts.

## M1 - Source Media Technical Facts

- [x] SATR-020 [owner=codex] [deps=SATR-010] [scope=crates/nako-core,crates/nako-media-probe]
  Goal: Extend `MediaProbeResult` and ffprobe mapping with source-aware facts
  needed for decoder, filter, HDR, subtitle, and audio decisions.
  Validation: `cargo nextest run -p nako-media-probe --no-fail-fast`
  Review: `review-workstream` before accepting completion.
  Evidence: parser fixture tests covering profile, level, pixel format, bit
  depth, frame rate, disposition, and color metadata.
  Handoff: Final status must be DONE, DONE_WITH_CONCERNS, BLOCKED, or NEEDS_CONTEXT.

- [x] SATR-030 [owner=codex] [deps=SATR-020] [scope=crates/nako-db,crates/nako-server]
  Goal: Persist and retrieve the enriched media facts without breaking existing
  probe rows or startup behavior.
  Validation: `cargo nextest run -p nako-db media_probe --no-fail-fast` and
  `cargo nextest run -p nako-server playback --no-fail-fast`
  Review: `review-workstream` for schema and migration compatibility.
  Evidence: migration or JSON compatibility tests proving old probe payloads
  still deserialize.
  Handoff: Split schema cleanup if compatibility pressure grows.

## M2 - Playback Requirement Deepening

- [x] SATR-040 [owner=codex] [deps=SATR-020] [scope=crates/nako-playback]
  Goal: Introduce a source-aware `TranscodeRequirement` that records selected
  streams, output codecs/container, bitrate/resolution/audio constraints,
  subtitle strategy, HDR/tone-map intent, and explicit transcode reasons.
  Validation: `cargo nextest run -p nako-playback --no-fail-fast`
  Review: `review-workstream` for planner boundary and public DTO drift.
  Evidence: planner tests that distinguish codec, profile, bit-depth, HDR,
  subtitle, and audio-channel reasons.
  Handoff: Public Client DTO changes must remain stable and redaction-safe.

- [x] SATR-050 [owner=codex] [deps=SATR-040] [scope=crates/nako-api,crates/nako-server]
  Goal: Map the richer playback/transcode reasons into Admin/Public API surfaces
  without exposing host command details.
  Validation: `cargo nextest run -p nako-api --no-fail-fast` and
  `cargo nextest run -p nako-server playback --no-fail-fast`
  Review: pending `review-workstream` for generated client contract impact.
  Evidence: Public playback decisions expose typed decision/report reasons while
  hiding internal `TranscodeRequirement` details; Admin readiness/support
  evidence exposes source-aware hardware fallback reasons and redaction-safe
  runtime metrics. `cargo nextest run -p nako-api --no-fail-fast` passed after
  refreshing the Admin TypeScript contract and mock response shape.
  Handoff: Public SDK remains intentionally stable for internal requirement
  details; Admin contract carries the support/runtime evidence.

## M3 - Source-Aware Pipeline And FFmpeg Command Planning

- [x] SATR-060 [owner=codex] [deps=SATR-040] [scope=crates/nako-transcode,crates/nako-server/src/app/playback]
  Goal: Extend `TranscodePipelineRequest` and planner logic so decode/filter/
  encode selections use input codec, profile, bit depth, HDR, subtitle, and
  output constraints.
  Validation: `cargo nextest run -p nako-transcode pipeline --no-fail-fast`,
  `cargo nextest run -p nako-server playback --no-fail-fast`
  Review: pending `review-workstream` for fallback semantics and stage ownership.
  Evidence: planner tests for source codec/bit-depth incompatibility fallback
  and server HLS regression proving ffprobe source facts feed execution policy.
  Handoff: Current source-aware hardware decode guard covers VAAPI, QSV, and
  VideoToolbox for H.264 8-bit input; deeper per-device codec/profile matrices
  should be expanded in SATR-070/SATR-080 follow-ups.

- [x] SATR-070 [owner=codex] [deps=SATR-060] [scope=crates/nako-transcode/src/ffmpeg.rs]
  Goal: Split HLS FFmpeg command building into device/input, filter graph,
  video encoder, audio encoder, subtitle, and HLS muxer components.
  Validation: `cargo nextest run -p nako-transcode ffmpeg --no-fail-fast`
  Review: pending `review-workstream` for argument ordering and path redaction.
  Evidence: command-plan tests for CPU, VAAPI, NVENC, QSV, subtitle omission,
  and HLS muxer options; full `nako-transcode` and `nako-server playback` gates
  passed after the split.
  Handoff: HLS command construction is now staged, and QSV `-hwaccel qsv` is
  emitted only in the pre-input device stage.

## M4 - Runtime Supervision And Progressive HLS Foundation

- [x] SATR-080 [owner=codex] [deps=SATR-070] [scope=crates/nako-core,crates/nako-db,crates/nako-transcode,crates/nako-server]
  Goal: Add FFmpeg progress parsing and persist redaction-safe transcode
  session metrics for Admin diagnostics and future playback throttling.
  Validation: `cargo nextest run -p nako-transcode progress --no-fail-fast` and
  `cargo nextest run -p nako-server playback --no-fail-fast`
  Review: pending `review-workstream` for cancellation and persistence behavior.
  Evidence: FFmpeg progress parser tests, HLS runner metric assertions, SQLite
  transcode session round-trip coverage, and server HLS persisted metric
  assertions.
  Handoff: Metrics store bounded numeric progress facts only; raw paths,
  command lines, and stderr remain outside the metric payload.

- [x] SATR-090 [owner=codex] [deps=SATR-080] [scope=crates/nako-server/src/app/playback]
  Goal: Prepare HLS serving for progressive segment readiness, throttling, and
  segment cleanup without regressing the existing completed-output path.
  Validation: `cargo nextest run -p nako-server hls --no-fail-fast`
  Review: pending `review-workstream` for route behavior and artifact
  lifecycle.
  Evidence: HLS route coverage now proves completed output still serves,
  missing running segments return not-ready conflicts, existing running
  segments are streamable, throttle waits once for an in-flight segment, and
  cleanup removes stale sibling `.ts` files without deleting the requested
  segment.
  Handoff: Adaptive ladders and fMP4 remain follow-ons.

## M5 - Closeout

- [x] SATR-100 [owner=codex] [deps=SATR-090] [scope=docs/workstreams/source-aware-transcode-runtime]
  Goal: Verify the lane, record evidence, close or split remaining work, and
  update ADR/workstream status.
  Validation: `cargo fmt --all -- --check`, `git diff --check`, focused package
  gates from `EVIDENCE_AND_GATES.md`
  Review: completed with no blocking workstream or code-quality findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`,
  `CLOSEOUT.md`
  Handoff: Follow-ons are adaptive ladders, fMP4, rsmpeg adapter feasibility,
  and remote transcode workers.
